import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

const EMPTY = {
  available: false,
  trackKey: "",
  title: "",
  artist: "",
  album: "",
  source: "",
  playing: false,
  position: 0,
  duration: 0,
  canPlayPause: false,
  canNext: false,
  canPrev: false,
  canSeek: false,
};

/**
 * Música real vía los controles multimedia de Windows (GSMTC): lo que suene en Spotify,
 * YouTube Music, el navegador, VLC… Rust emite `media-state`; aquí solo se refleja y se
 * interpola la posición entre eventos para que la barra avance con fluidez.
 */
export function useMedia() {
  const [state, setState] = useState(EMPTY);
  const [art, setArt] = useState(null);
  const [position, setPosition] = useState(0);
  const base = useRef({ pos: 0, at: 0, playing: false, duration: 0 });

  const apply = useCallback((s) => {
    const { art: newArt, ...rest } = s;
    if (newArt) setArt(newArt);
    else if (!s.artKey) setArt(null);
    base.current = { pos: s.position, at: performance.now(), playing: s.playing, duration: s.duration };
    setState(rest);
    setPosition(s.position);
  }, []);

  useEffect(() => {
    let unlisten;
    let cancelled = false;
    invoke("media_state").then((s) => !cancelled && apply(s)).catch(() => {});
    listen("media-state", (e) => apply(e.payload)).then((fn) => (cancelled ? fn() : (unlisten = fn)));
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [apply]);

  useEffect(() => {
    if (!state.playing) return;
    const id = setInterval(() => {
      const b = base.current;
      const p = b.pos + (performance.now() - b.at) / 1000;
      setPosition(b.duration > 0 ? Math.min(p, b.duration) : p);
    }, 250);
    return () => clearInterval(id);
  }, [state.playing, state.trackKey]);

  const control = useCallback((action) => invoke("media_control", { action }).catch(() => {}), []);
  const seek = useCallback((seconds) => {
    base.current = { ...base.current, pos: seconds, at: performance.now() };
    setPosition(seconds);
    invoke("media_seek", { seconds }).catch(() => {});
  }, []);

  return {
    available: state.available,
    playing: state.playing,
    trackKey: state.trackKey,
    track: {
      title: state.title,
      artist: state.artist,
      album: state.album,
      source: state.source,
      duration: state.duration,
      art,
    },
    can: { playPause: state.canPlayPause, next: state.canNext, prev: state.canPrev, seek: state.canSeek },
    position,
    toggle: useCallback(() => control("toggle"), [control]),
    next: useCallback(() => control("next"), [control]),
    prev: useCallback(() => control("prev"), [control]),
    seek,
  };
}
