import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";

// https://vitejs.dev/config/
export default defineConfig({
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
      "~assets": path.resolve(__dirname, "./src/assets"),
      "~components": path.resolve(__dirname, "./src/components"),
      "~pages": path.resolve(__dirname, "./src/pages"),
      "~router": path.resolve(__dirname, "./src/router"),
      "~store": path.resolve(__dirname, "./src/stores"),
    },
  },
  plugins: [react()],
  server: {
    proxy: {
      "/api": {
        target: "http://localhost:3000",
        changeOrigin: true,
        secure: false,
      },
      "/auth": {
        target: "http://localhost:3000",
        changeOrigin: true,
        secure: false,
      },
    },
  },
});
