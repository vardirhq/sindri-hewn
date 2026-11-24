import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// GitHub Pages hosts this repository below /sindri-hewn/.
// Local development still uses the site root.
export default defineConfig({
  plugins: [react()],
  base: process.env.GITHUB_PAGES === "true" ? "/sindri-hewn/" : "/",
});
