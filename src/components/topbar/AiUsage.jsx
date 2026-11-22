import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { locale } from "../../lib/i18n.js";

const providers = {
  claude: { name: "Claude", setting: "showClaudeQuota" },
  codex: { name: "ChatGPT / Codex", setting: "showCodexQuota" },
  gemini: { name: "Gemini", setting: "showGeminiQuota" },
};

export function enabledProviders(topbar) {
  if (!topbar?.enabled || !topbar.showAiUsage) return [];
  return Object.entries(providers).filter(([, { setting }]) => topbar[setting]);
}

export function ProviderIcon({ provider }) {
  if (provider === "claude") {
    return <svg aria-hidden="true" viewBox="0 0 24 24" className="ai-provider-icon ai-provider-claude" fill="none" stroke="currentColor" strokeWidth="2.1" strokeLinecap="round">
      <path d="m12 2 .6 9.2L20 5m2 7-9 1 6.2 6.5M12 22l-.5-9-6.3 6.2M2 12l9-.7L5.5 5" />
    </svg>;
  }
  if (provider === "gemini") {
    return <svg aria-hidden="true" viewBox="0 0 24 24" className="ai-provider-icon ai-provider-gemini" fill="currentColor">
      <path d="M12 1c1.2 6.5 4.5 9.8 11 11-6.5 1.2-9.8 4.5-11 11-1.2-6.5-4.5-9.8-11-11C7.5 10.8 10.8 7.5 12 1Z" />
    </svg>;
  }
  return <svg aria-hidden="true" viewBox="0 0 24 24" className="ai-provider-icon" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
    <rect x="2.5" y="3.5" width="19" height="17" rx="5" /><path d="m7 8 4 4-4 4m7 0h3.5" />
  </svg>;
}

export function useAiUsage(enabled) {
  const [snapshot, setSnapshot] = useState(null);
  useEffect(() => {
    if (!enabled) { setSnapshot(null); return; }
    let disposed = false;
    let received = false;
    const subscription = listen("ai-usage-updated", ({ payload }) => {
      received = true;
      if (!disposed) setSnapshot(payload);
    });
    subscription.then(() => invoke("ai_usage_snapshot")).then((value) => {
      if (!disposed && !received) setSnapshot(value);
    }).catch(console.error);
    return () => {
      disposed = true;
      subscription.then((unlisten) => unlisten()).catch(console.error);
    };
  }, [enabled]);
  return snapshot;
}

const timestamp = (seconds) => new Intl.DateTimeFormat(locale(), { dateStyle: "short", timeStyle: "short" }).format(new Date(seconds * 1000));

function duration(minutes) {
  if (!minutes) return "Ventana de cuota";
  const [value, unit] = minutes % 1440 === 0 ? [minutes / 1440, "day"] : minutes % 60 === 0 ? [minutes / 60, "hour"] : [minutes, "minute"];
  return new Intl.NumberFormat(locale(), { style: "unit", unit, unitDisplay: "long" }).format(value);
}

export function AiUsagePanel({ topbar }) {
  const visible = enabledProviders(topbar);
  const snapshot = useAiUsage(visible.length > 0);
  return <>
    <header className="ai-usage-header"><h2>Uso de proveedores de IA</h2>
      <button type="button" className="panel-link" onClick={() => invoke("refresh_ai_usage").catch(console.error)} disabled={!visible.length || snapshot?.refreshing}>Actualizar</button>
    </header>
    {!visible.length && <p className="panel-muted">No hay monitores de IA activos.</p>}
    {visible.map(([id, { name }]) => {
      const provider = snapshot?.[id];
      return <section key={id} className="ai-usage-provider">
        <h3><ProviderIcon provider={id} />{name}</h3>
        {provider?.limits.map((limit, index) => {
          const title = limit.label ?? duration(limit.minutes);
          return <div key={index} className="ai-usage-limit">
            <div><span>{title}</span><strong>{Math.round(limit.usedPercent)}%</strong></div>
            <progress max="100" value={limit.usedPercent} aria-label={title} />
            {limit.resetsAt && <p className="panel-muted">Se reinicia el <time dateTime={new Date(limit.resetsAt * 1000).toISOString()}>{timestamp(limit.resetsAt)}</time></p>}
          </div>;
        })}
        {provider?.updatedAt && <p className="panel-muted">Última consulta <time>{timestamp(provider.updatedAt)}</time></p>}
        {provider?.error && <p className="panel-muted" role="status">{provider.error}</p>}
        {!provider?.limits.length && !provider?.error && <p className="panel-muted">Consultando cuota…</p>}
      </section>;
    })}
  </>;
}
