# Torii ⛩️ - Roadmap & Checklist

## ✅ Implementado (v0.1.0 - v0.3.0)

### Core Git Operations
- [x] `torii init` - Inicializar repositorio
- [x] `torii save` - Commit simplificado
  - [x] `--amend` - Modificar último commit
  - [x] `--revert` - Revertir commit específico
  - [x] `--reset` - Reset a commit específico (soft/mixed/hard)
- [x] `torii sync` - Push/pull inteligente
  - [x] `--pull` - Solo pull
  - [x] `--push` - Solo push
  - [x] `--force` - Force push
  - [x] `--fetch` - Solo fetch sin merge
  - [x] Integración de ramas (`sync <branch>`)
- [x] `torii status` - Estado del repositorio
- [x] `torii log` - Historial de commits
- [x] `torii diff` - Mostrar cambios
- [x] `torii branch` - Gestión de ramas
  - [x] Listar ramas
  - [x] Cambiar a rama existente
  - [x] Crear nueva rama (`-c`)
  - [x] Eliminar rama (`-d`)
  - [x] Renombrar rama (`--rename`)
- [x] `torii clone` - Clonar repositorios
  - [x] Shortcuts de plataforma (github, gitlab, etc.)
  - [x] URLs completas

### Advanced Git Features
- [x] `torii cherry-pick` - Aplicar commits específicos
- [x] `torii blame` - Ver quién cambió cada línea
- [x] `torii tag` - Gestión de tags
  - [x] `create` - Crear tags
  - [x] `list` - Listar tags
  - [x] `delete` - Eliminar tags
  - [x] `push` - Push tags
  - [x] `show` - Mostrar detalles
- [x] `torii integrate` - Merge/rebase inteligente (DEPRECADO → usar `sync <branch>`)
- [x] `torii switch` - Cambio de ramas (DEPRECADO → usar `branch <name>`)

### Snapshot System
- [x] `torii snapshot create` - Crear snapshots
- [x] `torii snapshot list` - Listar snapshots
- [x] `torii snapshot restore` - Restaurar snapshot
- [x] `torii snapshot delete` - Eliminar snapshot
- [x] `torii snapshot stash` - Stash temporal
- [x] `torii snapshot unstash` - Restaurar stash
- [x] `torii snapshot undo` - Deshacer última operación
- [x] `torii undo` - Acceso rápido a snapshot undo
- [x] Auto-snapshot en operaciones críticas
- [x] Sistema de retención configurable

### Mirror Management
- [x] `torii mirror add-master` - Añadir mirror principal
- [x] `torii mirror add-slave` - Añadir mirror esclavo
- [x] `torii mirror list` - Listar mirrors
- [x] `torii mirror sync` - Sincronizar mirrors
- [x] `torii mirror set-master` - Cambiar mirror principal
- [x] `torii mirror remove` - Eliminar mirror
- [x] `torii mirror autofetch` - Configurar autofetch
- [x] Soporte para 8 plataformas:
  - [x] GitHub
  - [x] GitLab
  - [x] Codeberg
  - [x] Bitbucket
  - [x] Gitea
  - [x] Forgejo
  - [x] SourceHut (srht)
  - [x] SourceForge
- [x] Soporte para servidores Git custom (URLs completas)
- [x] Auto-detección SSH/HTTPS

### Custom Workflows
- [x] `torii custom add` - Crear workflow personalizado
- [x] `torii custom list` - Listar workflows
- [x] `torii custom run` - Ejecutar workflow
- [x] `torii custom remove` - Eliminar workflow
- [x] Persistencia en `~/.config/torii/aliases.toml`
- [x] Soporte para argumentos dinámicos

### History Management
- [x] `torii history rewrite` - Reescribir fechas de commits
- [x] `torii history clean` - Limpiar repositorio (gc, reflog)
- [x] `torii history verify-remote` - Verificar estado remoto

### Utilities
- [x] `torii ssh-check` - Verificar configuración SSH
- [x] Sistema de errores mejorado
- [x] Detección automática de protocolo
- [x] ToriIgnore (exclusión de archivos en snapshots)

### Documentation
- [x] README principal actualizado
- [x] 10 traducciones i18n completas
- [x] Documentación de contribución
- [x] Documentación de seguridad
- [x] Publicación defensiva de patentes
- [x] Guías de desarrollo y testing

---

## 🚧 En Progreso / Próximas Features

### CI/CD Portable (Mencionado en README pero NO implementado)
- [ ] `torii ci validate` - Validar configuración CI/CD
- [ ] `torii ci generate` - Generar configs para múltiples plataformas
- [ ] `torii ci import` - Importar configs existentes
- [ ] `torii ci sync` - Sincronizar configs entre plataformas
- [ ] `torii ci diff` - Mostrar diferencias de configs
- [ ] Soporte para:
  - [ ] GitHub Actions
  - [ ] GitLab CI
  - [ ] Bitbucket Pipelines
  - [ ] Drone CI
  - [ ] Jenkins

