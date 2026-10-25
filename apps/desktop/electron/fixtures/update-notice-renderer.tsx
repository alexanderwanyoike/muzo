import { createRoot } from "react-dom/client";
import App from "../../src/App";
import "@fontsource-variable/manrope";
import "../../src/themes.css";
import "../../src/styles.css";

const root = document.getElementById("root");
if (!root) throw new Error("Update fixture root required");
createRoot(root).render(<App />);
