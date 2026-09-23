/*
Copyright 2026 Element Creations Ltd.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

import { describe, it, expect, vi, afterEach } from 'vitest';
import * as wasm from '@vector-im/matrix-wysiwyg-wasm';

import {
    WysiwygViewModel,
    type WysiwygViewModelProps,
} from './WysiwygViewModel';
import { type InputEventProcessor, type WysiwygInputEvent } from './types';

const created: WysiwygViewModel[] = [];

function makeViewModel(props: WysiwygViewModelProps = {}): WysiwygViewModel {
    const vm = new WysiwygViewModel(props);
    created.push(vm);
    return vm;
}

async function makeReadyViewModel(
    props: WysiwygViewModelProps = {},
): Promise<WysiwygViewModel> {
    const vm = makeViewModel(props);
    await vm.ready;
    return vm;
}

function inputEvent(inputType: string, data?: string): WysiwygInputEvent {
    return { inputType, data } as WysiwygInputEvent;
}

/** Wait until the view model has swapped in a fresh model (panic recovery). */
async function waitForReinit(
    vm: WysiwygViewModel,
    previous: wasm.ComposerModel | null,
): Promise<void> {
    for (let i = 0; i < 20 && vm.composerModel === previous; i++) {
        await new Promise((resolve) => setTimeout(resolve, 0));
    }
}

