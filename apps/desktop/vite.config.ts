import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  build: {
    // PDF.js worker is emitted as a standalone lazy-loaded chunk around 2.1 MB.
    // Keep the warning limit aligned to that known worker ceiling while vendor
    // and syntax-highlighting code are split below.
    chunkSizeWarningLimit: 2300,
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (!id.includes("node_modules")) return undefined;
          if (id.includes("pdfjs-dist")) return "vendor-pdf";
          if (id.includes("xlsx")) return "vendor-xlsx";
          if (id.includes("mammoth")) return "vendor-docx";
          if (id.includes("@shikijs/themes")) return "vendor-shiki-theme";
          if (id.includes("@shikijs/core") || id.includes("@shikijs/types") || id.includes("/shiki/")) return "vendor-shiki-core";
          if (id.includes("react") || id.includes("scheduler")) return "vendor-react";
          return undefined;
        },
      },
    },
  },
  server: {
    host: "127.0.0.1",
    port: 1420,
    strictPort: true,
  },
});
