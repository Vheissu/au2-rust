// The Rust crate in crates/core, compiled to WebAssembly by `npm run wasm`.
// Types come from the generated declarations, so a field added to a Rust
// struct shows up here (and as a type error wherever it is missing).
import init from '../wasm/core.js';

export { charCount, limits, slugify, validateDraft } from '../wasm/core.js';
export type { Field, FieldError, Limits, Note, NoteDraft } from '../wasm/core.js';

/** Fetches and instantiates the module. Call once before using the exports. */
export async function loadCore(): Promise<void> {
  await init();
}
