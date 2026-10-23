/*
Copyright 2024 New Vector Ltd.
Copyright 2022 The Matrix.org Foundation C.I.C.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

import { initAsync, new_composer_model } from '@vector-im/matrix-wysiwyg-wasm';

import { extractActionStates, getInputFromKeyDown } from './event';
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

describe('getInputFromKeyDown', () => {
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
    ])('Should resolve %s from %o', (expected, input, userAgent?: string) => {
        if (userAgent) {
            mockUserAgent(userAgent);
        }

        const event = new KeyboardEvent('keydown', input);
        const model = new_composer_model();

        const result = getInputFromKeyDown(
            event,
            model,
            {} as FormattingFunctions,
            document.createElement('div'),
        );
        expect(result).toBe(expected);
    });

    it('Should offer other key presses to the inputEventProcessor', () => {
        const event = new KeyboardEvent('keydown', { key: 'a' });
        const model = new_composer_model();
        const editor = document.createElement('div');
        const inputEventProcessor = vi.fn((e) => e);

        const result = getInputFromKeyDown(
            event,
            model,
            {} as FormattingFunctions,
            editor,
            inputEventProcessor,
        );

        expect(result).toBeNull();
        expect(inputEventProcessor).toHaveBeenCalledWith(
            event,
            expect.anything(),
            editor,
        );
    });

    it('Should not call the inputEventProcessor without an editor', () => {
        const event = new KeyboardEvent('keydown', { key: 'a' });
        const model = new_composer_model();
        const inputEventProcessor = vi.fn((e) => e);

        getInputFromKeyDown(
            event,
            model,
            {} as FormattingFunctions,
            null,
            inputEventProcessor,
        );

        expect(inputEventProcessor).not.toHaveBeenCalled();
    });
});
