import { svelte } from '@sveltejs/vite-plugin-svelte'
import { defineConfig } from 'vite'

// https://vite.dev/config/
export default defineConfig({
  plugins: [svelte()],
  // Tauri's devUrl (tauri.conf.json) expects this port; fail loudly instead of picking another.
  server: {
    port: 1420,
    strictPort: true,
  },
})
