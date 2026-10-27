import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App.jsx";
import SettingsWindow from "./components/SettingsWindow.jsx";
import "./index.css";

// Una sola app, dos ventanas: la de Ajustes se abre con ?view=settings.
const view = new URLSearchParams(window.location.search).get("view") ?? "notch";
document.documentElement.dataset.view = view;

ReactDOM.createRoot(document.getElementById("root")).render(view === "settings" ? <SettingsWindow /> : <App />);
