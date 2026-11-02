import { useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";

/** Alto del cuerpo del campo (sin la zona del notch): solo el input, o el input con la respuesta debajo. */
export const QUICK_H = { input: 64, answer: 210 };
export const QUICK_W = 480;

/**
 * Pregunta rápida a la IA: un campo flotante en el notch, sin abrir el panel completo. Usa el mismo chat que la
 * pestaña «Asistente IA» (la conversación queda guardada ahí). Enter envía; Esc cierra el notch.
 */
export default function QuickAsk({ chat, asked, onAsk, onFull, collapsedH }) {
  const { messages, prompt, setPrompt, error, busy, submit, stop } = chat;
  const input = useRef(null);
  const bottom = useRef(null);
  const last = messages[messages.length - 1];
  const answer = asked && last?.role === "assistant" ? last.content : "";

  useEffect(() => {
    // La ventana tarda un instante en recibir el foco al abrirse con el atajo.
    const id = setTimeout(() => input.current?.focus(), 120);
    const id2 = setTimeout(() => input.current?.focus(), 420);
    return () => (clearTimeout(id), clearTimeout(id2));
  }, []);
  useEffect(() => {
    bottom.current?.scrollIntoView({ block: "end" });
  }, [answer, busy]);

  const send = () => {
    if (!prompt.trim() || busy) return;
    onAsk();
    submit();
  };

  return (
    <div className="flex h-full animate-[fadeIn_220ms_ease-out_both] flex-col gap-2 px-4 pb-3" style={{ paddingTop: collapsedH + 6 }}>
      <form onSubmit={(e) => (e.preventDefault(), send())} className="flex shrink-0 items-center gap-2">
        <span className="text-base" aria-hidden>✨</span>
        <input
          ref={input}
          value={prompt}
          onChange={(e) => setPrompt(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && !e.shiftKey && (e.preventDefault(), send())}
          placeholder="Pregunta algo a la IA…"
          className="h-8 min-w-0 flex-1 select-text rounded-full bg-white/10 px-4 text-xs outline-none transition-colors placeholder:text-white/35 focus:bg-white/15"
          aria-label="Pregunta rápida a la IA"
        />
        {busy ? (
          <button type="button" onClick={stop} className="h-8 rounded-full bg-red-500/80 px-3 text-xs transition-all hover:bg-red-500 active:scale-95">■</button>
        ) : (
          <button className="h-8 rounded-full bg-white px-3 text-sm font-medium text-black transition-all hover:bg-white/90 active:scale-95" aria-label="Enviar">➤</button>
        )}
        <button type="button" onClick={onFull} className="h-8 rounded-full bg-white/10 px-2.5 text-xs transition-colors hover:bg-white/20" title="Abrir el chat completo">⤢</button>
      </form>
      {(asked || error) && (
        <div className="min-h-0 flex-1 overflow-y-auto rounded-2xl bg-black/30 px-3 py-2 text-xs leading-relaxed">
          {error ? (
            <span className="text-red-300">
              {error}
              {/Ajustes|API|modelo|clave/i.test(error) && (
                <button type="button" onClick={() => invoke("open_settings", { section: "ai" })} className="ml-2 text-sky-300 underline">
                  Abrir ajustes
                </button>
              )}
            </span>
          ) : (
            <span className="select-text whitespace-pre-wrap">{answer || <span className="animate-pulse text-white/40">Escribiendo…</span>}</span>
          )}
          <div ref={bottom} />
        </div>
      )}
    </div>
  );
}
