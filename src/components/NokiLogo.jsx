import { useId } from "react";
import { DEFAULT_ICON, iconById } from "../lib/icons.js";

/** El logo de Noki (el mismo de assets/noki-logo.svg): un notch sonriente que cuelga del borde superior. `variant` elige el degradado. */
export default function NokiLogo({ size = 40, className = "", variant = DEFAULT_ICON }) {
  const uid = useId().replace(/:/g, "");
  const { stops } = iconById(variant);
  return (
    <svg viewBox="0 0 1024 1024" width={size} height={size} className={className} role="img" aria-label="Noki">
      <defs>
        <linearGradient id={`noki-bg-${uid}`} x1="0" y1="0" x2="0" y2="1">
          {stops.map(([offset, color]) => (
            <stop key={offset} offset={offset} stopColor={color} />
          ))}
        </linearGradient>
        <clipPath id={`noki-tile-${uid}`}>
          <rect width="1024" height="1024" rx="232" />
        </clipPath>
      </defs>
      <rect width="1024" height="1024" rx="232" fill={`url(#noki-bg-${uid})`} />
      <g clipPath={`url(#noki-tile-${uid})`}>
        <path d="M214 -10 H810 V400 a160 160 0 0 1 -160 160 H374 a160 160 0 0 1 -160 -160 Z" fill="#0C0C12" />
        <path d="M372 350 Q418 290 464 350" fill="none" stroke="#FFFFFF" strokeWidth="22" strokeLinecap="round" />
        <path d="M560 350 Q606 290 652 350" fill="none" stroke="#FFFFFF" strokeWidth="22" strokeLinecap="round" />
        <path d="M474 424 Q512 462 550 424" fill="none" stroke="#FFFFFF" strokeWidth="18" strokeLinecap="round" />
      </g>
    </svg>
  );
}
