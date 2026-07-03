import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  server: {
    allowedHosts: [".localhost"],
    port: 3001, // Replace with your desired port number
    strictPort: true, // Optional: forces Vite to exit if the port is already in use
  },
});
