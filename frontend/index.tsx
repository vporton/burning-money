import ReactDOM from "react-dom";
import { createRoot } from "react-dom/client";
import { App } from "./src/App";

// const app = document.getElementById("app");
// ReactDOM.render(<App />, app);

const root = createRoot(document.getElementById("app"));
root.render(<App />);