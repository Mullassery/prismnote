# Layout Restructure Plan - Code Editor as Central Focus

**Issue:** Code editor relegated to side panel - should be PRIMARY workspace
**Status:** Analysis Complete
**Implementation Effort:** 4-6 hours

---

## Problem Analysis

### Current Layout (Incorrect)
```
┌──────────────────────────────────────────┐
│ Header                                   │
├──────┬──────────────────────────────────┤
│      │                                  │
│ Nav  │   Code Editor (Takes less space) │
│      │                                  │
├──────┼──────────────────────────────────┤
│ Sidebar  │ Output / Results              │
│          │                              │
└──────────────────────────────────────────┘
```

**Problems:**
- Code editor compressed to fit sidebars
- Limited horizontal space for code
- Not optimal for long lines/complex code
- Output area also secondary
- Sidebars dominate the layout

### Target Layout (Correct)
```
┌──────────────────────────────────────────┐
│ Header / Toolbar                         │
├──────┬──────────────────────┬──────────┤
│      │                      │          │
│ Left │   CODE EDITOR        │ Right    │
│ Nav  │   (PRIMARY FOCUS)    │ Panel    │
│      │                      │          │
├──────┼──────────────────────┼──────────┤
│      │ OUTPUT / RESULTS     │          │
│ Sidebar  (Collapsible)       │ Collapsed│
│      │                      │          │
└──────────────────────────────────────────┘
```

**Improvements:**
- Code editor takes 60-70% of space
- Minimum 100+ characters per line visibility
- Output area below code
- Sidebars are secondary (collapsible)
- Primary workflow: write → execute → view results

---

## Proposed Layout Structure

### Three-Column Layout

```css
/* Main container */
.notebook-layout {
  display: grid;
  grid-template-columns: 240px 1fr 280px;  /* sidebar | code | panel */
  gap: 16px;
  height: 100vh;
}

/* Left sidebar - collapsible */
.left-sidebar {
  width: 240px;
  overflow-y: auto;
  background: var(--bg-secondary);
  /* Can collapse to icon bar only */
}

.left-sidebar.collapsed {
  width: 64px;
  /* Show only icons */
}

/* Central code editor area */
.editor-panel {
  grid-column: 2;
  display: flex;
  flex-direction: column;
  min-width: 0;
  
  > .cell {
    flex: 0 0 auto;
    border: 1px solid var(--border-light);
    border-radius: 8px;
    margin-bottom: 16px;
  }
  
  > .output {
    flex: 1;
    overflow-y: auto;
    background: var(--bg-secondary);
    border-radius: 8px;
    padding: 16px;
  }
}

/* Right panel - collapsible */
.right-panel {
  width: 280px;
  overflow-y: auto;
  background: var(--bg-secondary);
}

.right-panel.collapsed {
  width: 0;
  display: none;
}
```

### HTML Structure

```html
<div class="notebook-layout">
  <!-- Left Sidebar -->
  <div class="left-sidebar">
    <div class="sidebar-header">
      <h2>PrismNote</h2>
      <button class="toggle-sidebar">≡</button>
    </div>
    <div class="notebook-list">...</div>
    <div class="collaborators">...</div>
  </div>

  <!-- Central Editor Area -->
  <main class="editor-panel">
    <div class="toolbar">
      <input placeholder="Notebook Name" />
      <button>Save</button>
      <button>Share</button>
    </div>

    <!-- Cells Container -->
    <div class="cells-container">
      <!-- Each cell -->
      <div class="cell">
        <div class="cell-header">
          <span class="cell-number">In [1]:</span>
          <button class="run-button">▶</button>
        </div>
        <div class="cell-editor">
          <!-- Monaco Editor -->
        </div>
        <div class="cell-output">
          <!-- Output here -->
        </div>
      </div>
    </div>
  </main>

  <!-- Right Panel -->
  <aside class="right-panel">
    <div class="panel-tabs">
      <button>Variables</button>
      <button>History</button>
      <button>Comments</button>
    </div>
    <div class="panel-content">...</div>
  </aside>
</div>
```

---

## Component Organization

### Left Sidebar (Collapsible)
- Notebook list
- Recent notebooks
- Collaborators
- Cloud storage mounts
- External connections status
- **Toggle button** to collapse/expand

### Central Editor (Primary)
- Toolbar (notebook name, save, share, settings)
- Cells container
- Individual cells with code + output
- Add cell buttons (+)
- Execution controls
- **Maximum width allocation**

### Right Panel (Collapsible)
- Variable inspector
- Execution history
- Comments/annotations
- AI assistance
- Settings
- **Toggle button** to collapse/expand

---

## Implementation Steps

### Step 1: Update App Layout (1 hour)
```tsx
// App.tsx
function App() {
  const [sidebarOpen, setSidebarOpen] = useState(true)
  const [rightPanelOpen, setRightPanelOpen] = useState(true)

  return (
    <div className="notebook-layout">
      {sidebarOpen && <Sidebar />}
      <Editor /> {/* Main focus */}
      {rightPanelOpen && <RightPanel />}
    </div>
  )
}
```

