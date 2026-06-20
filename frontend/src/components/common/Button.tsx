import React from 'react'
import classNames from 'classnames'

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'tertiary'
  size?: 'sm' | 'md' | 'lg'
  isLoading?: boolean
  ariaLabel?: string
}

export const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  (
    {
      variant = 'primary',
      size = 'md',
      isLoading = false,
      className,
      children,
      disabled,
      ariaLabel,
      ...props
    },
    ref
  ) => {
    const baseStyles = classNames(
      'btn',
      {
        'btn-primary': variant === 'primary',
        'btn-secondary': variant === 'secondary',
        'btn-tertiary': variant === 'tertiary',
        'btn-sm': size === 'sm',
        'btn-md': size === 'md',
        'btn-lg': size === 'lg',
      },
      className
    )

    return (
      <button
        ref={ref}
        className={baseStyles}
        disabled={disabled || isLoading}
        aria-label={ariaLabel}
        {...props}
      >
        {isLoading && (
          <span className="animate-spin-slow" aria-hidden="true">
            ⟳
          </span>
        )}
        {children}
      </button>
    )
  }
)

Button.displayName = 'Button'
