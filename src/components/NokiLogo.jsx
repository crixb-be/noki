/** El logo de Noki (el mismo de assets/noki-logo.svg): un notch simpático que cuelga del borde superior. */
export default function NokiLogo({ size = 40, className = "" }) {
  return (
    <svg viewBox="0 0 1024 1024" width={size} height={size} className={className} role="img" aria-label="Noki">
      <defs>
        <linearGradient id="noki-bg" x1="0" y1="0" x2="1" y2="1">
          <stop offset="0" stopColor="#34D6B6" />
          <stop offset="1" stopColor="#3B9CF5" />
        </linearGradient>
        <clipPath id="noki-tile">
          <rect width="1024" height="1024" rx="232" />
        </clipPath>
      </defs>
      <rect width="1024" height="1024" rx="232" fill="url(#noki-bg)" />
      <g clipPath="url(#noki-tile)">
        <path d="M214 -10 H810 V400 a160 160 0 0 1 -160 160 H374 a160 160 0 0 1 -160 -160 Z" fill="#0C0C12" />
        <ellipse cx="338" cy="420" rx="34" ry="24" fill="#FF8FB1" opacity="0.8" />
        <ellipse cx="686" cy="420" rx="34" ry="24" fill="#FF8FB1" opacity="0.8" />
        <ellipse cx="406" cy="332" rx="52" ry="68" fill="#FFFFFF" />
        <ellipse cx="618" cy="332" rx="52" ry="68" fill="#FFFFFF" />
        <circle cx="420" cy="310" r="16" fill="#0C0C12" />
        <circle cx="632" cy="310" r="16" fill="#0C0C12" />
        <path d="M452 440 Q512 494 572 440" fill="none" stroke="#FFFFFF" strokeWidth="24" strokeLinecap="round" />
      </g>
    </svg>
  );
}
