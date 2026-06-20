# VSCode-Inspired Layout Architecture

**Inspiration Source:** Microsoft Visual Studio Code (proven professional layout)
**Status:** Reference Architecture
**Implementation:** 4-6 weeks

---

## VSCode Layout Elements We Should Adopt

### 1. Activity Bar (Left Edge)
**Purpose:** Quick navigation between different views
**Width:** 50px (icons only)
**Contents:**
- Notebooks icon
- Explorer/Files
- Search
- Source Control (Git)
- Extensions/Connections
- Settings

**Benefits:**
- Always visible, minimal space
- Quick view switching
- Professional appearance
- Similar to user expectations

### 2. Primary Sidebar (Collapsible)
**Width:** 240px (expanded) | 0px (collapsed)
**Transition:** Smooth 300ms
**Contents:** Dynamic based on activity bar selection

Examples:
- **Notebooks:** Recent, starred, all notebooks
- **Explorer:** File tree, cloud storage
- **Search:** Global search, replace
- **Source Control:** Git status, branches
- **Extensions:** Available, installed connections

### 3. Central Editor Group
**Width:** 1fr (takes remaining space)
**Primary Content:** Notebook cells, code editor
**Ratio:** ~70% of screen width (when both sidebars open)

Layout:
```
┌─ Cell Input ──────────────────┐
│ Monaco Editor                 │
├────────────────────────────────┤
│ Cell Output / Results          │
│ (Expandable)                   │
└────────────────────────────────┘
```

### 4. Secondary Sidebar (Right Panel)
**Width:** 300px (expanded) | 0px (collapsed)
**Trigger:** Icon in top-right or activity bar
**Contents:**
- Variables inspector (like VSCode debugger watch)
- Execution history
- Comments/annotations
- AI assistance panel
- Settings

### 5. Bottom Panel
**Height:** 200px (expanded) | 0px (collapsed)
**Trigger:** Icon in bottom-right
**Contents:**
- Terminal / Output console
- Execution logs
- Errors and warnings
- Execution statistics
- Connected services status

---

## Detailed Layout Structure

```
┌────────────────────────────────────────────────────────────┐
│ Command Palette (Cmd/Ctrl+K)                              │
│ File: notebook.ipynb | Settings | Help                    │
├─┬───────────────────────────────────────────────────────┬─┤
│▪│                                                       │▪│
│▪│   ACTIVITY BAR                                        │▪│
│▪│  (Notebooks)                                          │▪│
│ │  (Explorer)                 CENTRAL CODE EDITOR      │▪│
│ │  (Search)                   (Primary Focus)           │▪│
│ │  (Git)                      70% of width             │▪│
│ │  (Extensions)                                         │▪│
│ │  ────────                   ┌─────────────────────┐  │▪│
│ │                             │ In [1]:             │  │▪│
│ │                             │ import pandas       │  │▪│
│ │  SIDEBAR                    │ import numpy        │  │▪│
│ │  (240px, collapsible)       │                     │  │▪│
│ │                             │ [Run] [Share] [...]│  │▪│
│ │  Notebooks:                 └─────────────────────┘  │▪│
│ │  • Recent                                             │▪│
│ │  • Starred                  Output:                   │▪│
│ │  • All                      │ array([1, 2, 3])  │    │▪│
│ │                             └─────────────────────┘  │▪│
│ │                                                       │▪│
│ │                             [Add Cell] [+]            │▪│
│▪│                                                       │▪│
│▪│                                       VARIABLES      │▪│
│▪│                                       INSPECTOR      │▪│
│▪│                                       (300px,        │▪│
│▪│                                        collapsible)  │▪│
│▪│                                                       │▪│
├─┴───────────────────────────────────────────────────────┴─┤
│ BOTTOM PANEL (200px, collapsible)                         │
│ Execution Logs | Errors | Terminal | Status              │
│ [12 errors] [3 warnings] [Status: ✓ Connected]           │
└────────────────────────────────────────────────────────────┘
```

---

## CSS Grid Implementation

