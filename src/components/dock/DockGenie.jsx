import { useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

const STRIPS = 48;
const clamp = (value) => Math.min(1, Math.max(0, value));
const ease = (value) => value * value * (3 - 2 * value);
const mix = (start, end, amount) => start + (end - start) * amount;

function draw(context, image, source, target, progress) {
  const [left, top, right, bottom] = source;
  const [endLeft, endTop, endRight, endBottom] = target;
  const center = (left + right) / 2;
  const endX = (endLeft + endRight) / 2;
  const endY = (endTop + endBottom) / 2;
  const narrow = Math.max(3, Math.min(14, (endRight - endLeft) * 0.28));
  const amount = ease(progress);
  const y = (ratio) => mix(mix(top, bottom, ratio), endY + (ratio - 0.5) * narrow, amount);
  context.clearRect(0, 0, context.canvas.width, context.canvas.height);
  context.globalAlpha = 1 - ease(clamp((progress - 0.78) / 0.22));
  for (let index = 0; index < STRIPS; index++) {
    const row = index / STRIPS;
    const next = (index + 1) / STRIPS;
    // Las franjas inferiores se estrechan primero. La transición continua entre franjas dibuja la curva.
    const bend = ease(clamp((progress - (1 - row) * 0.32) / 0.68));
    const width = mix(right - left, narrow, bend);
    const x = mix(center, endX, bend) - width / 2;
    const from = y(row);
    context.drawImage(image, 0, row * image.height, image.width, image.height / STRIPS, x, from, width, y(next) - from + 0.6);
  }
  context.globalAlpha = 1;
}

export default function DockGenie() {
  const canvas = useRef(null);
  useEffect(() => {
    let disposed = false;
    let frame = 0;
    let currentId = 0;
    const surface = canvas.current;
    const context = surface.getContext("2d");
    const finish = (id) => invoke("dock_genie_finished", { id }).catch(() => {});

    async function play(animation) {
      if (!animation || disposed || animation.id <= currentId) return;
      currentId = animation.id;
      cancelAnimationFrame(frame);
      if (!context) { finish(animation.id); return; }
      context.clearRect(0, 0, surface.width, surface.height);
      const image = new Image();
      image.src = animation.image;
      try { await image.decode(); } catch { finish(animation.id); return; }
      if (disposed || currentId !== animation.id) return;
      surface.width = animation.width;
      surface.height = animation.height;
      context.imageSmoothingQuality = "high";
      const started = performance.now();
      function tick(now) {
        const progress = clamp((now - started) / animation.duration);
        draw(context, image, animation.source, animation.target, animation.restoring ? 1 - progress : progress);
        if (progress < 1) {
          frame = requestAnimationFrame(tick);
        } else {
          context.clearRect(0, 0, surface.width, surface.height);
          finish(animation.id);
        }
      }
      tick(started);
    }

    const subscription = listen("dock-genie", ({ payload }) => play(payload));
    subscription.then(() => invoke("dock_genie_state")).then(play).catch(console.error);
    return () => {
      disposed = true;
      cancelAnimationFrame(frame);
      subscription.then((unlisten) => unlisten()).catch(console.error);
    };
  }, []);

  return <canvas ref={canvas} className="dock-genie-canvas" aria-hidden="true" />;
}
