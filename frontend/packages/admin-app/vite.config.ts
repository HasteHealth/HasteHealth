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
    host: true,
    allowedHosts: [".localhost", "localhost"],
    port: 3001,
    strictPort: true,
  },
});
