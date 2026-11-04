/// <reference types="vitest/config" />
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// Tauri expects a fixed dev-server port and a host reachable from the webview.
const TAURI_DEV_PORT = 5173;

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: TAURI_DEV_PORT,
    strictPort: true,
    host: '0.0.0.0',
    // Allow the webview/preview proxy to reach us from other origins.
    allowedHosts: true,
    fs: {
      // shared fixtures live at the repository root
      allow: ['..'],
    },
  },
  envPrefix: ['VITE_', 'TAURI_ENV_'],
  build: {
    target: 'es2021',
    outDir: 'dist',
    sourcemap: true,
  },
  test: {
    environment: 'node',
    include: ['src/**/*.test.ts', 'src/**/*.test.tsx'],
  },
});
