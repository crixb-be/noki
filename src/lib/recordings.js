import { useCallback, useEffect, useState } from "react";

/** Las grabaciones (audio + datos) viven en IndexedDB de la app: todo queda en tu equipo. */
const DB = "noki-recordings";
const STORE = "items";
const listeners = new Set();

const open = () =>
  new Promise((resolve, reject) => {
    const r = indexedDB.open(DB, 1);
    r.onupgradeneeded = () => r.result.createObjectStore(STORE, { keyPath: "id" });
    r.onsuccess = () => resolve(r.result);
    r.onerror = () => reject(r.error);
  });

async function tx(mode, fn) {
  const db = await open();
  return new Promise((resolve, reject) => {
    const t = db.transaction(STORE, mode);
    const req = fn(t.objectStore(STORE));
    t.oncomplete = () => {
      db.close();
      resolve(req?.result);
    };
    t.onerror = () => reject(t.error);
  });
}

const notify = () => listeners.forEach((fn) => fn());

export const saveRecording = (rec) => tx("readwrite", (s) => s.put(rec)).then(notify);
export const deleteRecording = (id) => tx("readwrite", (s) => s.delete(id)).then(notify);
export const getRecording = (id) => tx("readonly", (s) => s.get(id));
const listAll = () => tx("readonly", (s) => s.getAll()).then((l) => (l ?? []).sort((a, b) => b.created - a.created));

export function useRecordings() {
  const [items, setItems] = useState([]);
  const refresh = useCallback(() => listAll().then(setItems).catch(() => {}), []);
  useEffect(() => {
    refresh();
    listeners.add(refresh);
    return () => listeners.delete(refresh);
  }, [refresh]);
  return items;
}
