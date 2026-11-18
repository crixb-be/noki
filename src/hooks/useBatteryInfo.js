import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

// Both bar windows consume the same native snapshot and two-second polling stream.
export function useBatteryInfo() {
  const [battery, setBattery] = useState(null);
  useEffect(() => {
    let disposed = false;
    let unlisten;
    let receivedEvent = false;
    listen("battery-info", ({ payload }) => {
      receivedEvent = true;
      if (!disposed) setBattery(payload);
    }).then(async (stop) => {
      if (disposed) { stop(); return; }
      unlisten = stop;
      const initial = await invoke("command_battery_info");
      if (!disposed && !receivedEvent) setBattery(initial);
    }).catch(console.error);
    return () => { disposed = true; unlisten?.(); };
  }, []);
  return battery;
}
