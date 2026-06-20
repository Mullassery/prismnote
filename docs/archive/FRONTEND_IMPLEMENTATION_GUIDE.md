# PrismNote Frontend Implementation Guide

**Based on:** UI/UX Design System v1.0  
**Framework:** React 18 + TypeScript + Vite  
**Styling:** Tailwind CSS + Custom CSS

---

## 1. Project Structure

```
frontend/
├── src/
│   ├── components/
│   │   ├── common/           (Reusable components)
│   │   │   ├── Button.tsx
│   │   │   ├── Input.tsx
│   │   │   ├── Modal.tsx
│   │   │   ├── Card.tsx
│   │   │   ├── Navigation.tsx
│   │   │   └── ...
│   │   ├── notebook/         (Notebook-specific)
│   │   │   ├── Editor.tsx
│   │   │   ├── Cell.tsx
│   │   │   ├── Output.tsx
│   │   │   └── ...
│   │   ├── collaboration/    (Real-time features)
│   │   │   ├── Presence.tsx
│   │   │   ├── Comments.tsx
│   │   │   └── ...
│   │   └── docker/           (Container features)
│   │       ├── ContainerList.tsx
│   │       ├── ExecutorPanel.tsx
│   │       └── ...
│   ├── hooks/
│   │   ├── useNotebook.ts
│   │   ├── useCollaboration.ts
│   │   ├── useDocker.ts
│   │   ├── useAccessibility.ts
│   │   └── ...
│   ├── styles/
│   │   ├── globals.css
│   │   ├── components.css
│   │   ├── accessibility.css
│   │   ├── animations.css
│   │   └── dark-mode.css
│   ├── lib/
│   │   ├── colors.ts          (Design tokens)
│   │   ├── spacing.ts
│   │   ├── typography.ts
│   │   └── accessibility.ts
│   ├── utils/
│   │   ├── validation.ts
│   │   ├── formatting.ts
│   │   └── analytics.ts
│   └── main.tsx
├── tailwind.config.js        (Design tokens config)
└── tsconfig.json
```

---

## 2. Design Tokens Configuration

### Tailwind Configuration

```js
// tailwind.config.js
module.exports = {
  content: ['./src/**/*.{ts,tsx}'],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        primary: '#2563EB',
        success: '#10B981',
        warning: '#F59E0B',
        error: '#EF4444',
        info: '#3B82F6',
      },
      spacing: {
        xs: '4px',
        sm: '8px',
        md: '16px',
        lg: '24px',
        xl: '32px',
        '2xl': '48px',
        '3xl': '64px',
      },
      typography: {
        DEFAULT: {
          css: {
            lineHeight: '1.6',
            letterSpacing: '0.3px',
          },
        },
      },
      borderRadius: {
        sm: '4px',
        md: '6px',
        lg: '8px',
        xl: '12px',
      },
      boxShadow: {
        sm: '0 1px 2px rgba(0,0,0,0.05)',
        md: '0 4px 6px rgba(0,0,0,0.1)',
        lg: '0 20px 25px rgba(0,0,0,0.15)',
      },
      transitionDuration: {
        fast: '100ms',
        standard: '200ms',
        slow: '300ms',
      },
    },
  },
};
```

---

## 3. Component Implementation Examples

### Button Component

```tsx
// Button.tsx
import React from 'react';
import classNames from 'classnames';

interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'tertiary';
  size?: 'sm' | 'md' | 'lg';
  isLoading?: boolean;
  isDisabled?: boolean;
  ariaLabel?: string;
}

export const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  (
    {
      variant = 'primary',
      size = 'md',
      isLoading = false,
      isDisabled = false,
      className,
      children,
      ariaLabel,
      ...props
    },
    ref
  ) => {
    const baseStyles =
      'font-medium rounded transition-all duration-standard focus:outline-none focus:ring-2 focus:ring-offset-2';

    const variants = {
      primary: 'bg-primary text-white hover:bg-blue-700 active:bg-blue-900 focus:ring-primary',
      secondary:
        'bg-gray-200 text-gray-900 hover:bg-gray-300 active:bg-gray-400 dark:bg-gray-700 dark:text-white',
      tertiary: 'text-primary hover:underline active:text-blue-900',
    };

    const sizes = {
      sm: 'px-3 py-1 text-sm h-8',
      md: 'px-4 py-2 text-md h-10',
      lg: 'px-6 py-3 text-lg h-12',
    };

    const disabledStyles = isDisabled ? 'opacity-50 cursor-not-allowed' : '';
    const loadingStyles = isLoading ? 'opacity-75 cursor-wait' : '';

    return (
      <button
        ref={ref}
        className={classNames(
          baseStyles,
          variants[variant],
          sizes[size],
          disabledStyles,
          loadingStyles,
          className
        )}
        disabled={isDisabled || isLoading}
        aria-label={ariaLabel}
        {...props}
      >
        {isLoading && <span className="mr-2">⏳</span>}
        {children}
      </button>
    );
  }
);

Button.displayName = 'Button';
```

### Input Component

