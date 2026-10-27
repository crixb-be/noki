import { useEffect, useRef, useState } from "react";
import { useStore } from "../hooks/useStore.js";

const title = (text) => text.split("\n")[0].trim() || "Nota vacía";

/** Notas rápidas: lista a la izquierda, editor a la derecha, guardado automático. */
export default function Notes() {
  const [notes, setNotes] = useStore("notes", []);
  const [selected, setSelected] = useState(() => notes[0]?.id ?? null);
  const [confirm, setConfirm] = useState(false);
  const current = notes.find((n) => n.id === selected);

  // Texto local para escribir fluido; se vuelca al almacén con un pequeño retraso.
  const [draft, setDraft] = useState(current?.text ?? "");
  const saveTimer = useRef(null);
  // Solo se guarda lo que se escribió aquí: con varios notch, otra ventana pudo cambiar la nota entretanto.
  const dirty = useRef(false);
  const draftFor = useRef(selected);

  useEffect(() => {
    clearTimeout(saveTimer.current);
    draftFor.current = selected;
    setDraft(notes.find((n) => n.id === selected)?.text ?? "");
    setConfirm(false);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selected]);

  const flush = (id, text) =>
    setNotes((l) => l.map((n) => (n.id === id ? { ...n, text, updated: Date.now() } : n)));

  const onChange = (e) => {
    const text = e.target.value;
    setDraft(text);
    dirty.current = true;
    clearTimeout(saveTimer.current);
    const id = selected;
    saveTimer.current = setTimeout(() => {
      flush(id, text);
      dirty.current = false;
    }, 350);
  };

  // Al cerrar el notch se desmonta: se guarda lo pendiente.
  useEffect(
    () => () => {
      if (saveTimer.current) {
        clearTimeout(saveTimer.current);
      }
    },
    []
  );
  const latest = useRef({});
  latest.current = { draft, selected };
  useEffect(
    () => () => {
      const { draft: d, selected: s } = latest.current;
      if (s != null && dirty.current) flush(s, d);
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    []
  );

  const create = () => {
    if (selected != null && dirty.current) {
      flush(selected, draft);
      dirty.current = false;
    }
    const n = { id: Date.now(), text: "", updated: Date.now() };
    setNotes((l) => [n, ...l]);
    setSelected(n.id);
  };

  const remove = () => {
    if (!confirm) return setConfirm(true);
    const rest = notes.filter((n) => n.id !== selected);
    clearTimeout(saveTimer.current);
    setNotes(rest);
    setSelected(rest[0]?.id ?? null);
  };

  const ordered = [...notes].sort((a, b) => b.updated - a.updated);

  return (
    <div className="flex h-full gap-2 rounded-3xl bg-white/5 p-3">
      <div className="flex w-36 shrink-0 flex-col gap-1">
        <button onClick={create} className="rounded-full bg-white/10 py-1 text-xs transition-colors hover:bg-white/20">＋ Nueva</button>
        <ul className="flex min-h-0 flex-1 flex-col gap-0.5 overflow-y-auto pr-1">
          {ordered.length === 0 && <li className="py-4 text-center text-[11px] text-white/30">Sin notas</li>}
          {ordered.map((n) => (
            <li key={n.id}>
              <button
                onClick={() => setSelected(n.id)}
                className={`w-full truncate rounded-xl px-2 py-1 text-left text-xs transition-colors ${n.id === selected ? "bg-white/15" : "text-white/55 hover:bg-white/5"}`}
              >
                {n.id === selected ? title(draft) : title(n.text)}
              </button>
            </li>
          ))}
        </ul>
      </div>
      {current ? (
        <div className="flex min-w-0 flex-1 flex-col gap-1">
          <textarea
            value={draft}
            onChange={onChange}
            placeholder="Escribe aquí…"
            spellCheck={false}
            className="min-h-0 flex-1 resize-none rounded-2xl bg-white/5 p-3 text-xs leading-relaxed outline-none placeholder:text-white/25 focus:bg-white/10 select-text"
          />
          <div className="flex items-center justify-between text-[10px] text-white/30">
            <span>{draft.length} caracteres</span>
            <button onClick={remove} className={confirm ? "text-red-400" : "hover:text-white/70"}>
              {confirm ? "¿Seguro? Clic para borrar" : "🗑 Borrar"}
            </button>
          </div>
        </div>
      ) : (
        <div className="flex flex-1 items-center justify-center text-xs text-white/30">Crea una nota para empezar</div>
      )}
    </div>
  );
}
