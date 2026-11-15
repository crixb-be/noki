import { useId } from "react";

/** Dedicated artwork: never resolve the recycle-bin window/executable through app_icon. */
export default function TrashIcon({ full }) {
  const id = `trash-${useId().replace(/:/g, "")}`;
  return <svg className="dock-trash-can" viewBox="0 0 64 64" fill="none" aria-hidden="true" data-trash-state={full ? "full" : "empty"}>
    <defs>
      <linearGradient id={`${id}-metal`} x1="13" y1="20" x2="51" y2="48" gradientUnits="userSpaceOnUse"><stop stopColor="#f4f7fb" stopOpacity=".9" /><stop offset=".25" stopColor="#aab8c7" stopOpacity=".58" /><stop offset=".55" stopColor="#e8eff6" stopOpacity=".8" /><stop offset="1" stopColor="#788797" stopOpacity=".8" /></linearGradient>
      <linearGradient id={`${id}-rim`} x1="12" y1="13" x2="52" y2="21" gradientUnits="userSpaceOnUse"><stop stopColor="#fff" /><stop offset=".5" stopColor="#a9b5c3" /><stop offset="1" stopColor="#f1f5fa" /></linearGradient>
      <linearGradient id={`${id}-inside`} x1="32" y1="13" x2="32" y2="24" gradientUnits="userSpaceOnUse"><stop stopColor="#26313f" /><stop offset="1" stopColor="#78899b" /></linearGradient>
      <clipPath id={`${id}-body`}><path d="M13 18C17 23 47 23 51 18L46 53C44 59 20 59 18 53Z" /></clipPath>
    </defs>
    <ellipse cx="32" cy="58" rx="18" ry="3" fill="#000" opacity=".2" />
    <ellipse cx="32" cy="18" rx="19" ry="7" fill={`url(#${id}-inside)`} />
    {full && <g stroke="#c6d0dd" strokeWidth=".7">
      <path d="m20 23-3-14 13-3 7 18" fill="#f7f9fc" /><path d="m23 11 8-2m-7 6 8-2m-7 6 8-2" stroke="#a2afbf" />
      <path d="m31 25 3-20 13 5-1 17" fill="#e1e8f0" /><path d="m37 10 7 3m-8 2 7 3" stroke="#9aa8ba" />
      <path d="m16 21 9-7 8 9 11-9 7 8-9 11-19-1Z" fill="#f9fbff" />
    </g>}
    <path d="M13 18C17 23 47 23 51 18L46 53C44 59 20 59 18 53Z" fill={`url(#${id}-metal)`} stroke="#ecf3fc" strokeOpacity=".75" strokeWidth="1.2" />
    <g clipPath={`url(#${id}-body)`} stroke="#526477" strokeOpacity=".38" strokeWidth=".7">
      {[18, 24, 30, 36, 42, 48].map((x) => <path key={x} d={`M${x} 20 ${32 + (x - 32) * .74} 58`} />)}
      {[27, 34, 41, 48, 54].map((y) => <path key={y} d={`M12 ${y}Q32 ${y + 7} 52 ${y}`} />)}
    </g>
    <path d="M16 22 21 52" stroke="#fff" strokeOpacity=".65" strokeWidth="1.5" strokeLinecap="round" />
    <path d="M19 53C23 58 41 58 45 53" stroke="#e4edf7" strokeWidth="1.5" />
    <ellipse cx="32" cy="18" rx="19" ry="7" stroke={`url(#${id}-rim)`} strokeWidth="2.6" />
    <path d="M14 17C17 10 44 9 50 17" stroke="#fff" strokeOpacity=".85" strokeWidth="1" />
  </svg>;
}