```tsx
// Input.tsx
import React from 'react';
import classNames from 'classnames';

interface InputProps
  extends React.InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  errorMessage?: string;
  helperText?: string;
  isRequired?: boolean;
}

export const Input = React.forwardRef<HTMLInputElement, InputProps>(
  (
    {
      label,
      errorMessage,
      helperText,
      isRequired = false,
      id,
      className,
      ...props
    },
    ref
  ) => {
    const inputId = id || `input-${Math.random()}`;

    return (
      <div className="form-group mb-6">
        {label && (
          <label htmlFor={inputId} className="block text-sm font-medium mb-2">
            {label}
            {isRequired && <span className="text-error ml-1">*</span>}
          </label>
        )}

        <input
          ref={ref}
          id={inputId}
          className={classNames(
            'w-full px-3 py-2 border rounded-md',
            'text-sm font-400 transition-colors duration-fast',
            'focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary',
            'dark:bg-gray-700 dark:border-gray-600 dark:text-white',
            errorMessage ? 'border-error' : 'border-gray-300',
            className
          )}
          aria-invalid={!!errorMessage}
          aria-describedby={errorMessage ? `${inputId}-error` : undefined}
          {...props}
        />

        {errorMessage && (
          <span
            id={`${inputId}-error`}
            className="block text-xs text-error mt-1"
            role="alert"
          >
            {errorMessage}
          </span>
        )}

        {helperText && (
          <span className="block text-xs text-gray-500 mt-1">
            {helperText}
          </span>
        )}
      </div>
    );
  }
);

Input.displayName = 'Input';
```

### Modal Component

```tsx
// Modal.tsx
import React, { useEffect } from 'react';

interface ModalProps {
  isOpen: boolean;
  title: string;
  children: React.ReactNode;
  onClose: () => void;
  actions?: React.ReactNode;
  size?: 'sm' | 'md' | 'lg';
}

export const Modal: React.FC<ModalProps> = ({
  isOpen,
  title,
  children,
  onClose,
  actions,
  size = 'md',
}) => {
  useEffect(() => {
    if (isOpen) {
      document.body.style.overflow = 'hidden';
      return () => {
        document.body.style.overflow = 'unset';
      };
    }
  }, [isOpen]);

  if (!isOpen) return null;

  const sizeClasses = {
    sm: 'max-w-sm',
    md: 'max-w-md',
    lg: 'max-w-lg',
  };

  return (
    <>
      <div
        className="fixed inset-0 bg-black bg-opacity-50 z-40"
        onClick={onClose}
        aria-hidden="true"
      />
      <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
        <div
          className={`bg-white dark:bg-gray-800 rounded-xl shadow-lg ${sizeClasses[size]} w-full`}
          role="dialog"
          aria-modal="true"
          aria-labelledby="modal-title"
        >
          <div className="flex items-center justify-between p-6 border-b">
            <h2 id="modal-title" className="text-xl font-bold">
              {title}
            </h2>
            <button
              onClick={onClose}
              className="text-gray-500 hover:text-gray-700 focus:outline-none focus:ring-2"
              aria-label="Close dialog"
            >
              ✕
            </button>
          </div>

          <div className="p-6">{children}</div>

          {actions && (
            <div className="flex justify-end gap-3 p-6 border-t bg-gray-50 dark:bg-gray-700">
              {actions}
            </div>
          )}
        </div>
      </div>
    </>
  );
};
```

---

## 4. Accessibility Checklist

### Component Accessibility

```tsx
// useAccessibility.ts
import { useEffect, useRef } from 'react';

export const useAccessibility = () => {
  const elementRef = useRef<HTMLElement>(null);

  useEffect(() => {
    // Ensure focus management
    if (elementRef.current?.matches('[role="dialog"]')) {
      elementRef.current.focus();
    }
  }, []);

  const announceMessage = (message: string, priority: 'polite' | 'assertive' = 'polite') => {
    const announcement = document.createElement('div');
    announcement.setAttribute('aria-live', priority);
    announcement.setAttribute('aria-atomic', 'true');
    announcement.className = 'sr-only'; // Screen reader only
    announcement.textContent = message;
    document.body.appendChild(announcement);

    setTimeout(() => announcement.remove(), 1000);
  };

  return { elementRef, announceMessage };
};
```

### Accessibility CSS

```css
/* accessibility.css */

/* Hide from visual users but keep for screen readers */
.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border-width: 0;
}

/* Respect user's motion preferences */
@media (prefers-reduced-motion: reduce) {
  * {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}

/* Ensure focus is always visible */
:focus-visible {
  outline: 2px solid #2563EB;
  outline-offset: 2px;
}

/* High contrast mode support */
@media (prefers-contrast: more) {
  button {
    border: 1px solid currentColor;
  }
  input {
    border-width: 2px;
  }
}

/* Forced colors mode (Windows High Contrast) */
@media (forced-colors: active) {
  button {
    border: 1px solid ButtonBorder;
    forced-color-adjust: none;
  }
}
```

---

## 5. Dark Mode Implementation

