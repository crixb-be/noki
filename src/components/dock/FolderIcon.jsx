import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export default function FolderIcon({ path, fallback, className, style }) {
  const [icon, setIcon] = useState(null);
  const node = useRef(null);

  useEffect(() => {
    if (!path) return;
    let alive = true;
    const observer = new IntersectionObserver(([entry]) => {
      if (!entry.isIntersecting) return;
      observer.disconnect();
      invoke("folder_custom_icon", { path }).then((url) => {
        if (alive && url) setIcon({ path, url });
      }).catch(() => {});
    });
    observer.observe(node.current);
    return () => { alive = false; observer.disconnect(); };
  }, [path]);

  return <span ref={node} aria-hidden="true" className={className} style={style}>
    {icon?.path === path && icon?.url ? <img src={icon.url} alt="" draggable="false" /> : fallback}
  </span>;
}