```css
.editor-layout {
  display: grid;
  grid-template-columns: 50px 240px 1fr 300px;
  grid-template-rows: 40px 1fr 200px;
  gap: 0;
  height: 100vh;
  
  grid-template-areas:
    "header header header header"
    "activity sidebar editor inspector"
    "activity bottom bottom bottom";
}

.activity-bar {
  grid-area: activity;
  width: 50px;
  background: var(--bg-secondary);
  border-right: 1px solid var(--border-light);
  
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 8px 0;
  gap: 4px;
}

.sidebar {
  grid-area: sidebar;
  width: 240px;
  transition: width 300ms ease;
  overflow: hidden;
  
  &.collapsed {
    width: 0;
    border: none;
  }
}

.editor-group {
  grid-area: editor;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.inspector-panel {
  grid-area: inspector;
  width: 300px;
  transition: width 300ms ease;
  overflow: hidden;
  border-left: 1px solid var(--border-light);
  
  &.collapsed {
    width: 0;
    border: none;
  }
}

.bottom-panel {
  grid-area: bottom;
  height: 200px;
  transition: height 300ms ease;
  overflow: hidden;
  border-top: 1px solid var(--border-light);
  
  &.collapsed {
    height: 0;
    border: none;
  }
}
```

---

## Activity Bar Icons & Functions

```
╔═════════════════╗
║  ≡ (Notebooks)  ║  Active: List recent/starred/all notebooks
║  ⊡ (Explorer)   ║  Files, cloud storage, recent files
║  ⊕ (Search)     ║  Global search across notebook cells
║  ⎇ (Git)        ║  Push/pull/sync notebooks to GitHub
║  ◉ (Connect)    ║  External connections status
║═════════════════║  ──────────────────────────────
║  ⚙ (Settings)   ║  User settings, preferences
╚═════════════════╝
```

---

## Sidebar Views (Dynamic)

### Notebooks View (Active by default)
```
Notebooks
├── Recent
│   ├── Data Analysis (2h ago)
│   ├── ML Pipeline (1d ago)
│   └── SQL Queries (3d ago)
├── ☆ Starred
│   ├── ★ Main Analysis
│   └── ★ Template
└── All
    ├── Folder 1
    ├── Folder 2
    └── ...
```

### Explorer View
```
Explorer
├── Notebooks
├── Cloud Storage
│   ├── S3: my-bucket
│   ├── GCS: data-lake
│   └── Azure: container
└── Recent Files
```

### Search View
```
Search (Ctrl+Shift+F)
┌─────────────────────┐
│ [Search text....] ✕ │
│ [Replace with..] → │
└─────────────────────┘
Results (15)
├── notebook1.ipynb (3 matches)
├── notebook2.ipynb (5 matches)
└── notebook3.ipynb (7 matches)
```

### Git/Connections View
```
Connections
├── GitHub
│   ├── Status: Connected
│   ├── Branch: main
│   └── Changes: 2 files
├── Snowflake
│   ├── Status: Connected
│   ├── Database: PROD
│   └── Latency: 145ms
└── S3
    ├── Status: Connected
    └── Buckets: 3
```

---

## Inspector Panel Tabs

```
┌──────────────────────┐
│ V R H                │  Tabs:
├──────────────────────┤  V = Variables
│ Variables:           │  R = Run History
│ df: DataFrame        │  H = Help/Docs
│ x: int = 42          │
│ result: list[...]    │
│                      │
│ Search variables ... │
└──────────────────────┘
```

---

## Bottom Panel Sections

```
┌────────────────────────────────┐
│ [■ Output] [⚠ Problems] [≡]   │
├────────────────────────────────┤
│ [Connection Status]             │
│ ✓ DuckDB: Connected             │
│ ✓ Snowflake: Connected (245ms)  │
│ ✓ S3: Connected                 │
│ ✗ BigQuery: Error - auth failed │
│                                 │
│ [Execution Log]                 │
│ 2026-06-20 10:45:23 - Cell 1    │
│ Executed successfully (1.2s)    │
│ 2026-06-20 10:46:01 - Cell 2    │
│ Error: NameError: x not defined │
└────────────────────────────────┘
```

---

## Command Palette

**Trigger:** Cmd/Ctrl+Shift+P

