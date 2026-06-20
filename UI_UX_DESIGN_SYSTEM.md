# PrismNote UI/UX Design System

**Version:** 1.0  
**Date:** 2026-06-20  
**Based on:** Google Material Design 3, Apple HIG, Nielsen's Usability Heuristics, WCAG 2.1

---

## 1. Design Principles

### 1.1 Core Principles
- **User-Centered**: Design decisions driven by user research and testing
- **Clear & Intuitive**: Minimize cognitive load, obvious affordances
- **Accessible**: WCAG 2.1 AA compliance minimum, inclusive design
- **Consistent**: Unified patterns across all interfaces
- **Responsive**: Works seamlessly on mobile, tablet, desktop
- **Fast**: Sub-100ms interactions, optimized performance
- **Beautiful**: Modern aesthetics with purposeful design

### 1.2 Design Values
1. **Clarity over Clever** - Simple, clear communication beats impressive but confusing design
2. **Progressive Disclosure** - Show essentials first, advanced options on demand
3. **Direct Manipulation** - Users work with objects directly, not through dialogs
4. **Feedback** - Every action receives immediate visual/auditory feedback
5. **Error Prevention** - Design prevents problems before they occur
6. **Consistency** - Patterns repeat, users learn faster

---

## 2. Color System

### 2.1 Primary Colors
```
Primary Blue:     #2563EB (accent, CTAs, focus states)
Surface:          #FFFFFF (light) / #1F2937 (dark)
Background:       #F9FAFB (light) / #111827 (dark)
Border:           #E5E7EB (light) / #374151 (dark)
Text Primary:     #1F2937 (light) / #F9FAFB (dark)
Text Secondary:   #6B7280 (light) / #D1D5DB (dark)
```

### 2.2 Semantic Colors
- **Success**: #10B981 (green for positive actions)
- **Warning**: #F59E0B (amber for caution)
- **Error**: #EF4444 (red for destructive/errors)
- **Info**: #3B82F6 (blue for information)

### 2.3 Color Accessibility
- Minimum contrast ratio: 4.5:1 for text
- 3:1 for UI components
- No color as sole indicator (always include icon or text)
- Color-blind safe palette (deuteranopia, protanopia, tritanopia)

---

## 3. Typography System

### 3.1 Font Family
- **Code**: Monaco, Menlo, SF Mono, Courier New (fallback)
- **UI**: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif
- **Font sizes**: 10px, 11px, 12px, 13px, 14px, 15px, 16px, 18px, 20px, 24px, 32px

### 3.2 Text Styles

**Display** (32px, bold)
```
Headlines, page titles
Line height: 1.2
Letter spacing: -0.5px
```

**Heading 1** (24px, bold)
```
Section titles
Line height: 1.3
Letter spacing: -0.25px
```

**Heading 2** (20px, 600)
```
Subsection titles
Line height: 1.4
Letter spacing: 0
```

**Body** (14px, 400)
```
Default text, descriptions
Line height: 1.6
Letter spacing: 0.3px
```

**Caption** (12px, 400)
```
Helper text, metadata
Line height: 1.5
Letter spacing: 0.4px
Color: secondary (6B7280 / D1D5DB)
```

**Mono** (13px, 400)
```
Code, values, technical text
Font-family: Monaco/Menlo/SF Mono
Line height: 1.6
```

### 3.3 Font Weights
- 400: Regular (default)
- 500: Medium (emphasis)
- 600: Semibold (strong emphasis)
- 700: Bold (headings)

---

## 4. Spacing & Layout

### 4.1 Spacing Scale
```
2px   (0.125rem)  - Minimal gaps
4px   (0.25rem)   - Tight spacing
8px   (0.5rem)    - Standard padding
12px  (0.75rem)   - Comfortable spacing
16px  (1rem)      - Standard margins
24px  (1.5rem)    - Section spacing
32px  (2rem)      - Large sections
48px  (3rem)      - Major sections
64px  (4rem)      - Page top spacing
```

### 4.2 Grid System
- **Base**: 4px grid (all dimensions multiples of 4)
- **Columns**: 12 column grid for responsive layouts
- **Breakpoints**:
  - Mobile: 320px - 640px (1-2 columns)
  - Tablet: 641px - 1024px (2-4 columns)
  - Desktop: 1025px+ (4-6 columns)

### 4.3 Container Sizes
- **Max width**: 1440px (large screens)
- **Sidebar**: 240px (default), 64px (collapsed)
- **Card**: 280px - 400px
- **Modal**: 480px - 640px

---

## 5. Component Design

### 5.1 Buttons

