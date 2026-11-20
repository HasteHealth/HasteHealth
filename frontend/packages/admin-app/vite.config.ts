import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@haste-health/components/dist/index.css": path.resolve(
        import.meta.dirname,
        "../components/src/index.css",
      ),
      "@haste-health/components": path.resolve(
        import.meta.dirname,
        "../components/src/index.ts",
      ),
    },
  },
  server: {
    allowedHosts: [".localhost"],
    port: 3001, // Replace with your desired port number
    strictPort: true, // Optional: forces Vite to exit if the port is already in use
  },
});
