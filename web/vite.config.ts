import { fileURLToPath } from 'node:url';
import { defineConfig, type Plugin } from 'vite';
import aurelia from '@aurelia/vite-plugin';
import { buildWasm } from './scripts/build-wasm.mjs';

export default defineConfig({
  server: {
    open: !process.env.CI,
    port: 9000,
    proxy: {
      // The Axum server (`cargo run`) listens here in development.
      '/api': process.env.API_URL ?? 'http://127.0.0.1:3000',
    },
  },
  build: {
    target: 'es2022',
  },
  plugins: [
    // Development builds of Aurelia (readable errors) for `vite`, production
    // builds for `vite build`.
    aurelia(),
    rebuildWasmOnRustChange(),
  ],
});

/**
 * Rebuilds web/wasm/ when Rust sources in crates/core or crates/wasm change.
 * Vite sees the regenerated module and reloads the page.
 */
function rebuildWasmOnRustChange(): Plugin {
  const crates = ['core', 'wasm'].map(name => fileURLToPath(new URL(`../crates/${name}`, import.meta.url)));

  return {
    name: 'au2-rust:wasm-watch',
    // Dev server only. Vitest also starts a Vite server; it builds once up front instead.
    apply: (_, { command }) => command === 'serve' && !process.env.VITEST,
    configureServer(server) {
      const { logger } = server.config;
      let building = false;
      let pending = false;

      const rebuild = async () => {
        if (building) {
          pending = true;
          return;
        }
        building = true;
        do {
          pending = false;
          try {
            await buildWasm({ dev: true });
            logger.info('wasm rebuilt', { timestamp: true });
          } catch (error) {
            logger.error(String(error), { timestamp: true });
          }
        } while (pending);
        building = false;
      };

      server.watcher.add(crates);
      server.watcher.on('change', file => {
        if (crates.some(dir => file.startsWith(dir)) && /\.(rs|toml)$/.test(file)) {
          void rebuild();
        }
      });
    },
  };
}
