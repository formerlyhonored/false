---
name: minimalist-dark-ui
description: >-
  Enforces zero-bloat UI layouts, crisp typography, disciplined dark palettes,
  and precise CSS variable scoping. Use when designing, styling, or refactoring
  user interfaces, components, stylesheets, or design systems.
---

# Minimalist Dark UI Skill

Enforces high-density, zero-bloat, distraction-free dark user interfaces. Prioritizes information hierarchy, crisp readability, tactile subtle borders, and strictly scoped design tokens over visual clutter and heavy decorative effects.

---

## Design Principles

### 1. Zero-Bloat Layout & Hierarchy
- **Functional Density**: Maximize meaningful content per pixel while providing deliberate breathing room. Eliminate superfluous containers, divider wrappers, and nested wrapper divs.
- **Structural Borders Over Shadows**: In dark mode, drop shadows are often muddy or invisible. Use crisp 1px borders with controlled opacity (8%–15% white on dark) to define card boundaries and panel splits.
- **Content-Driven Layout**: Use CSS Grid and Flexbox with explicit alignment. Avoid hardcoded heights and magic number margins.

### 2. Dark Palette Architecture
Never use pure pitch black (`#000000`) for content surfaces—it creates harsh ocular contrast and halos. Use layered neutral slate/zinc tones:

| Token Name | Default Value (Hex/HSL) | Purpose |
| :--- | :--- | :--- |
| `--bg-canvas` | `#09090b` (hsl 240, 10%, 3.9%) | App/window background level 0 |
| `--bg-surface` | `#121215` (hsl 240, 10%, 7%) | Base container / sidebars level 1 |
| `--bg-elevated` | `#18181b` (hsl 240, 5%, 10%) | Cards, dropdown menus, modals level 2 |
| `--bg-hover` | `#27272a` (hsl 240, 5%, 16%) | Interactive hover states |
| `--border-subtle` | `rgba(255, 255, 255, 0.08)` | Structural split lines, subtle panels |
| `--border-strong` | `rgba(255, 255, 255, 0.16)` | Inputs, active tabs, focused elements |
| `--text-primary` | `#f4f4f5` (hsl 240, 5%, 96%) | High-contrast headings and primary copy |
| `--text-secondary` | `#a1a1aa` (hsl 240, 4%, 65%) | Secondary labels, descriptions, metadata |
| `--text-muted` | `#71717a` (hsl 240, 5%, 46%) | Disabled text, placeholders, timestamps |
| `--accent-primary` | `#3b82f6` or `#06b6d4` | Restrained functional action, focus rings |

### 3. Crisp Typography Standards
- **Font Stack**:
  ```css
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Inter", "Helvetica Neue", sans-serif;
  ```
- **Monospace Stack** (for code, metrics, tables, data):
  ```css
  font-family: "JetBrains Mono", "SF Mono", "Fira Code", Menlo, Consolas, monospace;
  font-feature-settings: "tnum" 1; /* Tabular numbers prevent jitter in dynamic numbers */
  ```
- **Readability Rules**:
  - Always enable font smoothing:
    ```css
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    ```
  - Maintain contrast ratios exceeding 7:1 for body copy and 4.5:1 for secondary labels (WCAG AAA/AA).
  - Use tighter letter spacing (`letter-spacing: -0.015em`) on large headings and normal spacing on small body copy.

### 4. Precise CSS Variable Scoping
- Define base design tokens at `:root` or the theme root container:
  ```css
  :root {
    color-scheme: dark;
    --radius-sm: 4px;
    --radius-md: 6px;
    --radius-lg: 8px;
    --transition-fast: 120ms cubic-bezier(0.16, 1, 0.3, 1);
  }
  ```
- Component-specific overrides must be scoped locally within the component class or module:
  ```css
  /* Good: Component-scoped token overrides */
  .data-grid {
    --grid-row-height: 32px;
    --grid-border: var(--border-subtle);
  }
  ```
- Avoid global utility conflicts by using CSS Modules (`.module.css`), BEM naming, or component-encapsulated styles.

---

## UI Component Checklist

Before shipping any UI component:
- [ ] No un-styled raw HTML buttons or inputs.
- [ ] Active and focused states have distinct, visible keyboard focus indicators (`outline: 2px solid var(--accent-primary); outline-offset: 2px`).
- [ ] Interactive elements provide instant feedback via smooth, low-latency micro-transitions ($\le$150ms).
- [ ] Respects `prefers-reduced-motion`:
  ```css
  @media (prefers-reduced-motion: reduce) {
    *, *::before, *::after {
      animation-duration: 0.01ms !important;
      transition-duration: 0.01ms !important;
    }
  }
  ```
- [ ] Scrollbars are styled dark and subtle (no native white scrollbar gutters):
  ```css
  scrollbar-width: thin;
  scrollbar-color: var(--bg-hover) transparent;
  ```
