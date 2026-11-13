import React from "react";

const base = { viewBox: "0 0 24 24", width: 17, height: 17, fill: "none", stroke: "currentColor", strokeWidth: 1.75, strokeLinecap: "round", strokeLinejoin: "round", shapeRendering: "geometricPrecision", "aria-hidden": true };

function Svg({ children, ...props }) {
  return <svg {...base} {...props}>{children}</svg>;
}

export function WifiIcon({ level = 3, off = false }) {
  return <Svg><path opacity={off ? 0.35 : level < 1 ? 0.3 : 1} d="M3.5 8.8a14.8 14.8 0 0 1 17 0" /><path opacity={off ? 0.35 : level < 2 ? 0.3 : 1} d="M6.8 12.2a9.6 9.6 0 0 1 10.4 0" /><path opacity={off ? 0.35 : level < 3 ? 0.3 : 1} d="M10 15.5a4.5 4.5 0 0 1 4 0" /><circle cx="12" cy="19" r="1.2" fill="currentColor" stroke="none" />{off && <path d="m4 4 16 16" />}</Svg>;
}

export function BluetoothIcon({ off = false }) {
  return <Svg><path opacity={off ? 0.35 : 1} d="M12 3v18l5-4.5-10-9L17 3.5 7 13" />{off && <path d="m4 4 16 16" />}</Svg>;
}

export function SpeakerIcon({ level = 3, muted = false }) {
  return <Svg><path d="M4 10h3l4-3.5v11L7 14H4z" fill={muted ? "none" : "currentColor"} /><path opacity={muted ? 0.35 : level > 0 ? 1 : 0.3} d="M15 9a4 4 0 0 1 0 6" /><path opacity={muted ? 0.35 : level > 1 ? 1 : 0.3} d="M17.5 6.7a7.2 7.2 0 0 1 0 10.6" />{muted && <path d="m16 9 5 6m0-6-5 6" />}</Svg>;
}

export function BatteryIcon({ pct = 100, charging = false }) {
  const width = Math.max(1, Math.min(16, 16 * pct / 100));
  return <Svg><rect x="3" y="6" width="16" height="12" rx="3" /><path d="M20 10v4" /><rect x="5" y="8" width={width} height="8" rx="1.5" fill="currentColor" stroke="none" />{charging && <path d="m13 3-2 4h3l-2 4" />}</Svg>;
}

export function SearchIcon() { return <Svg><circle cx="10.5" cy="10.5" r="5.8" /><path d="m15 15 5 5" /></Svg>; }
export function ControlCenterIcon() { return <Svg><path d="M4 8h16M4 16h16" /><circle cx="9" cy="8" r="2" fill="currentColor" stroke="none" /><circle cx="15" cy="16" r="2" fill="currentColor" stroke="none" /></Svg>; }
export function RecordIcon() { return <Svg><circle cx="12" cy="12" r="8" /><circle cx="12" cy="12" r="3.2" fill="currentColor" stroke="none" /></Svg>; }
export function CameraIcon() { return <Svg><path d="M4 8.5h3l1.5-2h7l1.5 2h3v9H4z" /><circle cx="12" cy="13" r="3" /></Svg>; }
export function BellIcon() { return <Svg><path d="M6 17h12l-1.4-2v-3.4a4.6 4.6 0 0 0-9.2 0V15z" /><path d="M10 19a2.2 2.2 0 0 0 4 0" /></Svg>; }