### GUI (Tauri)
- [ ] Interfaz gráfica de escritorio
- [ ] Visualización de historial
- [ ] Gestión visual de ramas
- [ ] Diff visual
- [ ] Gestión de mirrors visual
- [ ] Configuración visual
- [ ] Soporte multiplataforma:
  - [ ] Windows
  - [ ] macOS
  - [ ] Linux

### TUI (Ratatui)
- [ ] Interfaz de terminal interactiva
- [ ] Navegación con teclado
- [ ] Visualización de estado en tiempo real
- [ ] Gestión interactiva de snapshots
- [ ] Vista de árbol de commits

### Mobile (Futuro)
- [ ] App iOS (Tauri)
- [ ] App Android (Tauri)
- [ ] Monitoreo de repositorios
- [ ] Operaciones básicas
- [ ] Notificaciones

### Mejoras de Mirror
- [ ] Sincronización bidireccional inteligente
- [ ] Gestión de PRs multi-plataforma desde Torii
- [ ] Detección y resolución de conflictos entre mirrors
- [ ] Dashboard de estado de mirrors
- [ ] Webhooks para sincronización automática
- [ ] Reconciliación de estado tras fallos parciales

### Análisis de Código (Premium - Planeado)
- [ ] Análisis estático de código
- [ ] Detección de code smells
- [ ] Sugerencias de refactoring
- [ ] Métricas de calidad
- [ ] Integración con herramientas de análisis

### Mejoras de Snapshots
- [ ] Implementar ToriIgnore completo para snapshots
- [ ] Compresión de snapshots antiguos
- [ ] Exportar/importar snapshots
- [ ] Snapshots remotos (backup en la nube)
- [ ] Diff visual entre snapshots

### Performance & Optimización
- [ ] Caché de operaciones Git frecuentes
- [ ] Paralelización de operaciones de mirror
- [ ] Optimización de snapshots grandes
- [ ] Índice de búsqueda rápida en historial
- [ ] Lazy loading de datos pesados

### Seguridad
- [ ] Firma GPG de commits desde Torii
- [ ] Verificación de firmas
- [ ] Gestión de credenciales segura
- [ ] Auditoría de operaciones
- [ ] 2FA para operaciones críticas

### Integrations
- [ ] Plugin system
- [ ] API REST para integraciones
- [ ] Webhooks personalizados
- [ ] Integración con IDEs (VS Code, IntelliJ, etc.)
- [ ] Integración con gestores de proyectos (Jira, Trello, etc.)

### Testing & Quality
- [ ] Aumentar cobertura de tests (actualmente básica)
- [ ] Tests de integración completos
- [ ] Tests end-to-end
- [ ] Benchmarks de performance
- [ ] Fuzzing para robustez

### Documentation
- [ ] Video tutoriales
- [ ] Documentación interactiva
- [ ] Ejemplos de casos de uso complejos
- [ ] Guía de migración desde Git puro
- [ ] Best practices guide

---

## 🎯 Roadmap por Versiones

### v0.4.0 (Próxima - Q2 2026)
- [ ] Implementar sistema CI/CD portable completo
- [ ] Mejorar sistema de mirrors (bidireccional)
- [ ] ToriIgnore completo en snapshots
- [ ] Tests de integración completos

### v0.5.0 (Q3 2026)
- [ ] TUI básico con Ratatui
- [ ] Plugin system básico
- [ ] API REST para integraciones
- [ ] Mejoras de performance

### v0.6.0 (Q4 2026)
- [ ] GUI básico con Tauri (Desktop)
- [ ] Gestión de PRs multi-plataforma
- [ ] Firma GPG de commits
- [ ] Dashboard de mirrors

### v1.0.0 (Q1 2027)
- [ ] GUI completo y pulido
- [ ] Mobile apps (iOS/Android)
- [ ] Sistema de plugins maduro
- [ ] Análisis de código (Premium)
- [ ] Documentación completa

---

## 📊 Estado Actual

**Versión Actual**: v0.3.0
**Comandos Implementados**: 45+
**Plataformas Soportadas**: 8 + custom
**Idiomas Documentados**: 11 (EN + 10 traducciones)
**Cobertura de Tests**: Básica (necesita mejora)

**Progreso General**: ~40% del roadmap completo

---

## 🤝 Contribuciones

Si quieres contribuir a alguna de estas features, revisa:
- `CONTRIBUTING.md` - Guía de contribución
- `DEVELOPMENT.md` - Guía de desarrollo
- `TESTING.md` - Guía de testing

Abre un issue para discutir features grandes antes de implementarlas.

---

**Última actualización**: Abril 2026
