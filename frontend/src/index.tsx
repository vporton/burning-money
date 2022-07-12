import React from "react";
import ReactDOM from "react-dom";
import { createRoot } from "react-dom/client";
import { App } from "./App";

// const app = document.getElementById("app");
// ReactDOM.render(<App />, app);

const root = createRoot(document.getElementById("app"));
root.render(<App />);