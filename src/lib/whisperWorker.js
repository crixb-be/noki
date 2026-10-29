// Transcripción local con Whisper (transformers.js + ONNX en WASM). El modelo se descarga una vez de Hugging Face
// y queda en la caché del navegador; el audio nunca sale del equipo.
import { env, pipeline } from "@huggingface/transformers";
import ortMjs from "onnxruntime-web/ort-wasm-simd-threaded.mjs?url";
import ortWasm from "onnxruntime-web/ort-wasm-simd-threaded.wasm?url";

env.allowLocalModels = false;
env.backends.onnx.wasm.wasmPaths = { mjs: ortMjs, wasm: ortWasm };

const MODELS = { tiny: "onnx-community/whisper-tiny", base: "onnx-community/whisper-base" };
const loaded = {};

const load = (size) =>
  (loaded[size] ??= pipeline("automatic-speech-recognition", MODELS[size] ?? MODELS.tiny, {
    dtype: "q8",
    device: "wasm",
    progress_callback: (p) => p.status === "progress" && self.postMessage({ type: "progress", file: p.file, progress: p.progress }),
  }));

self.onmessage = async ({ data }) => {
  const { id, audio, language, model = "tiny" } = data;
  try {
    const run = await load(model);
    self.postMessage({ type: "working", id });
    const out = await run(audio, { language, task: "transcribe", chunk_length_s: 30, stride_length_s: 5 });
    self.postMessage({ type: "done", id, text: out.text.trim() });
  } catch (e) {
    delete loaded[model];
    self.postMessage({ type: "error", id, error: String(e?.message ?? e) });
  }
};
