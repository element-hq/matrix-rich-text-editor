/*
Copyright 2024 New Vector Ltd.
Copyright 2022 The Matrix.org Foundation C.I.C.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

import { initAsync } from '@vector-im/matrix-wysiwyg-wasm';

let initPromise: Promise<void> | null = null;

/**
 * Initialise the WASM module, or do nothing if it is already initialised.
 * Safe to call concurrently: all callers share the same underlying promise.
 */
export async function initOnce(): Promise<void> {
    if (!initPromise) {
        initPromise = initAsync();
    }
    return initPromise;
}
