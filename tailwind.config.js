/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,jsx}"],
  theme: {
    extend: {
      keyframes: {
        fadeIn: { from: { opacity: 0 }, to: { opacity: 1 } },
        // Entrada del contenido del notch: aparece mientras la cápsula termina de crecer.
        contentIn: { from: { opacity: 0, transform: "translateY(-6px) scale(0.98)" }, to: { opacity: 1, transform: "none" } },
        tabIn: { from: { opacity: 0, transform: "translateY(8px)" }, to: { opacity: 1, transform: "none" } },
        popIn: { from: { opacity: 0, transform: "scale(0.85)" }, to: { opacity: 1, transform: "none" } },
        bannerIn: { from: { opacity: 0, transform: "translate(-50%, 10px) scale(0.95)" }, to: { opacity: 1, transform: "translate(-50%, 0) scale(1)" } },
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
