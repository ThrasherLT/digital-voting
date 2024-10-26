import init, * as bindings from './dist/client.js';
const wasm = await init('./dist/client_bg.wasm');

window.wasmBindings = bindings;

dispatchEvent(new CustomEvent("TrunkApplicationStarted", {detail: {wasm}}));
