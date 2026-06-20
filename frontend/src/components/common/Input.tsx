import React from 'react'
import classNames from 'classnames'

interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  label?: string
  errorMessage?: string
  helperText?: string
  isRequired?: boolean
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
    const inputId = id || `input-${Math.random()}`

    const inputClasses = classNames(
      'form-input',
      {
        'error': !!errorMessage,
      },
      className
    )

    return (
      <div className="form-group">
        {label && (
          <label htmlFor={inputId} className="form-label">
            {label}
            {isRequired && <span className="text-error ml-1">*</span>}
          </label>
        )}

        <input
          ref={ref}
          id={inputId}
          className={inputClasses}
          aria-invalid={!!errorMessage}
          aria-describedby={errorMessage ? `${inputId}-error` : undefined}
          {...props}
        />

        {errorMessage && (
          <span
            id={`${inputId}-error`}
            className="form-error"
            role="alert"
          >
            {errorMessage}
          </span>
        )}

        {helperText && (
          <span className="form-helper">
            {helperText}
          </span>
        )}
      </div>
    )
  }
)

Input.displayName = 'Input'
