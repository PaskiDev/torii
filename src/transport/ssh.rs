//! Smart SSH transport over russh (pure-Rust, no openssl/libssh2).
//!
//! Each Service maps to one SSH exec:
//!   UploadPackLs / UploadPack    → `git-upload-pack 'owner/repo.git'`
//!   ReceivePackLs / ReceivePack  → `git-receive-pack 'owner/repo.git'`
//!
//! libgit2 owns pkt-line / pack negotiation. We bridge libgit2's blocking
//! Read/Write to russh's async Channel via a current-thread tokio runtime
//! and mpsc channels (one per direction).

use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::sync::Arc;

use git2::transport::{Service, SmartSubtransport, SmartSubtransportStream, Transport};
use git2::{Error, Remote};
use russh::client;
use russh::keys::{load_secret_key, PrivateKeyWithHashAlg};
use russh::ChannelMsg;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

pub fn factory(remote: &Remote<'_>) -> Result<Transport, Error> {
    Transport::smart(remote, false, SshSubtransport)
}

struct SshSubtransport;

impl SmartSubtransport for SshSubtransport {
    fn action(
        &self,
        url: &str,
        action: Service,
    ) -> Result<Box<dyn SmartSubtransportStream>, Error> {
        let (user, host, port, path) = parse_ssh_url(url)?;
        let cmd = match action {
            Service::UploadPackLs | Service::UploadPack => format!("git-upload-pack '{}'", path),
            Service::ReceivePackLs | Service::ReceivePack => {
                format!("git-receive-pack '{}'", path)
            }
        };
        let stream = SshStream::connect(user, host, port, cmd)
            .map_err(|e| Error::from_str(&format!("ssh transport: {}", e)))?;
        Ok(Box::new(stream))
    }

    fn close(&self) -> Result<(), Error> {
        Ok(())
    }
}

/// Parse `ssh://user@host:port/path` and `user@host:path` (scp-like).
fn parse_ssh_url(url: &str) -> Result<(String, String, u16, String), Error> {
    if let Some(rest) = url.strip_prefix("ssh://") {
        let (userhost, path) = rest
            .split_once('/')
            .ok_or_else(|| Error::from_str("ssh url missing path"))?;
        let (user, hostport) = userhost
            .split_once('@')
            .ok_or_else(|| Error::from_str("ssh url missing user"))?;
        let (host, port) = match hostport.split_once(':') {
            Some((h, p)) => (
                h.to_string(),
                p.parse().map_err(|_| Error::from_str("bad port"))?,
            ),
            None => (hostport.to_string(), 22),
        };
        return Ok((user.to_string(), host, port, path.to_string()));
    }
    // scp-like: git@github.com:owner/repo.git
    let (user, rest) = url
        .split_once('@')
        .ok_or_else(|| Error::from_str("ssh url missing user"))?;
    let (host, path) = rest
        .split_once(':')
        .ok_or_else(|| Error::from_str("ssh url missing path"))?;
    Ok((user.to_string(), host.to_string(), 22, path.to_string()))
}

/// Bridges libgit2's blocking Read/Write to russh's async Channel.
struct SshStream {
    runtime: Arc<Runtime>,
    /// Bytes from server → libgit2 reads here.
    rx: mpsc::Receiver<Vec<u8>>,
    /// Pending leftover from a partial chunk.
    leftover: Vec<u8>,
    /// Bytes from libgit2 → server.
    tx: mpsc::Sender<Vec<u8>>,
    /// Set true when server EOF received; drain rx then return Ok(0).
    eof: bool,
}

impl SshStream {
    fn connect(
        user: String,
        host: String,
        port: u16,
        cmd: String,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let runtime = Arc::new(
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?,
        );
        let (read_tx, read_rx) = mpsc::channel::<Vec<u8>>(32);
        let (write_tx, mut write_rx) = mpsc::channel::<Vec<u8>>(32);

        runtime.block_on(async {
            let key_path = ssh_key_path()?;
            let key = load_secret_key(&key_path, None)
                .map_err(|e| format!("load key {:?}: {}", key_path, e))?;

            let config = Arc::new(client::Config {
                inactivity_timeout: Some(std::time::Duration::from_secs(60)),
                ..Default::default()
            });
            let mut session = client::connect(config, (host.as_str(), port), Handler).await?;

            let hash = session.best_supported_rsa_hash().await?.flatten();
            let auth = session
                .authenticate_publickey(
                    &user,
                    PrivateKeyWithHashAlg::new(Arc::new(key), hash),
                )
                .await?;
            if !auth.success() {
                return Err::<(), Box<dyn std::error::Error + Send + Sync>>(
                    "ssh auth failed (only key from disk supported in spike)".into(),
                );
            }

            let mut channel = session.channel_open_session().await?;
            channel.exec(true, cmd.as_bytes()).await?;

            // Spawn pump task: forward channel msgs → read_tx, drain write_rx → channel.
            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        msg = channel.wait() => {
                            match msg {
                                Some(ChannelMsg::Data { data }) => {
                                    if read_tx.send(data.to_vec()).await.is_err() {
                                        break;
                                    }
                                }
                                Some(ChannelMsg::Eof) | Some(ChannelMsg::ExitStatus { .. }) => {
                                    // keep draining for trailing data after exit
                                }
                                None => break,
                                _ => {}
                            }
                        }
                        out = write_rx.recv() => {
                            match out {
                                Some(buf) => {
                                    if channel.data(&buf[..]).await.is_err() {
                                        break;
                                    }
                                }
                                None => {
                                    let _ = channel.eof().await;
                                }
                            }
                        }
                    }
                }
            });

            Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
        })?;

        Ok(Self {
            runtime,
            rx: read_rx,
            leftover: Vec::new(),
            tx: write_tx,
            eof: false,
        })
    }
}

impl Read for SshStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if !self.leftover.is_empty() {
            let n = self.leftover.len().min(buf.len());
            buf[..n].copy_from_slice(&self.leftover[..n]);
            self.leftover.drain(..n);
            return Ok(n);
        }
        if self.eof {
            return Ok(0);
        }
        let chunk = self.runtime.block_on(self.rx.recv());
        match chunk {
            Some(data) => {
                let n = data.len().min(buf.len());
                buf[..n].copy_from_slice(&data[..n]);
                if n < data.len() {
                    self.leftover.extend_from_slice(&data[n..]);
                }
                Ok(n)
            }
            None => {
                self.eof = true;
                Ok(0)
            }
        }
    }
}

impl Write for SshStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let owned = buf.to_vec();
        let n = owned.len();
        self.runtime
            .block_on(self.tx.send(owned))
            .map_err(|e| io::Error::new(io::ErrorKind::BrokenPipe, e))?;
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn ssh_key_path() -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let home = std::env::var_os("HOME").ok_or("HOME not set")?;
    for name in ["id_ed25519", "id_rsa"] {
        let p = PathBuf::from(&home).join(".ssh").join(name);
        if p.exists() {
            return Ok(p);
        }
    }
    Err("no ssh key found in ~/.ssh (id_ed25519 or id_rsa)".into())
}

struct Handler;

impl client::Handler for Handler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _key: &russh::keys::ssh_key::PublicKey,
    ) -> Result<bool, Self::Error> {
        // SPIKE: accept any host key. Production must verify against ~/.ssh/known_hosts.
        Ok(true)
    }
}