**Primary Button**
```
Background: #2563EB
Color: White
Padding: 8px 16px
Border-radius: 6px
Font weight: 500
Min-height: 40px (touch-friendly)

Hover: #1D4ED8 (darker)
Active: #1E40AF (even darker)
Disabled: #E5E7EB, text #9CA3AF
Focus: Blue outline (4px), offset 2px
```

**Secondary Button**
```
Background: #E5E7EB (light) / #374151 (dark)
Color: Primary text
Border: 1px solid #D1D5DB
Padding: 8px 16px
```

**Tertiary Button**
```
Background: Transparent
Color: #2563EB
Border: None
Underline on hover
```

**Icon Button**
```
Size: 40x40px minimum
Padding: 8px
Circular or square
Hover background: 10% primary color
```

### 5.2 Input Fields

**Text Input**
```
Height: 40px
Padding: 8px 12px
Border: 1px solid #D1D5DB
Border-radius: 6px
Font: 14px, 400
Placeholder: secondary text color

Focus: Blue border (2px), shadow
Hover: Border #9CA3AF
Error: Red border, error text below
```

**Label**
```
Font: 14px, 500
Color: Primary text
Margin-bottom: 4px
Required indicator: red asterisk
```

**Helper Text**
```
Font: 12px, 400
Color: Secondary text
Margin-top: 4px
```

### 5.3 Cards

**Standard Card**
```
Background: White (light) / #1F2937 (dark)
Border: 1px solid #E5E7EB / #374151
Border-radius: 8px
Padding: 16px
Shadow: 0 1px 3px rgba(0,0,0,0.1)

Hover: Elevated shadow
Active: Border highlight
```

### 5.4 Navigation

**Sidebar Navigation**
```
Width: 240px (default), 64px (collapsed)
Items: 40px height, 12px vertical padding
Icons: 20px, centered
Text: 14px, 400, left-aligned

Active: Blue background (10%), text bold
Hover: 5% background color
Focus: Outline ring

Transition: 200ms smooth
```

**Breadcrumbs**
```
Font: 12px, 400
Separator: "/"
Color: Secondary text

Active: Primary text, bold
Hover: Underline
```

### 5.5 Modals & Dialogs

**Modal**
```
Background: White (light) / #1F2937 (dark)
Border-radius: 12px
Shadow: 0 20px 25px rgba(0,0,0,0.15)
Max-width: 640px
Padding: 24px

Header: 20px bold, 24px margin-bottom
Content: 14px, 400, line-height 1.6
Footer: Right-aligned buttons, margin-top 24px

Close button: Top-right, 32x32px
Overlay: rgba(0,0,0,0.5), dismissible
```

---

## 6. Micro-interactions

### 6.1 Transitions
```
Quick interactions: 100ms
Standard transitions: 200ms
Slow transitions: 300ms
Easing: cubic-bezier(0.4, 0, 0.2, 1) (Material standard)
```

### 6.2 Feedback States
- **Hover**: 50ms fade-in color change
- **Active**: Immediate visual feedback
- **Disabled**: 100ms fade to disabled state
- **Loading**: Smooth spinner rotation (1.2s)
- **Success**: Green checkmark pulse (300ms)
- **Error**: Red shake animation (200ms)

### 6.3 Animations
```
Entrance: Fade-in + subtle scale (200ms)
Exit: Fade-out (150ms)
Notification: Slide-in from top (200ms)
Toast: Auto-dismiss after 4000ms
```

---

## 7. Accessibility (WCAG 2.1 AA)

### 7.1 Keyboard Navigation
- All interactive elements keyboard-accessible (Tab order)
- Focus visible at all times (minimum 2px outline)
- Logical tab order (left-to-right, top-to-bottom)
- Escape key closes modals and dropdowns
- Enter/Space triggers buttons and toggles
- Arrow keys navigate lists and menus

### 7.2 Screen Reader Support
```html
<!-- Proper ARIA labels -->
<button aria-label="Close dialog">✕</button>

<!-- Region landmarks -->
<nav aria-label="Main navigation">...</nav>
<main aria-label="Content">...</main>

<!-- Form associations -->
<label for="email">Email address</label>
<input id="email" type="email" />

<!-- Live regions for updates -->
<div aria-live="polite" aria-atomic="true">
  Notebook saved
</div>
```

### 7.3 Color & Contrast
- Text on background: 4.5:1 minimum
- UI components: 3:1 minimum
- Focus indicators: 3:1 with adjacent colors
- Never rely on color alone for information