describe('WysiwygViewModel', () => {
    afterEach(() => {
        created.splice(0).forEach((vm) => vm.dispose());
        vi.restoreAllMocks();
    });

    describe('initialisation', () => {
        it('is not ready before the model exists', () => {
            const vm = makeViewModel();
            const snapshot = vm.getSnapshot();
            expect(snapshot.isReady).toBe(false);
            expect(snapshot.content).toBeNull();
            expect(snapshot.render).toBeNull();
        });

        it('becomes ready with an empty model', async () => {
            const vm = await makeReadyViewModel();
            const snapshot = vm.getSnapshot();
            expect(snapshot.isReady).toBe(true);
            expect(snapshot.content).toBe('');
            expect(snapshot.messageContent).toBe('');
            expect(snapshot.suggestion).toBeNull();
            expect(snapshot.render).toMatchObject({
                html: '',
                blocks: [],
                start: 0,
                end: 0,
            });
            expect(snapshot.actionStates.bold).toBe('enabled');
            expect(snapshot.actionStates.undo).toBe('disabled');
        });

        it('seeds the model from initialContent', async () => {
            const vm = await makeReadyViewModel({
                initialContent: '<strong>Hello</strong>',
            });
            const snapshot = vm.getSnapshot();
            expect(snapshot.content).toBe('<strong>Hello</strong>');
            expect(snapshot.messageContent).toContain('Hello');
            expect(snapshot.render?.html).toBe('<strong>Hello</strong>');
        });

        it('falls back to an empty model when initialContent cannot be parsed', async () => {
            vi.spyOn(wasm, 'new_composer_model_from_html').mockImplementation(
                () => {
                    throw new Error('parse failure');
                },
            );
            const vm = await makeReadyViewModel({ initialContent: '<<<' });
            expect(vm.getSnapshot().isReady).toBe(true);
            expect(vm.getSnapshot().content).toBe('');
        });

        it('passes emoji shortcodes to the model as custom suggestion patterns', async () => {
            const spy = vi.spyOn(
                wasm.ComposerModel.prototype,
                'set_custom_suggestion_patterns',
            );
            await makeReadyViewModel({
                emojiSuggestions: new Map([[':)', '🙂']]),
            });
            expect(spy).toHaveBeenCalledWith([':)']);
        });

        it('does not publish a model when disposed before the WASM module loads', async () => {
            const vm = makeViewModel();
            vm.dispose();
            await vm.ready;
            expect(vm.getSnapshot().isReady).toBe(false);
            expect(vm.composerModel).toBeNull();
        });
    });

    describe('subscriptions', () => {
        it('notifies subscribers when the snapshot changes', async () => {
            const vm = await makeReadyViewModel();
            const listener = vi.fn();
            vm.subscribe(listener);
            vm.insertText('a');
            expect(listener).toHaveBeenCalledTimes(1);
        });

        it('stops notifying after unsubscribe', async () => {
            const vm = await makeReadyViewModel();
            const listener = vi.fn();
            const unsubscribe = vm.subscribe(listener);
            unsubscribe();
            vm.insertText('a');
            expect(listener).not.toHaveBeenCalled();
        });
    });

    describe('text input', () => {
        it('publishes content, message content and a render instruction', async () => {
            const vm = await makeReadyViewModel();
            vm.insertText('hello');
            const snapshot = vm.getSnapshot();
            expect(snapshot.content).toBe('hello');
            expect(snapshot.messageContent).toBe('hello');
            expect(snapshot.render).toMatchObject({
                html: 'hello',
                start: 5,
                end: 5,
            });
            expect(snapshot.render?.blocks).toHaveLength(1);
        });

        it('publishes a new render object for every model rewrite', async () => {
            const vm = await makeReadyViewModel();
            vm.insertText('a');
            const first = vm.getSnapshot().render;
            vm.insertText('b');
            expect(vm.getSnapshot().render).not.toBe(first);
        });

        it('accepts native input events', async () => {
            const vm = await makeReadyViewModel();
            vm.handleInput(inputEvent('insertText', 'typed'));
            expect(vm.getSnapshot().content).toBe('typed');
        });

        it('clears the composer', async () => {
            const vm = await makeReadyViewModel();
            vm.insertText('hello');
            vm.clear();
            expect(vm.getSnapshot().content).toBe('');
            expect(vm.getSnapshot().messageContent).toBe('');
        });

        it('ignores input before the model is ready', () => {
            const vm = makeViewModel();
            vm.insertText('too early');
            expect(vm.getSnapshot().content).toBeNull();
        });
    });

    describe('formatting', () => {
        it('toggles the bold action state and formats typed text', async () => {
            const vm = await makeReadyViewModel();
            vm.bold();
            expect(vm.getSnapshot().actionStates.bold).toBe('reversed');
            vm.insertText('bold');
            expect(vm.getSnapshot().content).toBe('<strong>bold</strong>');
        });

        it('reports list state for indent controls', async () => {
            const vm = await makeReadyViewModel();
            vm.orderedList();
            expect(vm.getSnapshot().actionStates.orderedList).toBe('reversed');
        });

        it('undoes and redoes', async () => {
            const vm = await makeReadyViewModel();
            vm.insertText('hello');
            vm.undo();
            expect(vm.getSnapshot().content).toBe('');
            vm.redo();
            expect(vm.getSnapshot().content).toBe('hello');
        });

        it('inserts and removes links', async () => {
            const vm = await makeReadyViewModel();
            vm.link('https://example.org', 'example');
            expect(vm.getSnapshot().content).toContain(
                'href="https://example.org"',
            );
        });

        it('returns an empty link when nothing is selected', async () => {
            const vm = await makeReadyViewModel();
            expect(vm.getLink()).toBe('');
        });
    });

    describe('keyboard shortcuts', () => {
        it('resolves a shortcut to its formatting action and consumes the event', async () => {
            const vm = await makeReadyViewModel();
            const event = new KeyboardEvent('keydown', {
                ctrlKey: true,
                key: 'b',
                cancelable: true,
            });
            vm.handleKeyDown(event);
            expect(event.defaultPrevented).toBe(true);
            expect(vm.getSnapshot().actionStates.bold).toBe('reversed');
        });

        it('leaves other key presses to the browser', async () => {
            const vm = await makeReadyViewModel();
            const event = new KeyboardEvent('keydown', {
                key: 'a',
                cancelable: true,
            });
            vm.handleKeyDown(event);
            expect(event.defaultPrevented).toBe(false);
        });
    });

    describe('selection', () => {
        it('applies formatting to the selected range', async () => {
            const vm = await makeReadyViewModel();
            vm.insertText('hello world');
            vm.select(0, 5);
            vm.bold();
            expect(vm.getSnapshot().content).toBe(
                '<strong>hello</strong> world',
            );
        });

        it('updates action states from the selection', async () => {
            const vm = await makeReadyViewModel();
            vm.bold();
            vm.insertText('bold');
            vm.insertText(' ');
            vm.bold();
            vm.insertText('plain');
            vm.select(1, 1);
            expect(vm.getSnapshot().actionStates.bold).toBe('reversed');
            vm.select(8, 8);
            expect(vm.getSnapshot().actionStates.bold).toBe('enabled');
        });

        it('ignores a selection the model already has', async () => {
            const vm = await makeReadyViewModel();
            vm.insertText('hello');
            const spy = vi.spyOn(vm.composerModel!, 'select');
            vm.select(5, 5);
            expect(spy).not.toHaveBeenCalled();
        });

        it('ignores the reversed form of the current selection', async () => {
            const vm = await makeReadyViewModel();
            vm.insertText('hello');
            vm.select(1, 4);
            const spy = vi.spyOn(vm.composerModel!, 'select');
            vm.select(4, 1);
            expect(spy).not.toHaveBeenCalled();
        });

        it('defers to the test-case recorder about whether a selection is new', async () => {
            const vm = await makeReadyViewModel();
            vm.insertText('hello');
            const traceAction = vi.fn((update) => update);
            vm.setTestHooks({
                traceAction,
                getSelectionAccordingToActions: () => [0, 0],
            });
            vm.select(5, 5);
            expect(traceAction).toHaveBeenCalledWith(null, 'select', 5, 5);
        });
    });

    describe('suggestions', () => {
        it('publishes a mention suggestion while typing one', async () => {
            const vm = await makeReadyViewModel();
            vm.insertText('@al');
            const suggestion = vm.getSnapshot().suggestion;
            expect(suggestion?.type).toBe('mention');
            expect(suggestion?.keyChar).toBe('@');
        });

        it('keeps the same suggestion object while the pattern is unchanged', async () => {
            const vm = await makeReadyViewModel();
            vm.insertText('@al');
            const first = vm.getSnapshot().suggestion;
            vm.select(2, 2);
            expect(vm.getSnapshot().suggestion).toBe(first);
        });

        it('replaces the suggestion with a mention', async () => {
            const vm = await makeReadyViewModel();
            vm.insertText('@al');
            vm.mention(
                'https://matrix.to/#/@alice:example.org',
                'Alice',
                new Map(),
            );
            const snapshot = vm.getSnapshot();
            expect(snapshot.content).toContain(
                'href="https://matrix.to/#/@alice:example.org"',
            );
            expect(snapshot.suggestion).toBeNull();
        });

        it('replaces a command suggestion with the command', async () => {
            const vm = await makeReadyViewModel();
            vm.insertText('/spo');
            expect(vm.getSnapshot().suggestion?.type).toBe('command');
            vm.command('/spoiler');
            // The model appends a (non-breaking) space after a command.
            expect(vm.getSnapshot().content).toMatch(/^\/spoiler\s$/);
        });
    });

    describe('inputEventProcessor', () => {
        it('receives every input together with the editor element', async () => {
            const editor = document.createElement('div');
            const processor = vi.fn<InputEventProcessor>((e) => e);
            const vm = await makeReadyViewModel({
                inputEventProcessor: processor,
                editorRef: { current: editor },
            });
            vm.insertText('a');
            expect(processor).toHaveBeenCalledWith(
                expect.objectContaining({ inputType: 'insertText' }),
                expect.objectContaining({ actions: vm }),
                editor,
            );
        });

        it('suppresses the input when the processor returns null', async () => {
            const vm = await makeReadyViewModel({
                inputEventProcessor: () => null,
                editorRef: { current: document.createElement('div') },
            });
            vm.insertText('a');
            expect(vm.getSnapshot().content).toBe('');
        });

        it('is not consulted while no editor element is bound', async () => {
            const processor = vi.fn<InputEventProcessor>(() => null);
            const vm = await makeReadyViewModel({
                inputEventProcessor: processor,
            });
            vm.insertText('a');
            expect(processor).not.toHaveBeenCalled();
            expect(vm.getSnapshot().content).toBe('a');
        });

        it('can be replaced after construction', async () => {
            const vm = await makeReadyViewModel({
                editorRef: { current: document.createElement('div') },
            });
            vm.setInputEventProcessor(() => null);
            vm.insertText('a');
            expect(vm.getSnapshot().content).toBe('');
            vm.setInputEventProcessor(undefined);
            vm.insertText('b');
            expect(vm.getSnapshot().content).toBe('b');
        });
    });

    describe('emoji suggestions', () => {
        it('can be updated after construction', async () => {
            const vm = await makeReadyViewModel();
            const spy = vi.spyOn(
                vm.composerModel!,
                'set_custom_suggestion_patterns',
            );
            vm.setEmojiSuggestions(new Map([[':D', '😃']]));
            expect(spy).toHaveBeenCalledWith([':D']);
        });
    });

    describe('content replacement', () => {
        it('replaces content from HTML', async () => {
            const vm = await makeReadyViewModel();
            vm.insertText('old');
            vm.setContentFromHtml('<em>new</em>');
            const snapshot = vm.getSnapshot();
            expect(snapshot.content).toBe('<em>new</em>');
            expect(snapshot.render?.html).toBe('<em>new</em>');
        });

        it('replaces content from Markdown', async () => {
            const vm = await makeReadyViewModel();
            vm.setContentFromMarkdown('**bold**');
            expect(vm.getSnapshot().content).toBe('<strong>bold</strong>');
        });
    });

    describe('native edit reconciliation', () => {
        it('reports whether the model handled an input', async () => {
            const vm = await makeReadyViewModel();
            expect(vm.handleInput(inputEvent('insertText', 'a'))).toBe(true);
            vi.spyOn(console, 'error').mockImplementation(() => undefined);
            expect(vm.handleInput(inputEvent('insertFromDrop'))).toBe(false);
        });

        it('reports an input the processor suppressed as unhandled', async () => {
            const vm = await makeReadyViewModel({
                inputEventProcessor: () => null,
                editorRef: { current: document.createElement('div') },
            });
            expect(vm.handleInput(inputEvent('insertText', 'a'))).toBe(false);
        });

        it('replaces a text range and re-publishes the whole content', async () => {
            const vm = await makeReadyViewModel();
            vm.insertText('helo');
            const before = vm.getSnapshot().render;
            vm.replaceTextIn('l', 2, 2);
            const snapshot = vm.getSnapshot();
            expect(snapshot.content).toBe('hello');
            expect(snapshot.render).not.toBe(before);
            expect(snapshot.render?.html).toBe('hello');
        });
    });

    describe('panic recovery', () => {
        it('rebuilds the model with the last plain text when an action throws', async () => {
            const vm = await makeReadyViewModel();
            vm.insertText('keep me');
            const broken = vm.composerModel!;
            vi.spyOn(broken, 'bold').mockImplementation(() => {
                throw new Error('wasm panic');
            });

            vm.bold();
            await waitForReinit(vm, broken);

            expect(vm.composerModel).not.toBe(broken);
            expect(vm.getSnapshot().isReady).toBe(true);
            expect(vm.getSnapshot().content).toBe('keep me');
        });
    });

    describe('dispose', () => {
        it('frees the model and stops accepting input', async () => {
            const vm = await makeReadyViewModel();
            const free = vi.spyOn(vm.composerModel!, 'free');
            vm.dispose();
            expect(free).toHaveBeenCalled();
            expect(vm.composerModel).toBeNull();
            vm.insertText('ignored');
            expect(vm.getSnapshot().content).toBe('');
        });
    });
});
