import { defineConfig } from "vite";
import path from "path";
import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  // We develop inside a devcontainer, so the dev server has to listen on 0.0.0.0: Vite's default
  // (`localhost`) binds the container's loopback only, where nothing outside the container — the
  // editor's port forwarder included — can reach it.
  //
  // strictPort matters as much as host here. The port is referenced from outside by fixed number
  // (the editor forward, and `adb reverse tcp:5173 tcp:5173` for on-device testing — see
  // docs/ANDROID_DEBUGGING.md), so silently falling back to 5174 when 5173 is busy would leave those
  // pointing at nothing. Fail loudly instead.
  server: {
    host: true,
    port: 5173,
    strictPort: true,
  },
  // Same reasoning for `vite preview` (npm run preview), which defaults to localhost:4173.
  preview: {
    host: true,
    port: 4173,
    strictPort: true,
  },
  build: {
    outDir: "dist",
  },
});
