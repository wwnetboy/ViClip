import ReactDOM from "react-dom/client";
import App from "./App";
import RadialMenu from "./components/RadialMenu";
import Toast from "./components/Toast";
import ImagePreview from "./components/ImagePreview";
import "./styles/index.css";
import "./i18n";
import { isMacOS } from "./utils";

// Disable browser default context menu for all windows
document.addEventListener("contextmenu", (e) => e.preventDefault());

// Add platform class for platform-specific CSS
if (isMacOS()) {
  document.documentElement.classList.add("macos");
}

const isRadialWindow = window.location.search.includes("radial=1");
const isToastWindow = window.location.search.includes("toast=1");
const isPreviewWindow = window.location.search.includes("preview=1");

const root = document.getElementById("root");
if (root) {
  ReactDOM.createRoot(root).render(
    isToastWindow ? <Toast /> : isRadialWindow ? <RadialMenu /> : isPreviewWindow ? <ImagePreview /> : <App />
  );
}
