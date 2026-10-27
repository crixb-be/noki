import { useCallback, useState } from "react";
import { useStore } from "./useStore.js";
import { useAi } from "./useAi.js";

const MAX_STORED = 40; // mensajes guardados
const MAX_SENT = 12; // mensajes que se envían como contexto (menos contexto = respuesta más rápida)

/**
 * Estado del chat de IA. Vive en la ventana del notch (no en la pestaña) para que una respuesta en curso
 * siga llegando y guardándose aunque cambies de pestaña o cierres el notch.
 */
export function useAiChat() {
  const [messages, setMessages] = useStore("chat", []);
  const [prompt, setPrompt] = useState("");
  const [error, setError] = useState(null);
  const { busy, send, stop } = useAi();

  const submit = useCallback(async () => {
    const text = prompt.trim();
    if (!text || busy) return;
    setPrompt("");
    setError(null);
    const history = [...messages, { role: "user", content: text }];
    setMessages([...history, { role: "assistant", content: "" }]);

    let answer = "";
    const { error: err } = await send({
      messages: history.slice(-MAX_SENT),
      onChunk: (chunk) => {
        answer += chunk;
        setMessages([...history, { role: "assistant", content: answer }]);
      },
    });
    if (err) setError(err);
    // Se guarda lo recibido (aunque se haya detenido o fallado a medias); una respuesta vacía no se conserva.
    setMessages(answer ? [...history, { role: "assistant", content: answer }].slice(-MAX_STORED) : history.slice(-MAX_STORED));
  }, [prompt, busy, messages, send, setMessages]);

  const clear = useCallback(() => {
    setMessages([]);
    setError(null);
  }, [setMessages]);

  return { messages, prompt, setPrompt, error, busy, submit, stop, clear };
}
