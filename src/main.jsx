import React from "react";
import ReactDOM from "react-dom/client";
import { emit } from "@tauri-apps/api/event";
import App from "./App.jsx";
import SettingsWindow from "./components/SettingsWindow.jsx";
import TopBar from "./components/topbar/TopBar.jsx";
import BarPanel from "./components/topbar/BarPanel.jsx";
import Dock from "./components/dock/Dock.jsx";
import Launchpad from "./components/launchpad/Launchpad.jsx";
import Spotlight from "./components/spotlight/Spotlight.jsx";
import "./index.css";

// Una sola app, varias ventanas nativas: cada vista mantiene su propio WebView transparente.
const view = new URLSearchParams(window.location.search).get("view") ?? "notch";
document.documentElement.dataset.view = view;

const content = view === "settings" ? <SettingsWindow /> : view === "topbar" ? <TopBar /> : view === "barpanel" ? <BarPanel /> : view === "dock" ? <Dock /> : view === "launchpad" ? <Launchpad /> : view === "spotlight" ? <Spotlight /> : <App />;
ReactDOM.createRoot(document.getElementById("root")).render(content);
emit("frontend-ready", view).catch(() => {});
