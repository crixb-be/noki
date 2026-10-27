/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,jsx}"],
  theme: {
    extend: {
      keyframes: {
        fadeIn: { from: { opacity: 0 }, to: { opacity: 1 } },
        // Aro ámbar que se expande: avisa sin volver translúcido el fondo negro del notch.
        ringPulse: {
          "0%": { boxShadow: "0 0 0 0 rgba(251,191,36,0.75)" },
          "100%": { boxShadow: "0 0 0 9px rgba(251,191,36,0)" },
        },
      },
    },
  },
  plugins: [],
};
