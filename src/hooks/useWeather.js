import { useCallback, useEffect, useRef, useState } from "react";
import { useStore } from "./useStore.js";

const REFRESH_MS = 30 * 60 * 1000;

export function weatherUrl({ lat, lon, units }) {
  const q = new URLSearchParams({
    latitude: lat,
    longitude: lon,
    current: "temperature_2m,apparent_temperature,relative_humidity_2m,weather_code,wind_speed_10m,is_day,surface_pressure",
    hourly: "temperature_2m,weather_code,precipitation_probability",
    daily: "weather_code,temperature_2m_max,temperature_2m_min,precipitation_probability_max,sunrise,sunset,uv_index_max",
    timezone: "auto",
    forecast_days: 7,
    temperature_unit: units === "f" ? "fahrenheit" : "celsius",
    wind_speed_unit: units === "f" ? "mph" : "kmh",
  });
  return `https://api.open-meteo.com/v1/forecast?${q}`;
}

/** Clima de Open-Meteo (gratuito, sin clave). Se guarda la última respuesta para mostrarla sin esperar. */
export function useWeather({ weather, enabled, leader = true }) {
  const [cache, setCache] = useStore("weatherCache", { key: "", at: 0, data: null });
  const [error, setError] = useState(null);
  const [loading, setLoading] = useState(false);
  const latest = useRef(cache);
  latest.current = cache;

  const configured = Boolean(weather.city) && !(weather.lat === 0 && weather.lon === 0);
  // El «2» invalida las respuestas guardadas antes de añadir presión, UV y salida/puesta del sol.
  const key = `${weather.lat},${weather.lon},${weather.units},2`;

  const refresh = useCallback(async () => {
    if (!configured) return;
    setLoading(true);
    try {
      const res = await fetch(weatherUrl(weather));
      if (!res.ok) throw new Error(`Error ${res.status}`);
      const data = await res.json();
      setCache({ key, at: Date.now(), data });
      setError(null);
    } catch {
      setError("No se pudo obtener el clima. Revisa tu conexión.");
    } finally {
      setLoading(false);
    }
  }, [configured, key]); // eslint-disable-line react-hooks/exhaustive-deps

  useEffect(() => {
    if (!enabled || !configured || !leader) return;
    if (latest.current.key !== key || Date.now() - latest.current.at > REFRESH_MS) refresh();
    const id = setInterval(refresh, REFRESH_MS);
    return () => clearInterval(id);
  }, [enabled, configured, leader, key, refresh]);

  // Los datos guardados de otra ciudad/unidad no se muestran.
  const data = cache.key === key ? cache.data : null;
  return { configured, data, error, loading, refresh, updatedAt: cache.key === key ? cache.at : 0 };
}
