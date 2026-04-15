# 🎨 Torii GUI Vision

## Overview
Torii GUI será la interfaz visual para todas las funcionalidades del CLI, construida con **Tauri** para máxima portabilidad y rendimiento.

## 🚀 Plataformas Soportadas
- **Desktop**: Windows, macOS, Linux
- **Mobile**: iOS, Android
- **Web**: Progressive Web App (PWA)

## 💡 Ventajas del Enfoque CLI-First

### ✅ Ya Tenemos Todo Implementado
El CLI ya tiene **todas** las funcionalidades core:
- ✅ Gestión multi-plataforma de repositorios
- ✅ Operaciones batch en múltiples plataformas
- ✅ Snapshots locales
- ✅ Mirroring automático
- ✅ Workflows personalizados
- ✅ Reescritura de historial
- ✅ Integración con SSH

### 🎯 El GUI Solo Necesita
1. **Llamar a los comandos CLI** - Ya funcionan perfectamente
2. **Mostrar resultados** - Parsear output JSON/texto
3. **Proveer UX visual** - Botones, formularios, gráficos
4. **Añadir visualizaciones** - Grafos de commits, timelines, etc.

## 🎨 Mockups de Funcionalidades Clave

### 1. Dashboard Principal
```
┌─────────────────────────────────────────────────────┐
│  🏠 Torii                    [user@email.com] ⚙️    │
├─────────────────────────────────────────────────────┤
│                                                     │
│  📊 Repositories Overview                          │
│  ┌──────────┬──────────┬──────────┬──────────┐    │
│  │ GitHub   │ GitLab   │ Codeberg │ Gitea    │    │
│  │   12     │    8     │    5     │    3     │    │
│  └──────────┴──────────┴──────────┴──────────┘    │
│                                                     │
│  🔄 Recent Activity                                │
│  • my-project pushed to GitHub, GitLab             │
│  • snapshot created: pre-refactor                  │
│  • mirror synced: backup-server                    │
│                                                     │
│  ⚡ Quick Actions                                   │
│  [+ New Repo] [📸 Snapshot] [🔄 Sync All]         │
│                                                     │
└─────────────────────────────────────────────────────┘
```

### 2. Multi-Platform Repo Creation
```
┌─────────────────────────────────────────────────────┐
│  Create Repository on Multiple Platforms           │
├─────────────────────────────────────────────────────┤
│                                                     │
│  Repository Name: [my-awesome-project          ]   │
│  Description:     [An amazing tool for...      ]   │
│                                                     │
│  📍 Select Platforms:                              │
│  ☑ GitHub        ☑ GitLab       ☑ Codeberg        │
│  ☐ Gitea         ☐ Forgejo      ☐ Custom          │
│                                                     │
│  🔒 Visibility:   ⚪ Public  ⚫ Private            │
│                                                     │
│  ☑ Push code after creation                        │
│  ☑ Add as mirrors for auto-sync                    │
│                                                     │
│         [Cancel]              [Create on 3 ▼]      │
│                                                     │
└─────────────────────────────────────────────────────┘
```

### 3. Batch Operations Progress
```
┌─────────────────────────────────────────────────────┐
│  Creating 'my-project' on 3 platforms...           │
├─────────────────────────────────────────────────────┤
│                                                     │
│  📦 GitHub     ✅ Created                          │
│     └─ https://github.com/user/my-project          │
│                                                     │
│  📦 GitLab     ⏳ Creating...                      │
│     └─ [████████░░] 80%                            │
│                                                     │
│  📦 Codeberg   ⏸️  Waiting...                      │
│                                                     │
│  ────────────────────────────────────────────────  │
│  Progress: 1/3 completed                           │
│                                                     │
└─────────────────────────────────────────────────────┘
```

### 4. Visual Snapshot Manager
```
┌─────────────────────────────────────────────────────┐
│  📸 Snapshots                          [+ Create]   │
├─────────────────────────────────────────────────────┤
│                                                     │
│  Timeline View:                                     │
│                                                     │
│  2026-04-11  ●─────●─────●                         │
│              │     │     └─ post-refactor          │
│              │     └─────── pre-refactor           │
│              └───────────── initial-setup          │
│                                                     │
│  Selected: pre-refactor                            │
│  ┌─────────────────────────────────────────────┐  │
│  │ Created: 2026-04-11 10:30                   │  │
│  │ Files: 42 changed, 1,234 insertions         │  │
│  │ Message: "Before major refactoring"         │  │
│  │                                              │  │
│  │ [Restore] [Compare] [Delete]                │  │
│  └─────────────────────────────────────────────┘  │
│                                                     │
└─────────────────────────────────────────────────────┘
```

### 5. Mirror Sync Dashboard
```
┌─────────────────────────────────────────────────────┐
│  🔄 Mirrors & Sync Status                          │
├─────────────────────────────────────────────────────┤
│                                                     │
│  Master: github.com/user/torii                     │
│                                                     │
│  Mirrors:                                          │
│  ┌──────────────────────────────────────────────┐ │
│  │ ✅ GitLab        Last sync: 2 min ago        │ │
│  │ ✅ Codeberg      Last sync: 5 min ago        │ │
│  │ ⚠️  Gitea        Last sync: 2 hours ago      │ │
│  │ ❌ Custom        Failed: connection timeout   │ │
│  └──────────────────────────────────────────────┘ │
│                                                     │
│  Auto-sync: ☑ Enabled  Interval: [30] minutes     │
│                                                     │
│  [Sync Now] [Add Mirror] [Configure]               │
│                                                     │
└─────────────────────────────────────────────────────┘
```