```tsx
// useTheme.ts
import { useEffect, useState } from 'react';

export const useTheme = () => {
  const [theme, setTheme] = useState<'light' | 'dark'>(() => {
    // Check user preference from localStorage
    const saved = localStorage.getItem('theme');
    if (saved) return saved as 'light' | 'dark';

    // Check system preference
    return window.matchMedia('(prefers-color-scheme: dark)').matches
      ? 'dark'
      : 'light';
  });

  useEffect(() => {
    const root = document.documentElement;
    root.classList.toggle('dark', theme === 'dark');
    localStorage.setItem('theme', theme);
  }, [theme]);

  const toggleTheme = () => {
    setTheme(prev => (prev === 'light' ? 'dark' : 'light'));
  };

  return { theme, toggleTheme };
};
```

```css
/* Dark mode support */
@media (prefers-color-scheme: dark) {
  :root {
    --bg-primary: #1f2937;
    --bg-secondary: #111827;
    --text-primary: #f9fafb;
    --text-secondary: #d1d5db;
  }
}

.dark {
  --bg-primary: #1f2937;
  --bg-secondary: #111827;
  --text-primary: #f9fafb;
  --text-secondary: #d1d5db;
}
```

---

## 6. Performance Optimization

### Code Splitting

```tsx
// Routes with lazy loading
import { lazy, Suspense } from 'react';

const NotebookEditor = lazy(() => import('./pages/NotebookEditor'));
const Settings = lazy(() => import('./pages/Settings'));
const CloudStorage = lazy(() => import('./pages/CloudStorage'));

export const Routes = () => (
  <Suspense fallback={<LoadingSpinner />}>
    <Switch>
      <Route path="/notebook/:id" component={NotebookEditor} />
      <Route path="/settings" component={Settings} />
      <Route path="/cloud-storage" component={CloudStorage} />
    </Switch>
  </Suspense>
);
```

### Image Optimization

```tsx
// Image component with lazy loading
export const OptimizedImage = ({
  src,
  alt,
  width,
  height,
}: {
  src: string;
  alt: string;
  width: number;
  height: number;
}) => (
  <img
    src={src}
    alt={alt}
    width={width}
    height={height}
    loading="lazy"
    decoding="async"
  />
);
```

### Bundle Analysis

```bash
# Check bundle size
npm run build
npm install -D webpack-bundle-analyzer

# Then analyze
webpack-bundle-analyzer dist/stats.json
```

---

## 7. Testing Strategy

### Unit Tests

```tsx
// Button.test.tsx
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Button } from './Button';

describe('Button', () => {
  it('renders with text', () => {
    render(<Button>Click me</Button>);
    expect(screen.getByText('Click me')).toBeInTheDocument();
  });

  it('handles click events', async () => {
    const handleClick = jest.fn();
    render(<Button onClick={handleClick}>Click</Button>);
    await userEvent.click(screen.getByText('Click'));
    expect(handleClick).toHaveBeenCalled();
  });

  it('is accessible with aria-label', () => {
    render(<Button ariaLabel="Save notebook">Save</Button>);
    expect(screen.getByLabelText('Save notebook')).toBeInTheDocument();
  });

  it('disables when disabled prop is true', () => {
    render(<Button disabled>Click</Button>);
    expect(screen.getByText('Click')).toBeDisabled();
  });
});
```

### Accessibility Testing

```tsx
// axe integration
import { axe, toHaveNoViolations } from 'jest-axe';

expect.extend(toHaveNoViolations);

it('Button has no accessibility violations', async () => {
  const { container } = render(<Button>Test</Button>);
  const results = await axe(container);
  expect(results).toHaveNoViolations();
});
```

---

## 8. Development Workflow

### Environment Setup

```bash
# Install dependencies
npm install

# Start dev server
npm run dev

# Build for production
npm run build

# Preview production build
npm run preview

# Run tests
npm test

# Lint code
npm run lint

# Format code
npm run format
```

### Git Workflow

```bash
# Create feature branch
git checkout -b feature/ui-improvement

# Make changes, test, commit
git add .
git commit -m "chore: improve button accessibility"

# Push and create PR
git push origin feature/ui-improvement
```

---

## 9. Performance Targets

```
Lighthouse Scores:
├── Performance:  ≥ 90
├── Accessibility: ≥ 95
├── Best Practices: ≥ 95
└── SEO: ≥ 95

Load Metrics:
├── FCP (First Contentful Paint): < 1.5s
├── LCP (Largest Contentful Paint): < 2.5s
├── CLS (Cumulative Layout Shift): < 0.1
└── FID (First Input Delay): < 100ms
```

---

## 10. Continuous Improvement

### User Feedback Loop
1. **Collect**: Analytics, user surveys, bug reports
2. **Analyze**: Identify patterns and pain points
3. **Implement**: Update UI/UX based on feedback
4. **Measure**: Track metrics, repeat

### Design System Updates
- Monthly: Component review
- Quarterly: Accessibility audit
- Bi-annually: Performance review
- Annually: Major refresh

---

This implementation guide ensures all PrismNote frontend components meet the highest standards for accessibility, performance, and user experience.

