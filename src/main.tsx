import { StrictMode } from "react";
import ReactDOM from "react-dom/client";
import "./index.css";
import "@fortawesome/fontawesome-free/css/all.min.css";
import { App } from "./App";

const rootElement = document.getElementById("root");
if (rootElement === null) {
  throw new Error("ルート要素 #root が見つかりません");
}

ReactDOM.createRoot(rootElement).render(
  <StrictMode>
    <App />
  </StrictMode>
);
