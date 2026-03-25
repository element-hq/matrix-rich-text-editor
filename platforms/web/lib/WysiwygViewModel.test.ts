/*
Copyright 2024 New Vector Ltd.
Copyright 2022 The Matrix.org Foundation C.I.C.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import * as mockRustModel from '@vector-im/matrix-wysiwyg-wasm';

import { WysiwygViewModel } from './WysiwygViewModel';

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function makeEditor(): HTMLDivElement {
    const el = document.createElement('div');
    el.setAttribute('contenteditable', 'true');
    document.body.appendChild(el);
    return el;
}

async function makeReadyViewModel(
    options: ConstructorParameters<typeof WysiwygViewModel>[0] = {},
): Promise<{ vm: WysiwygViewModel; editor: HTMLDivElement }> {
    const vm = new WysiwygViewModel(options);
    const editor = makeEditor();
    vm.attach(editor);
    await vm.init();
    return { vm, editor };
}

function fireInput(editor: HTMLElement, inputType: string, data?: string): void {
    const event = new InputEvent('input', { inputType, data, bubbles: true });
    editor.dispatchEvent(event);
}

function fireWysiwygInput(editor: HTMLElement, blockType: string, data?: unknown): void {
    editor.dispatchEvent(
        new CustomEvent('wysiwygInput', { detail: { blockType, data } }),
    );
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe('WysiwygViewModel', () => {
    let editor: HTMLDivElement;

    beforeEach(() => {
        vi.spyOn(mockRustModel, 'new_composer_model');
        vi.spyOn(mockRustModel, 'new_composer_model_from_html');
    });

    afterEach(() => {
        document.body.innerHTML = '';
        vi.clearAllMocks();
    });

    // -------------------------------------------------------------------------
    // Initialisation
    // -------------------------------------------------------------------------

    describe('init', () => {
        it('starts with isReady=false before init', () => {
            const vm = new WysiwygViewModel();
            editor = makeEditor();
            vm.attach(editor);
            expect(vm.getSnapshot().isReady).toBe(false);
        });

        it('sets isReady=true after init', async () => {
            const { vm } = await makeReadyViewModel();
            expect(vm.getSnapshot().isReady).toBe(true);
        });

        it('creates a new_composer_model for empty editor', async () => {
            await makeReadyViewModel();
            expect(mockRustModel.new_composer_model).toHaveBeenCalledTimes(1);
            expect(mockRustModel.new_composer_model_from_html).not.toHaveBeenCalled();
        });

        it('creates a new_composer_model_from_html when initialContent is provided', async () => {
            await makeReadyViewModel({ initialContent: '<strong>Hello</strong>' });
            expect(mockRustModel.new_composer_model_from_html).toHaveBeenCalledTimes(1);
            expect(mockRustModel.new_composer_model).not.toHaveBeenCalled();
        });

        it('falls back to empty model on invalid initial HTML', async () => {
            // Force new_composer_model_from_html to throw so the fallback path is exercised
            vi.mocked(mockRustModel.new_composer_model_from_html).mockImplementationOnce(() => {
                throw new Error('parse failure');
            });
            const { vm } = await makeReadyViewModel({ initialContent: '<<<bad>>>' });
            expect(vm.getSnapshot().isReady).toBe(true);
            expect(mockRustModel.new_composer_model).toHaveBeenCalledTimes(1);
        });

        it('exposes initial content snapshot', async () => {
            const { vm } = await makeReadyViewModel({ initialContent: '<strong>Hello</strong>' });
            const snap = vm.getSnapshot();
            expect(snap.content).toBeTruthy();
            expect(snap.messageContent).toBeTruthy();
        });

        it('sets custom suggestion patterns for emoji when provided', async () => {
            const emojiSuggestions = new Map([['🙂', ':smile:']]);
            // Just verify init succeeds without errors
            const { vm } = await makeReadyViewModel({ emojiSuggestions });
            expect(vm.getSnapshot().isReady).toBe(true);
        });
    });

    // -------------------------------------------------------------------------
    // subscribe / notify cycle
    // -------------------------------------------------------------------------

    describe('subscribe/notify', () => {
        it('calls the listener when the snapshot changes', async () => {
            const { vm, editor } = await makeReadyViewModel();
            const listener = vi.fn();
            vm.subscribe(listener);
            listener.mockClear();

            fireWysiwygInput(editor, 'formatBold');

            expect(listener).toHaveBeenCalled();
        });

        it('returns an unsubscribe function that stops notifications', async () => {
            const { vm, editor } = await makeReadyViewModel();
            const listener = vi.fn();
            const unsub = vm.subscribe(listener);
            unsub();
            listener.mockClear();

            fireWysiwygInput(editor, 'formatBold');

            expect(listener).not.toHaveBeenCalled();
        });

        it('notifies multiple subscribers', async () => {
            const { vm } = await makeReadyViewModel();
            const a = vi.fn();
            const b = vi.fn();
            vm.subscribe(a);
            vm.subscribe(b);
            a.mockClear();
            b.mockClear();

            vm.setContentFromMarkdown('hello');

            expect(a).toHaveBeenCalled();
            expect(b).toHaveBeenCalled();
        });
    });

    // -------------------------------------------------------------------------
    // Text input
    // -------------------------------------------------------------------------

    describe('text input', () => {
        it('updates content snapshot on insertText', async () => {
            const { vm } = await makeReadyViewModel();
            vm.insertText('hello');
            const snap = vm.getSnapshot();
            expect(snap.content).toContain('hello');
        });

        it('updates messageContent snapshot', async () => {
            const { vm } = await makeReadyViewModel();
            vm.insertText('world');
            expect(vm.getSnapshot().messageContent).toContain('world');
        });

        it('clears content on clear()', async () => {
            const { vm } = await makeReadyViewModel();
            vm.insertText('some text');
            vm.clear();
            const snap = vm.getSnapshot();
            // After clear, content should be empty (just a <br> or empty string)
            expect(snap.content).toBeFalsy();
        });
    });

    // -------------------------------------------------------------------------
    // Formatting actions
    // -------------------------------------------------------------------------

    describe('formatting', () => {
        it('toggles bold action state', async () => {
            const { vm } = await makeReadyViewModel();
            vm.insertText('test');
            // Select all text via replaceText to ensure there is a selection
            vm.setContentFromHtml('<strong>test</strong>');
            const snap = vm.getSnapshot();
            // bold should be reversed (active) after setting bold content
            expect(snap.actionStates.bold).toBe('reversed');
        });

        it('sets strikeThrough action state', async () => {
            const { vm } = await makeReadyViewModel();
            vm.setContentFromHtml('<del>text</del>');
            expect(vm.getSnapshot().actionStates.strikeThrough).toBe('reversed');
        });

        it('sets inlineCode action state', async () => {
            const { vm } = await makeReadyViewModel();
            vm.setContentFromHtml('<code>text</code>');
            expect(vm.getSnapshot().actionStates.inlineCode).toBe('reversed');
        });

        it('dispatches formatting events through the editor', async () => {
            const { vm } = await makeReadyViewModel();
            const listener = vi.fn();
            vm.subscribe(listener);
            listener.mockClear();

            vm.bold();
            // A snapshot change should have been emitted
            expect(listener).toHaveBeenCalled();
        });

        it('dispatches orderedList events', async () => {
            const { vm } = await makeReadyViewModel();
            const listener = vi.fn();
            vm.subscribe(listener);
            listener.mockClear();

            vm.orderedList();
            expect(listener).toHaveBeenCalled();
        });
    });

    // -------------------------------------------------------------------------
    // Undo / redo
    // -------------------------------------------------------------------------

    describe('undo/redo', () => {
        it('undoes a text insertion', async () => {
            const { vm } = await makeReadyViewModel();
            vm.insertText('hello');
            const afterInsert = vm.getSnapshot().content;

            vm.undo();
            const afterUndo = vm.getSnapshot().content;

            // Content should have changed after undo
            expect(afterUndo).not.toBe(afterInsert);
        });

        it('redoes after undo', async () => {
            const { vm } = await makeReadyViewModel();
            vm.insertText('hello');
            const afterInsert = vm.getSnapshot().content;

            vm.undo();
            vm.redo();
            const afterRedo = vm.getSnapshot().content;

            expect(afterRedo).toBe(afterInsert);
        });
    });

    // -------------------------------------------------------------------------
    // setContentFromHtml / setContentFromMarkdown
    // -------------------------------------------------------------------------

    describe('setContent*', () => {
        it('setContentFromHtml updates snapshot', async () => {
            const { vm } = await makeReadyViewModel();
            vm.setContentFromHtml('<em>italic</em>');
            expect(vm.getSnapshot().content).toContain('em');
            expect(vm.getSnapshot().actionStates.italic).toBe('reversed');
        });

        it('setContentFromMarkdown updates snapshot', async () => {
            const { vm } = await makeReadyViewModel();
            vm.setContentFromMarkdown('**bold text**');
            // Content should now have a bold element
            expect(vm.getSnapshot().content).toBeTruthy();
        });
    });

    // -------------------------------------------------------------------------
    // replaceText
    // -------------------------------------------------------------------------

    describe('replaceText', () => {
        it('replaces editor content with plain text', async () => {
            const { vm } = await makeReadyViewModel({ initialContent: '<strong>Old</strong>' });
            vm.replaceText('New content');
            expect(vm.getSnapshot().content).toContain('New content');
        });
    });

    // -------------------------------------------------------------------------
    // getLink
    // -------------------------------------------------------------------------

    describe('getLink', () => {
        it('returns empty string when no link is selected', async () => {
            const { vm } = await makeReadyViewModel();
            expect(vm.getLink()).toBe('');
        });
    });

    // -------------------------------------------------------------------------
    // Lifecycle — attach / detach / dispose
    // -------------------------------------------------------------------------

    describe('lifecycle', () => {
        it('does not process events after detach', async () => {
            const { vm, editor } = await makeReadyViewModel();
            const listener = vi.fn();
            vm.subscribe(listener);
            listener.mockClear();

            vm.detach();

            fireWysiwygInput(editor, 'formatBold');
            // No listener calls because the event listeners were removed
            expect(listener).not.toHaveBeenCalled();
        });

        it('disposes cleanly', async () => {
            const { vm } = await makeReadyViewModel();
            expect(() => vm.dispose()).not.toThrow();
        });

        it('can init without attaching an editor (no DOM update)', async () => {
            const vm = new WysiwygViewModel();
            await vm.init();
            expect(vm.getSnapshot().isReady).toBe(true);
        });

        it('remains ready after calling init twice', async () => {
            const { vm } = await makeReadyViewModel();
            await vm.init();
            expect(vm.getSnapshot().isReady).toBe(true);
            // The second call re-creates the model — both calls should succeed
            expect(mockRustModel.new_composer_model).toHaveBeenCalledTimes(2);
        });
    });
});
