import { useCallback, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

/**
 * Conversación con el proveedor de IA activo (Gemini, Claude, OpenAI-compatible u Ollama).
 * Rust hace la llamada y emite `ai-chunk` con cada fragmento; aquí solo se acumula el texto.
 */
export function useAi() {
  const [busy, setBusy] = useState(false);
  const current = useRef(null);

  /** Devuelve `{ error }` si falló; en cualquier caso `onChunk` ya recibió el texto parcial. */
  const send = useCallback(async ({ messages, system, onChunk }) => {
    const id = crypto.randomUUID();
    current.current = id;
    setBusy(true);
    const unlisten = await listen("ai-chunk", (e) => e.payload.id === id && onChunk(e.payload.text));
    try {
      await invoke("ai_chat", { id, messages, system: system ?? null });
      return {};
    } catch (e) {
      return { error: String(e) };
    } finally {
      unlisten();
      current.current = null;
      setBusy(false);
    }
  }, []);

  const stop = useCallback(() => {
    if (current.current) invoke("ai_cancel", { id: current.current }).catch(() => {});
  }, []);

  return { busy, send, stop };
}

export const PROVIDER_NAMES = { gemini: "Gemini", claude: "Claude", openai: "OpenAI", ollama: "Ollama" };
