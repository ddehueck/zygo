import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { RouterProvider } from "@tanstack/react-router";
import { loadSnapshot } from "./db/snapshot";
import { startSync } from "./features/sync/sync-ipc";
import { getRouter } from "./router";

const router = getRouter();

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <RouterProvider router={router} />
  </StrictMode>,
);

async function initializeData() {
  try {
    await loadSnapshot();
  } catch (error) {
    console.error("initial snapshot load failed", error);
  }

  // The sync command stays open for the lifetime of the app.
  void startSync();
}

void initializeData();
