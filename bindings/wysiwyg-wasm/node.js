/*
Copyright 2024 New Vector Ltd.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

// @ts-check

/**
 * This is the entrypoint on node-compatible ESM environments.
 * `asyncLoad` will use `fs.readFile` to load the WASM module.
 */

import { fileURLToPath } from "node:url";
import { join, dirname } from "node:path";
import { readFileSync } from "node:fs";
import { readFile } from "node:fs/promises";
import * as bindings from "./pkg/wysiwyg_bg.js";

const filename = join(dirname(fileURLToPath(import.meta.url)), "pkg", "wysiwyg_bg.wasm");

// In node environments, we want to automatically load the WASM module
// synchronously if the consumer did not call `initAsync`. To do so, we install
// a `Proxy` that will intercept calls to the WASM module.
bindings.__wbg_set_wasm(
    new Proxy(
        {},
        {
            get(_target, prop) {
                const instance = loadModuleSync();
                return instance[prop];
            },
        },
    ),
);

/**
 * Stores a promise of the `loadModule` call
 * @type {Promise<void> | null}
 */
let modPromise = null;

/**
 * Tracks whether the module has been instantiated or not
 * @type {boolean}
 */
let initialised = false;

/**
 * Loads and instantiates the WASM module synchronously
 *
 * It will throw if there is an attempt to load the module asynchronously running
 *
 * @returns {typeof import("./pkg/wysiwyg_bg.wasm.d")}
 */
function loadModuleSync() {
  if (modPromise) throw new Error("The WASM module is being loaded asynchronously but hasn't finished");
  const bytes = readFileSync(filename);
  const mod = new WebAssembly.Module(bytes);

  const instance = new WebAssembly.Instance(mod, {
    // @ts-expect-error: The bindings don't exactly match the 'ExportValue' type
    "./wysiwyg_bg.js": bindings,
  });

  initInstance(instance);

  // @ts-expect-error: Typescript doesn't know what the instance exports exactly
  return instance.exports;
}

/**
 * Loads the WASM module asynchronously
 *
 * @returns {Promise<void>}
 */
async function loadModuleAsync() {
  const bytes = await readFile(filename);
  const { instance } = await WebAssembly.instantiate(bytes, {
    // @ts-expect-error: The bindings don't exactly match the 'ExportValue' type
    "./wysiwyg_bg.js": bindings,
  });

  initInstance(instance);

  // @ts-expect-error: Typescript doesn't know what the instance exports exactly
  return instance.exports;
}

/**
 * Initializes the WASM module and returns the exports from the WASM module.
 *
 * @param {WebAssembly.Instance} instance
 */
function initInstance(instance) {
  if (initialised) throw new Error("initInstance called twice");
  bindings.__wbg_set_wasm(instance.exports);
  // @ts-expect-error: Typescript doesn't know what the instance exports exactly
  instance.exports.__wbindgen_start();
  initialised = true;
}

/**
 * Load the WebAssembly module in the background, if it has not already been loaded.
 *
 * Returns a promise which will resolve once the other methods are ready.
 *
 * @returns {Promise<void>}
 */
export async function initAsync() {
    if (initialised) return;
    if (!modPromise) modPromise = loadModuleAsync();
    await modPromise;
}

export * from "./pkg/wysiwyg_bg.js";
