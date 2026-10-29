import { EAR, NOTCH_EASE, NOTCH_MS } from "../../lib/notchGeometry.js";

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
        transition: `background ${NOTCH_MS}ms ${NOTCH_EASE}`,
      }}
    />
  );
}
