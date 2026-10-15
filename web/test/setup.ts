import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { BrowserPlatform } from '@aurelia/platform-browser';
import { setPlatform, onFixtureCreated, type IFixture } from '@aurelia/testing';
import { beforeAll, afterEach } from 'vitest';
import { initSync } from '../wasm/core.js';

// Load the real WebAssembly module. Tests exercise the same Rust code the
// browser and the server run, not a mock of it.
initSync({ module: readFileSync(join(import.meta.dirname, '../wasm/core_bg.wasm')) });

function bootstrapTestEnv() {
  const platform = new BrowserPlatform(window);
  setPlatform(platform);
  BrowserPlatform.set(globalThis, platform);
}

const fixtures: IFixture<object>[] = [];

beforeAll(() => {
  bootstrapTestEnv();
  onFixtureCreated(fixture => {
    fixtures.push(fixture);
  });
});

afterEach(async () => {
  for (const fixture of fixtures) {
    try {
      await fixture.stop(true);
    } catch {
      // Already stopped by the test.
    }
  }
  fixtures.length = 0;
});
