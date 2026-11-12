/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  theme: {
    extend: {
      colors: {
        obsidian: {
          950: '#0a0c13',
          900: '#0f111a',
          850: '#141724',
          800: '#191d2e',
          750: '#21263c',
          700: '#2c3350',
          600: '#374064',
        },
        omni: {
          indigo: '#6366f1',
          indigoBright: '#818cf8',
          cyan: '#06b6d4',
          cyanBright: '#22d3ee',
          emerald: '#10b981',
          emeraldBright: '#34d399',
          amber: '#f59e0b',
          amberBright: '#fbbf24',
          rose: '#ef4444',
          roseBright: '#f87171',
          lavender: '#818cf8',
        }
      }
    },
  },
  plugins: [],
};