### Step 2: Update CSS Grid (1.5 hours)
- Implement responsive grid layout
- Add collapse/expand animations
- Set proper breakpoints for mobile
- Ensure proper spacing and gaps

### Step 3: Update Components (2 hours)
- Update Sidebar to support collapse
- Update RightPanel to support collapse
- Add toggle buttons with icons
- Update cell styling for full width

### Step 4: Testing & Polish (1.5 hours)
- Test collapse/expand functionality
- Verify code editor has sufficient space
- Check responsive behavior on different screens
- Visual refinements

---

## Responsive Breakpoints

### Desktop (> 1440px)
```
Sidebar: 240px | Editor: 1fr | Panel: 280px
All panels visible by default
```

### Tablet (768px - 1440px)
```
Sidebar: 64px (collapsed) | Editor: 1fr | Panel: 280px
Sidebar icons only
Right panel visible but narrower
```

### Mobile (< 768px)
```
Full width editor
Sidebar: Drawer (toggleable)
Right panel: Bottom sheet (toggleable)
Stacked layout
```

---

## CSS Variables Needed

```css
:root {
  /* Layout */
  --sidebar-width: 240px;
  --sidebar-width-collapsed: 64px;
  --right-panel-width: 280px;
  --editor-min-width: 600px;
  
  /* Breakpoints */
  --break-mobile: 768px;
  --break-tablet: 1024px;
  --break-desktop: 1440px;
  
  /* Transitions */
  --sidebar-transition: all 300ms ease;
  --panel-transition: all 300ms ease;
}
```

---

## Key Improvements

### For User Experience
1. **Code visibility** - More horizontal space
2. **Workflow efficiency** - Central focus on editor
3. **Less scrolling** - Output below code
4. **Collapsible sidebars** - Reclaim space when needed
5. **Professional feel** - Similar to VSCode, PyCharm layout

### For Code Quality
1. **Longer lines visible** - No unnecessary wrapping
2. **Better syntax highlighting** - More space
3. **Easier debugging** - Output in context
4. **Better for complex code** - Full width available

### Accessibility
1. **Better for users with vision needs** - Larger code area
2. **Touch-friendly** - Wider buttons and inputs
3. **Mobile-friendly** - Responsive layout

---

## Mobile Considerations

### Mobile Layout
```
┌──────────────────────┐
│ ☰ Notebook Name  ⋮ │  Header
├──────────────────────┤
│                      │
│   CODE EDITOR        │  Full width
│   (Swipe left/right) │  Primary
│                      │
├──────────────────────┤
│     OUTPUT           │
│  (Swipe up/down)     │  Secondary
└──────────────────────┘

Sidebar: Hamburger menu (drawer)
Right panel: Bottom drawer or hidden
```

### Touch Interactions
- Swipe left: Open right panel
- Swipe right: Open left sidebar
- Swipe up: Expand output
- Swipe down: Collapse output

---

## Comparison with Competitors

### Deepnote
- Central editor with tabs
- Right panel for variables
- Left sidebar for navigation
- **≈ Proposed layout**

### Google Colab
- Full-width cells
- Linear flow (top to bottom)
- Minimal sidebars
- **Full central focus**

### Jupyter Lab
- Central editor
- Collapsible sidebars
- File browser left
- Properties right
- **Similar to proposed**

---

## Migration Path

### Phase 1: Implement Grid (Week 1)
- Update CSS layout
- Add toggle buttons
- Test responsive behavior

### Phase 2: Update Components (Week 2)
- Refactor Sidebar
- Refactor RightPanel
- Add collapse animations

### Phase 3: Polish & Testing (Week 3)
- Visual refinements
- Cross-browser testing
- Mobile testing
- Performance optimization

---

## Metrics to Track

- Code editor horizontal space (pixels)
- Lines visible without horizontal scroll
- Sidebar usage (collapsed vs expanded)
- Right panel usage
- Mobile viewport distribution
- User feedback on layout preference

---

## Before & After

### Current (Poor)
- Code editor: ~50% width
- Limited to ~80-90 characters per line
- Sidebars dominate
- Output cramped

### After (Optimal)
- Code editor: ~65% width
- Visible: ~120-140 characters per line
- Sidebars secondary
- Output full below
- Professional workspace feel

---

## Rollout Plan

1. **Feature flag** - Behind feature flag initially
2. **Beta group** - Internal testing
3. **Gradual rollout** - 25% → 50% → 100%
4. **Fallback option** - Keep old layout toggle for users who prefer it
5. **Gather feedback** - Metrics and user surveys

---

## Success Criteria

- [ ] Code editor takes 60-70% of horizontal space
- [ ] Minimum 120 characters visible without scroll
- [ ] Sidebars collapse smoothly
- [ ] Right panel collapses smoothly
- [ ] Mobile layout works on all breakpoints
- [ ] Accessibility maintained (WCAG AA)
- [ ] Performance: < 16ms transition time
- [ ] User satisfaction > 85% in survey

---

This restructuring will make PrismNote's code editor the primary focus, aligning with user expectations and professional development tool standards.