### 7.4 Motion & Animation
- Respect `prefers-reduced-motion`
```css
@media (prefers-reduced-motion: reduce) {
  * { animation-duration: 0.01ms !important; }
}
```

### 7.5 Form Accessibility
- All inputs have associated labels
- Error messages linked to inputs (aria-describedby)
- Required fields clearly marked
- Form validation messages clear and timely
- Success feedback provided

---

## 8. Responsive Design

### 8.1 Mobile-First Approach
```css
/* Base: Mobile (320px) */
.container { width: 100%; }

/* Tablet (641px) */
@media (min-width: 641px) {
  .container { width: 90%; }
}

/* Desktop (1025px) */
@media (min-width: 1025px) {
  .container { max-width: 1200px; }
}
```

### 8.2 Touch-Friendly Design
- Minimum touch target: 44x44px
- Spacing between targets: 8px minimum
- Avoid hover-only interactions
- Large enough text (14px+ minimum)
- Adequate padding around text

### 8.3 Performance
- First Contentful Paint: < 1.5s
- Largest Contentful Paint: < 2.5s
- Cumulative Layout Shift: < 0.1
- Interaction to Paint: < 100ms

---

## 9. Dark Mode

### 9.1 Implementation
```css
/* Automatic detection */
@media (prefers-color-scheme: dark) {
  :root {
    --bg-primary: #1F2937;
    --bg-secondary: #111827;
    --text-primary: #F9FAFB;
    --text-secondary: #D1D5DB;
  }
}

/* User preference override */
[data-theme="dark"] { ... }
[data-theme="light"] { ... }
```

