import Aurelia from 'aurelia';
import { loadCore } from './core';
import { MyApp } from './my-app';
import './styles.css';

// Instantiate the WebAssembly module before any component can call into it.
await loadCore();

await Aurelia
  .app(MyApp)
  .start();
