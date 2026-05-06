/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./src/**/*.{html,ts,tsx}"],
  theme: {
    extend: {
      colors: {
        brand: {
          50: "rgb(var(--hh-color-brand-50) / <alpha-value>)",
          100: "rgb(var(--hh-color-brand-100) / <alpha-value>)",
          200: "rgb(var(--hh-color-brand-200) / <alpha-value>)",
          300: "rgb(var(--hh-color-brand-300) / <alpha-value>)",
          400: "rgb(var(--hh-color-brand-400) / <alpha-value>)",
          500: "rgb(var(--hh-color-brand-500) / <alpha-value>)",
          600: "rgb(var(--hh-color-brand-600) / <alpha-value>)",
          700: "rgb(var(--hh-color-brand-700) / <alpha-value>)",
          800: "rgb(var(--hh-color-brand-800) / <alpha-value>)",
          900: "rgb(var(--hh-color-brand-900) / <alpha-value>)",
        },
      },
    },
  },
  plugins: [],
};