### 9.2 Color Adjustments
- Increase contrast in dark mode
- Reduce brightness of colors (use #1E40AF instead of #2563EB)
- Use true blacks sparingly (prefer #1F2937)
- Subtle borders in dark backgrounds

---

## 10. Error Handling & Validation

### 10.1 Form Validation
```
Real-time feedback (after user leaves field)
Clear error messages (avoid technical jargon)
Suggest corrections
Mark required fields
Validate on blur, not keystroke (except length)
```

### 10.2 Error Messages
- **Do**: "Email address must contain @"
- **Don't**: "Validation error code E42"
- **Do**: Place inline near field
- **Don't**: Generic alerts far from cause
- **Do**: Use red (#EF4444) consistently
- **Don't**: Only use color

### 10.3 Feedback Messages
```
Success: "Notebook saved" (green, 4s timeout)
Error: "Failed to save" (red, persistent)
Loading: Spinner with "Saving..." text
Warning: Amber icon + text before destructive action
```

---

## 11. Data Visualization

### 11.1 Chart Design
- Clean, simple axes
- Limit to 5-7 colors (colorblind-safe)
- Clear legends with descriptions
- Meaningful gridlines
- No unnecessary 3D or decorations

### 11.2 Tables
```
Header row: Bold, background color
Alternating row colors (every 2nd row, 5% opacity)
Right-aligned numbers
Left-aligned text
Sortable column headers (triangle indicator)
Hover highlight (very subtle, 2% opacity)
```

### 11.3 Data Density
- **Compact**: 32px rows (data-heavy)
- **Standard**: 40px rows (balanced)
- **Spacious**: 48px rows (scanning-focused)

---

## 12. Information Architecture

### 12.1 Navigation Hierarchy
```
Primary Nav (Main sections)
├── Notebooks
├── Cloud Storage
├── Collaboration
├── Settings
└── Help

Secondary Nav (Contextual)
└── Within each section

Tertiary Nav (Details)
└── Breadcrumbs, tabs
```

### 12.2 Wayfinding
- Breadcrumbs on every page
- Current location highlighted
- Consistent menu placement
- Clear section titles
- "Back" button when needed

### 12.3 Search
- Prominent search bar (top)
- Real-time results
- Filter suggestions
- Keyboard shortcut (Cmd+K / Ctrl+K)
- Clear empty state

---

## 13. User Feedback & Testing

### 13.1 Metrics to Track
- **Usability**: Task completion rate, error rate
- **Performance**: Load time, interaction latency
- **Engagement**: Feature adoption, session duration
- **Satisfaction**: NPS, CSAT, SUS score

### 13.2 User Testing
- Conduct with 5-8 users per round
- Test with real notebooks (not wireframes)
- Observe without interrupting
- Record think-aloud protocol
- Iterate based on findings

### 13.3 Accessibility Audit
- Automated testing: axe, Lighthouse
- Manual keyboard navigation
- Screen reader testing (NVDA, JAWS, VoiceOver)
- Color contrast verification
- Motion sensitivity testing

---

## 14. Component Checklist

### Before Shipping Any Component
- [ ] Keyboard accessible
- [ ] Focus visible
- [ ] ARIA labels complete
- [ ] Color contrast ≥ 4.5:1
- [ ] Touch target ≥ 44x44px
- [ ] Works on mobile/tablet/desktop
- [ ] Hover/active/disabled states
- [ ] Loading state (if applicable)
- [ ] Error state (if applicable)
- [ ] Tested with screen reader
- [ ] Animations < 300ms
- [ ] No motion-only interactions
- [ ] Documented in design system

---

## 15. Implementation Examples

### Example 1: Accessible Button
```jsx
<button
  className="btn btn-primary"
  onClick={handleSave}
  aria-label="Save notebook"
  disabled={isSaving}
>
  {isSaving ? 'Saving...' : 'Save'}
</button>

.btn {
  min-height: 40px;
  padding: 8px 16px;
  border-radius: 6px;
  font-weight: 500;
  transition: all 200ms ease;
  cursor: pointer;
  border: none;
  
  &:focus {
    outline: 2px solid #2563EB;
    outline-offset: 2px;
  }
  
  &:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
}
```

### Example 2: Accessible Form
```jsx
<div className="form-group">
  <label htmlFor="notebook-name">
    Notebook name
    <span aria-label="required" className="required">*</span>
  </label>
  <input
    id="notebook-name"
    type="text"
    required
    aria-describedby="name-error"
    onChange={handleChange}
  />
  {error && (
    <span id="name-error" className="error-message" role="alert">
      {error}
    </span>
  )}
</div>

.form-group {
  margin-bottom: 24px;
}

label {
  display: block;
  font-weight: 500;
  margin-bottom: 8px;
  color: #1F2937;
}

input {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid #D1D5DB;
  border-radius: 6px;
  font: inherit;
  
  &:focus {
    border-color: #2563EB;
    outline: none;
    box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.1);
  }
}

.error-message {
  display: block;
  color: #EF4444;
  font-size: 12px;
  margin-top: 4px;
}
```

### Example 3: Accessible Modal
```jsx
<div className="modal-overlay" onClick={onClose}>
  <div
    className="modal"
    role="dialog"
    aria-labelledby="modal-title"
    aria-modal="true"
    onClick={e => e.stopPropagation()}
  >
    <div className="modal-header">
      <h2 id="modal-title">Create Notebook</h2>
      <button
        className="modal-close"
        onClick={onClose}
        aria-label="Close dialog"
      >
        ✕
      </button>
    </div>
    <div className="modal-content">
      {/* Form content */}
    </div>
    <div className="modal-footer">
      <button onClick={onClose}>Cancel</button>
      <button className="btn-primary" onClick={handleCreate}>
        Create
      </button>
    </div>
  </div>
</div>

.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal {
  background: white;
  border-radius: 12px;
  max-width: 640px;
  width: 90%;
  box-shadow: 0 20px 25px rgba(0, 0, 0, 0.15);
}
```

---

## 16. Design Tokens

```json
{
  "colors": {
    "primary": "#2563EB",
    "success": "#10B981",
    "warning": "#F59E0B",
    "error": "#EF4444",
    "text-primary": "#1F2937",
    "text-secondary": "#6B7280"
  },
  "spacing": {
    "xs": "4px",
    "sm": "8px",
    "md": "16px",
    "lg": "24px",
    "xl": "32px"
  },
  "typography": {
    "body": "14px/1.6",
    "caption": "12px/1.5",
    "heading": "20px/1.3 bold"
  },
  "radius": {
    "sm": "4px",
    "md": "6px",
    "lg": "8px",
    "xl": "12px"
  },
  "shadow": {
    "sm": "0 1px 2px rgba(0,0,0,0.05)",
    "md": "0 4px 6px rgba(0,0,0,0.1)",
    "lg": "0 20px 25px rgba(0,0,0,0.15)"
  },
  "transition": {
    "fast": "100ms ease",
    "standard": "200ms ease",
    "slow": "300ms ease"
  }
}
```

---

## 17. Continuous Improvement

### Regular Reviews
- Monthly: User feedback analysis
- Quarterly: Usability testing
- Bi-annually: Accessibility audit
- Annually: Design system refresh

### Design System Maintenance
- Document all decisions with rationale
- Version the design system (keep changelog)
- Train team on latest patterns
- Review and update components quarterly
- Deprecate outdated patterns gradually

---

**This design system ensures PrismNote provides a world-class user experience that is accessible, performant, and beautiful.**

All components should be audited against this system before shipping.

