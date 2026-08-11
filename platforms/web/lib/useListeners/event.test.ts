/*
Copyright 2024 New Vector Ltd.
Copyright 2022 The Matrix.org Foundation C.I.C.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

import { initAsync, new_composer_model } from '@vector-im/matrix-wysiwyg-wasm';

import { extractActionStates, handleKeyDown } from './event';
import { type FormatBlockEvent } from './types';
import { type FormattingFunctions } from '../types';
import { WINDOWS_UA, mockUserAgent } from '../utils.test';

beforeAll(initAsync);

describe('getFormattingState', () => {
    it('Should be a map of action to state', () => {
        // Given
        const model = new_composer_model();
        const menuStateUpdate = model.bold().menu_state().update();

        // When
        if (!menuStateUpdate) {
            fail('There should be an update!');
        }
        const states = extractActionStates(menuStateUpdate);

        // Then
        expect(states.italic).toBe('enabled');
        expect(states.bold).toBe('reversed');
        expect(states.redo).toBe('disabled');
    });
});

describe('handleKeyDown', () => {
    let originalUserAgent = '';

    beforeAll(() => {
        originalUserAgent = navigator.userAgent;
    });

    afterAll(() => {
        mockUserAgent(originalUserAgent);
    });

    it.each([
        ['formatBold', { ctrlKey: true, key: 'b' }],
        ['formatBold', { metaKey: true, key: 'b' }],
        ['formatItalic', { ctrlKey: true, key: 'i' }],
        ['formatItalic', { metaKey: true, key: 'i' }],
        ['formatUnderline', { ctrlKey: true, key: 'u' }],
        ['formatUnderline', { metaKey: true, key: 'u' }],
        ['historyRedo', { ctrlKey: true, key: 'y' }],
        ['historyRedo', { metaKey: true, key: 'y' }],
        ['historyRedo', { ctrlKey: true, key: 'Z' }],
        ['historyRedo', { metaKey: true, key: 'Z' }],
        ['historyUndo', { ctrlKey: true, key: 'z' }],
        ['historyUndo', { metaKey: true, key: 'z' }],
        ['sendMessage', { ctrlKey: true, key: 'Enter' }],
        ['sendMessage', { metaKey: true, key: 'Enter' }],
        ['formatStrikeThrough', { shiftKey: true, altKey: true, key: '5' }],
        ['deleteWordBackward', { ctrlKey: true, key: 'Backspace' }, WINDOWS_UA],
    ])(
        'Should dispatch %s when %o',
        async (expected, input, userAgent?: string) => {
            if (userAgent) {
                mockUserAgent(userAgent);
            }

            const elem = document.createElement('input');
            const event = new KeyboardEvent('keydown', input);

            const result = new Promise((resolve) => {
                elem.addEventListener('wysiwygInput', (({
                    detail: { blockType },
                }: FormatBlockEvent) => {
                    resolve(blockType);
                }) as EventListener);
            });

            const model = new_composer_model();

            handleKeyDown(event, elem, model, {} as FormattingFunctions);
            expect(await result).toBe(expected);
        },
    );

    describe('table cell navigation', () => {
        function selectInsideCell(cell: HTMLElement): void {
            const range = document.createRange();
            range.selectNodeContents(cell);
            const selection = document.getSelection();
            selection?.removeAllRanges();
            selection?.addRange(range);
        }

        it('dispatches moveToNextCell for Tab inside a table cell', async () => {
            const editor = document.createElement('div');
            editor.innerHTML = '<table><tr><td>a</td><td>b</td></tr></table>';
            document.body.appendChild(editor);
            const cell = editor.querySelector('td')!;
            selectInsideCell(cell);

            const event = new KeyboardEvent('keydown', { key: 'Tab' });
            const result = new Promise((resolve) => {
                editor.addEventListener('wysiwygInput', (({
                    detail: { blockType },
                }: FormatBlockEvent) => {
                    resolve(blockType);
                }) as EventListener);
            });

            const model = new_composer_model();
            handleKeyDown(event, editor, model, {} as FormattingFunctions);

            expect(await result).toBe('moveToNextCell');
            editor.remove();
        });

        it('dispatches moveToPreviousCell for Shift+Tab inside a table cell', async () => {
            const editor = document.createElement('div');
            editor.innerHTML = '<table><tr><td>a</td><td>b</td></tr></table>';
            document.body.appendChild(editor);
            const cell = editor.querySelectorAll('td')[1]!;
            selectInsideCell(cell);

            const event = new KeyboardEvent('keydown', {
                key: 'Tab',
                shiftKey: true,
            });
            const result = new Promise((resolve) => {
                editor.addEventListener('wysiwygInput', (({
                    detail: { blockType },
                }: FormatBlockEvent) => {
                    resolve(blockType);
                }) as EventListener);
            });

            const model = new_composer_model();
            handleKeyDown(event, editor, model, {} as FormattingFunctions);

            expect(await result).toBe('moveToPreviousCell');
            editor.remove();
        });

        it('does not intercept Tab outside of a table cell', () => {
            const editor = document.createElement('div');
            editor.innerHTML = '<p>a</p>';
            document.body.appendChild(editor);
            const paragraph = editor.querySelector('p')!;
            selectInsideCell(paragraph);

            const event = new KeyboardEvent('keydown', { key: 'Tab' });
            let dispatched = false;
            editor.addEventListener('wysiwygInput', () => {
                dispatched = true;
            });

            const model = new_composer_model();
            handleKeyDown(event, editor, model, {} as FormattingFunctions);

            expect(dispatched).toBe(false);
            editor.remove();
        });
    });
});
