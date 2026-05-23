import ReactDOM from "react-dom/client";
import App from "./App";
import RadialMenu from "./components/RadialMenu";
import Toast from "./components/Toast";
import ImagePreview from "./components/ImagePreview";
import "./styles/index.css";
import "./i18n";

const isRadialWindow = window.location.search.includes("radial=1");
const isToastWindow = window.location.search.includes("toast=1");
const isPreviewWindow = window.location.search.includes("preview=1");

ReactDOM.createRoot(document.getElementById("root")!).render(
  isToastWindow ? <Toast /> : isRadialWindow ? <RadialMenu /> : isPreviewWindow ? <ImagePreview /> : <App />
);