```
Type command or search...
────────────────────────
> Save Notebook
> Export as PDF
> Share Notebook
> Push to GitHub
> Run All Cells
> Format Code
> Settings
> Keyboard Shortcuts
> Extensions
> Terminal
```

---

## Keyboard Shortcuts (VSCode-inspired)

```
Cmd/Ctrl+B          Toggle left sidebar
Cmd/Ctrl+J          Toggle bottom panel
Cmd/Ctrl+Shift+E    Focus explorer
Cmd/Ctrl+Shift+F    Focus search
Cmd/Ctrl+Shift+G    Focus git
Cmd/Ctrl+Shift+D    Focus variables
Cmd/Ctrl+K, Cmd/Ctrl+P  Command palette
Cmd/Ctrl+Shift+P    Command palette (alternative)
Shift+Enter         Run current cell
Cmd/Ctrl+Enter      Run cell and select below
Cmd/Ctrl+Shift+Enter Run cell
Cmd/Ctrl+/          Toggle comment
Cmd/Ctrl+L          Select line
```

---

## Responsive Breakpoints

### Desktop (> 1440px)
All panels visible, full functionality

### Laptop (1024px - 1440px)
- Sidebar: 200px (narrower)
- Inspector: 280px
- Bottom: 180px
- All features available

### Tablet (768px - 1024px)
- Activity bar + Auto-hiding sidebar (drawer)
- Inspector: Hidden by default (toggle only)
- Bottom: Hidden by default
- Touch-optimized

### Mobile (< 768px)
```
┌─────────────────────┐
│ ☰ Notebook Name [+] │ Header
├─────────────────────┤
│                     │
│   EDITOR (Full)     │ Main area
│                     │
├─────────────────────┤
│ Output              │ Collapsible
└─────────────────────┘

Sidebars: Drawer menus (swipe)
Panels: Bottom sheets (drag up)
```

---

## Migration from Current Layout

### Phase 1: Add Activity Bar (Week 1)
- Implement 50px activity bar
- Add icons with tooltips
- Connect to sidebar visibility

### Phase 2: Refactor Sidebar (Week 2)
- Convert to dynamic view system
- Implement Notebooks, Explorer, Search, Git views
- Add collapse animation

### Phase 3: Implement Bottom Panel (Week 3)
- Create bottom panel component
- Add output/logs viewer
- Add execution status

### Phase 4: Polish & Responsive (Week 4)
- Mobile responsiveness
- Touch interactions
- Keyboard shortcuts
- Theme compatibility

---

## VSCode Features to Adopt

1. **Activity Bar** - Professional icon-based navigation
2. **Dynamic Sidebars** - Context-aware content
3. **Bottom Panel** - Terminal/logs area
4. **Command Palette** - Quick command access
5. **View Switching** - Fast toggle between views
6. **Keyboard Shortcuts** - Power user efficiency
7. **Theming** - Light/dark/custom themes
8. **Extensions** - Extensible architecture

---

## VSCode Features to Adapt

1. **Panel Sizes** - Draggable dividers to resize
2. **View Icons** - Badge counts (errors, changes)
3. **Breadcrumbs** - Navigation path in editor
4. **Quick Open** - File/cell quick selector
5. **Minimap** - Code preview (optional for notebooks)
6. **Status Bar** - Runtime info, connection status
7. **Zen Mode** - Full-screen code editing
8. **Accessibility** - Screen reader support

---

## Why VSCode Layout Works

1. **Central focus** on code (editor takes up to 70% space)
2. **Professional appearance** (proven by millions of developers)
3. **Efficient workflow** (quick view switching via activity bar)
4. **Scalable** (works on all screen sizes)
5. **Familiar** (users already know the pattern)
6. **Extensible** (easy to add new views)
7. **Accessibility** (designed for inclusive use)

---

## Success Metrics

- Code editor: 65-70% of screen width
- Activity bar toggle: < 300ms transition
- Keyboard shortcuts: 90%+ commonly used
- User familiarity: High (VSCode users recognize layout)
- Professional impression: Very high
- Accessibility: WCAG AAA compliant

---

This VSCode-inspired architecture provides a proven, professional layout that users already understand and trust.
