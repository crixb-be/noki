/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,jsx}"],
  theme: {
    extend: {
      keyframes: {
        fadeIn: { from: { opacity: 0 }, to: { opacity: 1 } },
        // Notch abierto: el contenido aparece tras empezar a crecer; la cabecera baja, la vista sube.
        expandIn: { from: { opacity: 0, transform: "scale(0.96)" }, to: { opacity: 1, transform: "scale(1)" } },
        dropIn: { from: { opacity: 0, transform: "translateY(-8px)" }, to: { opacity: 1, transform: "translateY(0)" } },
        tabIn: { from: { opacity: 0, transform: "translateY(8px)" }, to: { opacity: 1, transform: "translateY(0)" } },
        // Notch cerrado y avisos.
        compactIn: { from: { opacity: 0, transform: "scale(0.94)" }, to: { opacity: 1, transform: "scale(1)" } },
        peekIn: { from: { opacity: 0, transform: "scale(0.96)" }, to: { opacity: 1, transform: "scale(1)" } },
        bannerIn: {
          from: { opacity: 0, transform: "translate(-50%, 12px) scale(0.9)" },
          to: { opacity: 1, transform: "translate(-50%, 0) scale(1)" },
        },
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
