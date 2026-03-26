/*
Copyright 2024 New Vector Ltd.
Copyright 2022 The Matrix.org Foundation C.I.C.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
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
        return new Response(new Uint8Array(await fs.readFile(wasmPath)), {
            headers: { 'Content-Type': 'application/wasm' },
        });
    } else {
        throw new Error('fetch is not defined');
    }
};

// Work around missing ClipboardEvent type
class MyClipboardEvent {}

// @ts-ignore
globalThis.ClipboardEvent = MyClipboardEvent as unknown as ClipboardEvent;

// jsdom 26 added selection.collapse(element, 0) inside focus(), which resets
// the cursor to position 0 whenever focus moves to a new element. Real browsers
// preserve the selection when returning focus to a contenteditable element (e.g.
// after clicking a toolbar button). Patch HTMLElement.prototype.focus to save
// and restore the selection around the native call.
// See: https://github.com/jsdom/jsdom/issues/3825
const originalFocus = HTMLElement.prototype.focus;
HTMLElement.prototype.focus = function (this: HTMLElement, ...args): void {
    const sel = document.getSelection();
    const range =
        sel && sel.rangeCount > 0 ? sel.getRangeAt(0).cloneRange() : null;

    originalFocus.apply(this, args);

    if (range && sel) {
        sel.removeAllRanges();
        sel.addRange(range);
    }
};

afterEach(() => {
    cleanup();
});
