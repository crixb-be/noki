// Deben coincidir con EAR y SHADOW en src-tauri/src/notch.rs
export const EAR = 16;

/** Orejas cóncavas que unen el notch con el borde superior de la pantalla. */
export default function Ear({ side, bg }) {
  const left = side === "left";
  return (
    <div
      className="absolute top-0"
      style={{
        width: EAR,
        height: EAR,
        [left ? "right" : "left"]: "100%",
        background: `radial-gradient(circle at ${left ? "0" : "100%"} 100%, transparent ${EAR - 1}px, ${bg} ${EAR - 0.5}px)`,
      }}
    />
  );
}
