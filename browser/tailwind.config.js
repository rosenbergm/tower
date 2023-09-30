/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  theme: {
    extend: {
      fontFamily: {
        mono: ["'Berkeley Mono'", 'monospace'],
        sans: ["'elza-text'", 'sans-serif']
      },
      gridTemplateColumns: {
        'auto-fit': 'repeat(auto-fill, minmax(18rem, 1fr))',
      },
      fontSize: {
        '2xs': '.625rem',
      }
    },
  },
  plugins: [],
}

