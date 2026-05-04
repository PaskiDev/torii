//! GET /api/v1/whoami — verify API key + load org info.

use serde::Deserialize;

use super::{check_status, CloudClient};
use crate::error::{Result, ToriiError};

#[derive(Debug, Clone, Deserialize)]
pub struct WhoAmI {
    pub org_id: String,
    pub org_name: String,
    pub org_slug: String,
    pub plan: String,
    pub seats: i64,
    pub suspended: bool,
}

pub fn whoami(client: &CloudClient) -> Result<WhoAmI> {
    let resp = client
        .get("/api/v1/whoami")
        .send()
        .map_err(|e| ToriiError::InvalidConfig(format!("whoami request: {}", e)))?;
    let resp = check_status(resp)?;
    resp.json::<WhoAmI>()
        .map_err(|e| ToriiError::InvalidConfig(format!("whoami parse: {}", e)))
}
