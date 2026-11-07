---
name: component-scaffolder
description: >-
  Given a single component prompt or specification, scaffolds complete, production-grade
  components including logic, scoped styling, TypeScript interfaces, unit tests, and
  barrel exports according to strict folder structure conventions.
  Use when creating new UI components, widgets, or views.
---

# Component Scaffolder Skill

Standardizes the end-to-end scaffolding of frontend UI components. When given a request to build a new component, it automatically generates a self-contained, typed, styled, accessible, and tested component package in a single coordinated operation.

---

## Canonical Component Directory Structure

Every scaffolded component resides in its own directory named in `PascalCase`:

```text
src/components/<ComponentName>/
├── index.ts                     # Clean barrel export
├── <ComponentName>.tsx          # Component presentation, hooks & logic
├── <ComponentName>.types.ts     # TypeScript interfaces, props, and variants
├── <ComponentName>.module.css   # Scoped CSS Module styling
└── <ComponentName>.test.tsx     # Unit and interaction tests
```

---

## Scaffolding Generation Order & File Standards

When asked to scaffold a component, generate all 5 files sequentially:

### 1. Type Definitions (`<ComponentName>.types.ts`)
Define strict props, variants, states, and accessibility contracts:
- Extend standard HTML attributes when wrapping native elements (e.g., `React.ButtonHTMLAttributes<HTMLButtonElement>`).
- Enforce explicit union types for visual variants and sizes (e.g. `size?: 'sm' | 'md' | 'lg'`).
- Avoid `any` or loose `Record<string, unknown>`.

```typescript
import React from 'react';

export type ComponentVariant = 'default' | 'primary' | 'subtle' | 'danger';
export type ComponentSize = 'sm' | 'md' | 'lg';

export interface ComponentNameProps extends React.HTMLAttributes<HTMLDivElement> {
  variant?: ComponentVariant;
  size?: ComponentSize;
  disabled?: boolean;
  isLoading?: boolean;
  children?: React.ReactNode;
}
```

### 2. Scoped Styling (`<ComponentName>.module.css`)
Follow the `minimalist-dark-ui` conventions:
- Use local class names (`.root`, `.header`, `.content`, `.primary`, `.disabled`).
- Use CSS variables for tokens (`var(--bg-surface)`, `var(--text-primary)`, `var(--border-subtle)`).
- Include focus-visible and active states.

### 3. Component Implementation (`<ComponentName>.tsx`)
- Implement accessible semantic HTML (use `<button>`, `<dialog>`, `<nav>`, `<aside>` where appropriate, or assign explicit ARIA roles).
- Support `ref` forwarding with `React.forwardRef` where applicable.
- Handle loading and disabled states cleanly.

```tsx
import React, { forwardRef } from 'react';
import styles from './ComponentName.module.css';
import { ComponentNameProps } from './ComponentName.types';

export const ComponentName = forwardRef<HTMLDivElement, ComponentNameProps>(
  ({ variant = 'default', size = 'md', disabled = false, children, className, ...rest }, ref) => {
    return (
      <div
        ref={ref}
        className={[
          styles.root,
          styles[variant],
          styles[size],
          disabled ? styles.disabled : '',
          className || '',
        ].filter(Boolean).join(' ')}
        aria-disabled={disabled || undefined}
        {...rest}
      >
        {children}
      </div>
    );
  }
);

ComponentName.displayName = 'ComponentName';
```

### 4. Unit Tests (`<ComponentName>.test.tsx`)
Write tests using standard testing libraries (React Testing Library / Jest / Vitest):
- Test rendering with default props.
- Test variant and size class applications.
- Test user interactions (clicks, keyboard navigation).
- Test disabled/loading states (ensuring callbacks are not invoked).
- Test ARIA attributes and accessibility compliance.

### 5. Barrel Export (`index.ts`)
Provide a single entrypoint for clean imports:
```typescript
export { ComponentName } from './ComponentName';
export type { ComponentNameProps, ComponentVariant, ComponentSize } from './ComponentName.types';
```

---

## Execution Workflow

1. **Parse Requirements**: Extract component name, variants, slots/children, interactive behaviors, and target path.
2. **Generate Files**: Write each of the 5 files into the destination folder.
3. **Verify Build & Tests**:
   Run the type checker and test runner:
   ```powershell
   npx tsc --noEmit
   npm test -- --runInBand <ComponentName>
   ```
4. **Report**: Present the created component structure and verify clean test execution.
