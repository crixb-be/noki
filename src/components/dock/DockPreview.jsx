import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useSettings } from "../../hooks/useSettings.js";
import { useGlass, useGlassShape } from "../../lib/glass.js";

export default function DockPreview() {
  const { settings } = useSettings();
  const [preview, setPreview] = useState(null);
  const panel = useRef(null);
  const glass = useGlass(settings, "panels");
  useGlassShape(panel, glass, [preview?.request]);

  useEffect(() => {
    let alive = true;
    const subscriptions = [
      listen("dock-preview", ({ payload }) => setPreview(payload)),
      listen("dock-preview-image", ({ payload: [request, card] }) => {
        setPreview((current) => current?.request === request
          ? { ...current, cards: current.cards.map((entry) => entry.hwnd === card.hwnd ? card : entry) }
          : current);
      }),
    ];
    Promise.all(subscriptions).then(() => invoke("dock_preview_state")).then((state) => {
      if (alive && state) setPreview((current) => !current || current.request < state.request ? state : current);
    }).catch(console.error);
    return () => {
      alive = false;
      subscriptions.forEach((subscription) => subscription.then((unlisten) => unlisten()).catch(console.error));
    };
  }, []);

  const act = (hwnd, dismiss) => {
    invoke("dock_preview_action", { request: preview.request, hwnd, dismiss }).catch(console.error);
  };

  return <div ref={panel} className="dock-preview">
    {preview?.cards.length ? <div className="dock-preview-list">
      {preview.cards.map((card) => <div key={card.hwnd} className="dock-preview-card">
        <button className="dock-preview-window" onClick={() => act(card.hwnd, false)} aria-label={card.title || preview.name}>
          <span className="dock-preview-title">{card.title || preview.name}</span>
          <span className="dock-preview-image">
            {card.image ? <img src={card.image} alt="" draggable="false" /> : <span className="dock-preview-placeholder" aria-hidden="true">▣</span>}
          </span>
        </button>
        <button className="dock-preview-close" aria-label={`Cerrar ${card.title || preview.name}`} onClick={() => act(card.hwnd, true)}>×</button>
      </div>)}
    </div> : <span className="dock-preview-label">{preview?.name}</span>}
  </div>;
}
