# au2-rust

A starter for building apps with an Aurelia 2 frontend and a Rust backend.

The interesting part is `crates/core`. It holds the app's types and validation rules, and it gets compiled twice: natively into the Axum server, and to WebAssembly for the browser. The form validates as you type using the same Rust function the server runs when the note arrives, so the two can't disagree. The TypeScript types for the API payloads are generated from the Rust structs too, so there's no hand-written `interface Note` to keep in sync.

The demo is a small notes app. It's there to show every piece connected; delete it once you've seen how they fit.

```
crates/
  core/     Shared types and rules. Plain Rust, no web dependencies by default.
  server/   Axum JSON API under /api. Serves the built frontend in production.
  wasm/     wasm-bindgen wrapper that exposes core to JavaScript.
web/
  src/      Aurelia 2 app (Vite, TypeScript).
  test/     Vitest + @aurelia/testing, running against the real .wasm.
  scripts/  build-wasm.mjs: cargo build + wasm-bindgen.
  wasm/     Generated JS glue, .d.ts and .wasm (git-ignored).
```

## Requirements

- Rust (stable). `rust-toolchain.toml` asks rustup for the `wasm32-unknown-unknown` target, clippy and rustfmt.
- The wasm-bindgen CLI, at the exact version in `Cargo.lock`:

  ```sh
  cargo install wasm-bindgen-cli --version 0.2.129 --locked
  ```

  `npm run wasm` checks this and prints the right command if your installed version doesn't match.
- Node.js 22.12 or newer.

## Development

Two terminals:

```sh
# 1. API on http://127.0.0.1:3000
cargo run

# 2. Frontend on http://localhost:9000
cd web
npm install
npm run dev
```

Vite proxies `/api` to the Rust server. Edit anything in `crates/core` or `crates/wasm` while `npm run dev` is running and the dev server rebuilds the WebAssembly module and reloads the page. Changes to `crates/server` need a restart of `cargo run` (or use [cargo-watch](https://crates.io/crates/cargo-watch) / [bacon](https://dystroy.org/bacon/) if you want that automated).

If port 3000 is taken, run the server with `BIND_ADDR=127.0.0.1:3100 cargo run` and start Vite with `API_URL=http://127.0.0.1:3100 npm run dev`.

## How the WebAssembly step works

There's no wasm-pack here. It's two commands, and `web/scripts/build-wasm.mjs` runs them:

1. `cargo build -p au2-rust-wasm --target wasm32-unknown-unknown --release` compiles the wasm crate. The `#[wasm_bindgen]` attributes embed a description of every exported function into the `.wasm` file.
2. `wasm-bindgen --target web --out-dir web/wasm --out-name core <file>.wasm` reads that description and writes `core.js` (an ES module that loads the `.wasm`), `core.d.ts` and `core_bg.wasm`.

The CLI and the `wasm-bindgen` crate have to be the same version because the embedded format changes between releases. That's why the crate is pinned with `=` in the root `Cargo.toml`. To upgrade, change the pin, run `cargo update -p wasm-bindgen`, and install the matching CLI.

`--target web` is the output that suits Vite best: `core.js` loads the binary with `new URL('core_bg.wasm', import.meta.url)`, which Vite already understands, so it needs no plugin. `src/main.ts` awaits the module before starting Aurelia, which means components can call into Rust synchronously.

Types cross the boundary through [tsify](https://crates.io/crates/tsify). Structs in `core` derive `Tsify` behind the `typescript` feature, which only the wasm crate turns on, so the server never compiles wasm-bindgen. Functions take and return `Ts<T>` and convert inside the function body, so a malformed value from JavaScript comes back as an ordinary `Result` error. The older `from_wasm_abi` approach converts inside wasm-bindgen's glue, where a failure can only throw and leaks memory on the way out.

### Why share code at all?

Here's one concrete example from the demo. The title limit is 80 characters. In JavaScript, `"🦀".length` is 2, because strings are UTF-16. In Rust, `"🦀".chars().count()` is 1. Write the check twice and a title full of emoji passes on one side and fails on the other. Write it once in Rust and run it in both places, and the bug can't exist. The tests in `crates/core` and `web/test` both cover this case.

The rule of thumb: anything both sides need to agree on (validation, parsing, formatting, pricing, permission checks) goes in `core`. `wasm` stays a thin layer of conversions.

## Production

```sh
cd web && npm run build && cd ..
cargo run --release
```

`npm run build` builds the release `.wasm`, type-checks, and writes `web/dist`. The server serves `web/dist` when it finds `index.html` there, with `index.html` as the fallback for any path that isn't a file, so client-side routes survive a reload. Unknown `/api` paths return a JSON 404 rather than the HTML page.

Environment variables:

| Variable     | Default          | Purpose                                  |
| ------------ | ---------------- | ---------------------------------------- |
| `BIND_ADDR`  | `127.0.0.1:3000` | Address the server listens on.           |
| `STATIC_DIR` | `web/dist`       | Built frontend. Skipped if missing.      |
| `RUST_LOG`   | `au2_rust_server=info,tower_http=info` | Log filter. `tower_http=debug` logs each request. |

### Docker

```sh
docker build -t au2-rust .
docker run --rm -p 3000:3000 au2-rust
```

The image is three stages: Rust builds the server and the `.wasm`, Node builds the frontend against it, and the final `debian:trixie-slim` image holds the binary and the static files. If you bump wasm-bindgen, update `WASM_BINDGEN_VERSION` in the `Dockerfile` too.

## Tests

```sh
cargo test          # core rules and API integration tests
cd web && npm test  # component tests in jsdom
```

The web tests load the real `core_bg.wasm` with `initSync` in `test/setup.ts`. Nothing about validation is mocked; only the HTTP layer is swapped out, by passing a `save` function into the form.

CI (`.github/workflows/ci.yml`) runs rustfmt, clippy for both the native and wasm targets, both test suites, and a production build.

## Making it yours

- Rename the crates (`au2-rust-*`) in each `Cargo.toml`, then update the crate name in `web/scripts/build-wasm.mjs` and the `Dockerfile`.
- Replace `NoteStore` in `crates/server/src/notes.rs` with a database. The handlers only use `list`, `insert` and `remove`.
- The frontend has no router. Add `@aurelia/router` when you need more than one page; the server's `index.html` fallback is already in place for it.
- `web/src/styles.css` is deliberately plain. Throw it away.

## License

MIT