### 6. Commit Graph Visualizer
```
┌─────────────────────────────────────────────────────┐
│  📊 Commit History                                  │
├─────────────────────────────────────────────────────┤
│                                                     │
│    main    ●───●───●───●───●  HEAD                │
│             \       \                               │
│    feature   ●───●───●  (merged)                   │
│                                                     │
│  Filters: [All Branches ▼] [Last 30 days ▼]       │
│                                                     │
│  Recent Commits:                                    │
│  ● Add multi-platform support      2 hours ago     │
│  ● Fix compilation errors          3 hours ago     │
│  ● Implement batch operations      5 hours ago     │
│                                                     │
│  [Rewrite History] [Clean] [Export]                │
│                                                     │
└─────────────────────────────────────────────────────┘
```

## 🎯 Funcionalidades Específicas del GUI

### Desktop
- **Drag & Drop** - Arrastrar archivos para commit
- **System Tray** - Notificaciones de sync
- **Keyboard Shortcuts** - Workflow rápido
- **Multi-window** - Múltiples repos abiertos
- **Terminal integrada** - Acceso directo al CLI

### Mobile
- **Quick Actions** - Crear repo, ver status
- **Push Notifications** - Alertas de sync/mirrors
- **QR Code** - Compartir repos fácilmente
- **Offline Mode** - Ver snapshots sin conexión
- **Biometric Auth** - Seguridad para tokens

## 🛠️ Stack Tecnológico

### Frontend
- **Framework**: React + TypeScript
- **UI Library**: shadcn/ui + Tailwind CSS
- **Icons**: Lucide React
- **Charts**: Recharts / D3.js
- **State**: Zustand / Jotai

### Backend (Tauri)
- **Core**: Rust (ya tenemos todo el CLI)
- **IPC**: Tauri Commands
- **Storage**: SQLite para cache local
- **Updates**: Tauri Updater

## 📱 Ventajas del Enfoque

### 1. **Desarrollo Rápido**
```rust
// El CLI ya hace todo el trabajo pesado
#[tauri::command]
fn create_repo_multi_platform(name: String, platforms: Vec<String>) -> Result<String> {
    // Simplemente llamamos al CLI existente
    Command::new("torii")
        .args(&["repo", &name, "--create", "--platforms", &platforms.join(",")])
        .output()
}
```

### 2. **Consistencia Total**
- CLI y GUI usan **exactamente** la misma lógica
- Bugs arreglados en uno se arreglan en ambos
- Features nuevas disponibles inmediatamente

### 3. **Testing Simplificado**
- CLI ya está testeado
- GUI solo testea la capa visual
- E2E tests reutilizan comandos CLI

### 4. **Performance**
- Tauri es **extremadamente** ligero (~3MB)
- Rust backend = velocidad nativa
- Sin Electron = menos RAM

## 🎨 Temas y Personalización

```typescript
// Tema oscuro/claro
const themes = {
  light: {
    primary: '#6366f1',
    background: '#ffffff',
    text: '#1f2937'
  },
  dark: {
    primary: '#818cf8',
    background: '#111827',
    text: '#f9fafb'
  },
  torii: {
    // Tema personalizado japonés
    primary: '#dc2626',
    accent: '#fbbf24',
    background: '#0f172a'
  }
}
```

## 🚀 Roadmap GUI

### Phase 1: MVP Desktop (Q2 2026)
- [ ] Dashboard principal
- [ ] Gestión básica de repos
- [ ] Operaciones multi-plataforma
- [ ] Snapshots visuales

### Phase 2: Advanced Features (Q3 2026)
- [ ] Commit graph visualizer
- [ ] Mirror sync dashboard
- [ ] Custom workflows editor
- [ ] Settings & preferences

### Phase 3: Mobile (Q4 2026)
- [ ] iOS app
- [ ] Android app
- [ ] Sync con desktop
- [ ] Push notifications

### Phase 4: Collaboration (Q1 2027)
- [ ] Team features
- [ ] Shared snapshots
- [ ] Real-time sync status
- [ ] Activity feed

## 💡 Killer Features del GUI

1. **Visual Multi-Platform Manager**
   - Ver todos tus repos en todas las plataformas
   - Crear/eliminar en batch con clicks
   - Drag & drop para configurar mirrors

2. **Snapshot Timeline**
   - Línea de tiempo visual de snapshots
   - Preview de cambios antes de restaurar
   - Comparación visual entre snapshots

3. **Smart Notifications**
   - Alertas cuando mirrors se desincronicen
   - Recordatorios de snapshots automáticos
   - Notificaciones de updates disponibles

4. **One-Click Workflows**
   - Templates predefinidos
   - Workflows personalizados visuales
   - Automatización con GUI builder

## 🎯 Conclusión

El GUI de Torii será **increíblemente fácil de desarrollar** porque:

✅ **Todo el backend ya existe** (CLI completo)  
✅ **Solo necesitamos la capa visual**  
✅ **Tauri hace el resto** (empaquetado, updates, etc.)  
✅ **Cross-platform automático**  
✅ **Performance nativo**  

**Estimación**: Con el CLI ya completo, el MVP del GUI desktop se puede tener en **2-3 meses** de desarrollo. El mobile en otros **2-3 meses**.

---

*"CLI-first approach = GUI development on easy mode"* 🚀
