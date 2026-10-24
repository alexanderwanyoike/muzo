import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "@fontsource-variable/manrope";
import "./themes.css";
import "./styles.css";
import { applyAppearance, DEFAULT_APPEARANCE, loadAppearance } from "./appearance";

const savedAppearance = loadAppearance();
applyAppearance(savedAppearance.ok ? savedAppearance.value : DEFAULT_APPEARANCE);

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
