/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{vue,js,ts,jsx,tsx}'],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        brand: 'hsl(var(--brand) / <alpha-value>)',
        'brand-hover': 'hsl(var(--brand-hover) / <alpha-value>)',
        'brand-light': 'hsl(var(--brand-light) / <alpha-value>)',

        'text-primary': 'hsl(var(--text-high) / <alpha-value>)',
        'text-high': 'hsl(var(--text-high) / <alpha-value>)',
        'text-normal': 'hsl(var(--text-normal) / <alpha-value>)',
        'text-muted': 'hsl(var(--text-low) / <alpha-value>)',
        'text-low': 'hsl(var(--text-low) / <alpha-value>)',

        'bg-primary': 'hsl(var(--bg-primary) / <alpha-value>)',
        'bg-secondary': 'hsl(var(--bg-secondary) / <alpha-value>)',
        'bg-panel': 'hsl(var(--bg-panel) / <alpha-value>)',
        'bg-hover': 'hsl(var(--bg-hover) / <alpha-value>)',
        'bg-tertiary': 'hsl(var(--bg-tertiary) / <alpha-value>)',

        border: 'hsl(var(--border-color) / <alpha-value>)',
        'border-normal': 'hsl(var(--border-color) / <alpha-value>)',

        success: 'hsl(var(--success) / <alpha-value>)',
        warning: 'hsl(var(--warning) / <alpha-value>)',
        error: 'hsl(var(--error) / <alpha-value>)',
        info: 'hsl(var(--info) / <alpha-value>)',

        // 状态色
        'status-pending': 'hsl(var(--status-pending) / <alpha-value>)',
        'status-running': 'hsl(var(--status-running) / <alpha-value>)',
        'status-completed': 'hsl(var(--status-completed) / <alpha-value>)',
        'status-failed': 'hsl(var(--status-failed) / <alpha-value>)',
        'status-cancelled': 'hsl(var(--status-cancelled) / <alpha-value>)',
      },
      fontFamily: {
        sans: ['Inter', 'system-ui', 'sans-serif'],
        mono: ['JetBrains Mono', 'monospace'],
      },
      spacing: {
        navbar: 'var(--navbar-height)',
      },
      borderRadius: {
        DEFAULT: 'var(--border-radius)',
        lg: 'var(--border-radius-lg)',
      },
      boxShadow: {
        sm: 'var(--shadow-sm)',
        DEFAULT: 'var(--shadow)',
        lg: 'var(--shadow-lg)',
      },
    },
  },
  plugins: [],
}
