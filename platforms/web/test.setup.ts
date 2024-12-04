/*
Copyright 2024 New Vector Ltd.
Copyright 2022 The Matrix.org Foundation C.I.C.

SPDX-License-Identifier: AGPL-3.0-only
Please see LICENSE in the repository root for full details.
*/

import '@testing-library/jest-dom';
import { cleanup } from '@testing-library/react';
import fs from 'node:fs/promises';
import path from 'node:path';

globalThis.fetch = async (url): Promise<Response> => {
    // wysiwyg.js binding uses fetch to get the wasm file
    // we return manually here the wasm file
    if (url instanceof URL && url.href.includes('wysiwyg_bg.wasm')) {
        const wasmPath = path.resolve(
            __dirname,
            '..',
            '..',
            'bindings',
            'wysiwyg-wasm',
            'pkg',
            'wysiwyg_bg.wasm',
        );
        return new Response(await fs.readFile(wasmPath), {
            headers: { 'Content-Type': 'application/wasm' },
        });
    } else {
        throw new Error('fetch is not defined');
    }
};

// Work around missing ClipboardEvent type
class MyClipboardEvent {}
// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore
globalThis.ClipboardEvent = MyClipboardEvent as unknown as ClipboardEvent;

afterEach(() => {
    cleanup();
});
