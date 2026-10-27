import { useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { PROVIDER_NAMES } from "../hooks/useAi.js";

export default function AIChat({ ai, chat }) {
  const { messages, prompt, setPrompt, error, busy, submit, stop, clear } = chat;
  const bottom = useRef(null);

  const cfg = ai[ai.provider];
  const label = `${PROVIDER_NAMES[ai.provider]} · ${cfg.model}`;

  useEffect(() => {
    bottom.current?.scrollIntoView({ block: "end" });
  }, [messages, busy]);

  const onKeyDown = (e) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      submit();
    }
  };

  return (
    <div className="flex h-full flex-col gap-2">
      <div className="flex shrink-0 items-center justify-between px-1 text-[10px] text-white/35">
        <span className="truncate">{label}</span>
        {messages.length > 0 && (
          <button onClick={clear} disabled={busy} className="hover:text-white/70 disabled:opacity-40">
            Nueva conversación
          </button>
        )}
      </div>
      <div className="min-h-0 flex-1 space-y-2 overflow-y-auto rounded-3xl bg-white/5 p-3 text-xs">
        {messages.length === 0 && !error && (
          <div className="flex h-full flex-col items-center justify-center gap-1 text-center text-white/40">
            <span className="text-xl">✨</span>
            <span>Pregunta lo que quieras a {PROVIDER_NAMES[ai.provider]}</span>
            <span className="text-[10px] text-white/25">Enter envía · Mayús+Enter salto de línea</span>
          </div>
        )}
        {messages.map((m, i) => (
          <div key={i} className={m.role === "user" ? "flex justify-end" : "flex justify-start"}>
            <div
              className={`max-w-[88%] select-text whitespace-pre-wrap rounded-2xl px-3 py-1.5 leading-relaxed ${
                m.role === "user" ? "bg-white/15" : "bg-black/40"
              }`}
            >
              {m.content || (busy && i === messages.length - 1 ? <span className="animate-pulse text-white/40">Escribiendo…</span> : "")}
            </div>
          </div>
        ))}
        {error && (
          <div className="rounded-2xl bg-red-500/10 px-3 py-2 text-red-300">
            {error}
            {/Ajustes|API|modelo|clave/i.test(error) && (
              <button onClick={() => invoke("open_settings", { section: "ai" })} className="ml-2 text-sky-300 underline">
                Abrir ajustes
              </button>
            )}
          </div>
        )}
        <div ref={bottom} />
      </div>
      <form onSubmit={(e) => { e.preventDefault(); submit(); }} className="flex shrink-0 items-end gap-2">
        <textarea
          value={prompt}
          onChange={(e) => setPrompt(e.target.value)}
          onKeyDown={onKeyDown}
          rows={1}
          placeholder="Escribe tu pregunta…"
          className="max-h-16 min-h-[34px] min-w-0 flex-1 resize-none rounded-2xl bg-white/10 px-4 py-2 text-xs outline-none transition-all duration-300 placeholder:text-white/30 focus:bg-white/15 select-text"
        />
        {busy ? (
          <button type="button" onClick={stop} className="h-[34px] rounded-full bg-red-500/80 px-4 text-xs font-medium transition-all hover:bg-red-500 active:scale-95">
            ■ Parar
          </button>
        ) : (
          <button className="h-[34px] rounded-full bg-white px-4 text-sm font-medium text-black transition-all duration-300 hover:bg-white/90 active:scale-95">➤</button>
        )}
      </form>
    </div>
  );
}
