/*
Copyright 2024 New Vector Ltd.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

// @ts-check

/**
 * This is the entrypoint on non-node ESM environments which support the ES Module Integration Proposal for WebAssembly [1]
 * (such as Element Web).
 *
 * [1]: https://github.com/webassembly/esm-integration
 */

import * as bindings from './pkg/wysiwyg_bg.js';

// We want to throw an error if the user tries to use the bindings before
// calling `initAsync`.
bindings.__wbg_set_wasm(
    new Proxy(
        {},
        {
            get() {
                throw new Error(
                    '@element-hq/matrix-wysiwyg was used before it was initialized. Call `initAsync` first.',
                );
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
 * Loads the WASM module asynchronously
 *
 * @returns {Promise<void>}
 */
async function loadModule() {
    const wasm = await import('./pkg/wysiwyg_bg.wasm');
    bindings.__wbg_set_wasm(wasm);
    wasm.__wbindgen_start();
}

/**
 * Load the WebAssembly module in the background, if it has not already been loaded.
 *
 * Returns a promise which will resolve once the other methods are ready.
 *
 * @returns {Promise<void>}
 */
export async function initAsync() {
    if (!modPromise) modPromise = loadModule();
    await modPromise;
}

// Re-export everything from the generated javascript wrappers
export * from './pkg/wysiwyg_bg.js';
