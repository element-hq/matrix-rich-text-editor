/*
Copyright 2024 New Vector Ltd.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

// @ts-check

/**
 * This is the entrypoint on non-node ESM environments.
 * `initAsync` will load the WASM module using a `fetch` call.
 */

import * as bindings from './pkg/wysiwyg_bg.js';

const moduleUrl = new URL(
    './pkg/wysiwyg_bg.wasm?url',
    import.meta.url,
);

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
    let mod;
    if (typeof WebAssembly.compileStreaming === 'function') {
        mod = await WebAssembly.compileStreaming(fetch(moduleUrl));
    } else {
        // Fallback to fetch and compile
        const response = await fetch(moduleUrl);
        if (!response.ok) {
            throw new Error(`Failed to fetch wasm module: ${moduleUrl}`);
        }
        const bytes = await response.arrayBuffer();
        mod = await WebAssembly.compile(bytes);
    }

    /** @type {{exports: typeof import("./pkg/wysiwyg_bg.wasm.d.ts")}} */
    // @ts-expect-error: Typescript doesn't know what the instance exports exactly
    const instance = await WebAssembly.instantiate(mod, {
        './wysiwyg_bg.js': bindings,
    });

    bindings.__wbg_set_wasm(instance.exports);
    instance.exports.__wbindgen_start();
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
