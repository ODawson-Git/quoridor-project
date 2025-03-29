/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html", // Checks the root index.html
    "./src/**/*.{js,jsx,ts,tsx}", // Checks all relevant files in src/**
  ],
  theme: {
    extend: {},
  },
  plugins: [],
}