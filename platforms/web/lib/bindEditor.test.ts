/*
Copyright 2026 Element Creations Ltd.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

import { describe, it, expect, vi, afterEach } from 'vitest';

import { bindEditor } from './bindEditor';
import { WysiwygViewModel } from './WysiwygViewModel';
import { selectContent } from './dom';

const cleanups: Array<() => void> = [];

function makeEditor(): HTMLDivElement {
    const editor = document.createElement('div');
    editor.setAttribute('contenteditable', 'true');
    document.body.appendChild(editor);
    return editor;
}

async function makeBoundEditor(
    initialContent?: string,
): Promise<{ vm: WysiwygViewModel; editor: HTMLDivElement }> {
    const vm = new WysiwygViewModel({ initialContent });
    const editor = makeEditor();
    cleanups.push(bindEditor(vm, editor), () => vm.dispose());
    await vm.ready;
    return { vm, editor };
}

function typeText(editor: HTMLElement, data: string): void {
    editor.dispatchEvent(
        new InputEvent('input', { data, inputType: 'insertText' }),
    );
}

function changeSelection(
    editor: HTMLElement,
    start: number,
    end: number,
): void {
    selectContent(editor, start, end);
    document.dispatchEvent(new CustomEvent('selectionchange'));
}

describe('bindEditor', () => {
    afterEach(() => {
        cleanups.splice(0).forEach((cleanup) => cleanup());
        document.body.innerHTML = '';
        vi.restoreAllMocks();
    });

    it('gives an empty editor a <br> so the caret can be placed', () => {
        const vm = new WysiwygViewModel();
        const editor = makeEditor();
        cleanups.push(bindEditor(vm, editor), () => vm.dispose());
        expect(editor.innerHTML).toBe('<br>');
    });

    it('renders initial content once the model is ready, without taking focus', async () => {
        const { editor } = await makeBoundEditor('<em>hi</em>');
        expect(editor.innerHTML).toBe('<div><em>hi</em></div>');
        expect(document.activeElement).not.toBe(editor);
    });

    it('renders content the model already holds when bound late', async () => {
        const vm = new WysiwygViewModel({ initialContent: 'late' });
        cleanups.push(() => vm.dispose());
        await vm.ready;
        const editor = makeEditor();
        cleanups.push(bindEditor(vm, editor));
        expect(editor.innerHTML).toBe('<div>late</div>');
    });

    it('forwards input events to the model and renders the result', async () => {
        const { vm, editor } = await makeBoundEditor();
        typeText(editor, 'abc');
        expect(vm.getSnapshot().content).toBe('abc');
        expect(editor.innerHTML).toBe('<div>abc</div>');
    });

    it('ignores input events that are part of an IME composition', async () => {
        const { vm, editor } = await makeBoundEditor();
        editor.dispatchEvent(
            new InputEvent('input', {
                data: 'か',
                inputType: 'insertCompositionText',
                isComposing: true,
            }),
        );
        expect(vm.getSnapshot().content).toBe('');
    });

    it('commits composed text when the composition ends', async () => {
        const { vm, editor } = await makeBoundEditor();
        editor.dispatchEvent(
            new CompositionEvent('compositionend', { data: 'か' }),
        );
        expect(vm.getSnapshot().content).toBe('か');
    });

    it('renders actions invoked on the view model', async () => {
        const { vm, editor } = await makeBoundEditor();
        vm.insertText('hello');
        expect(editor.innerHTML).toBe('<div>hello</div>');
        vm.setContentFromHtml('<strong>replaced</strong>');
        expect(editor.innerHTML).toBe('<div><strong>replaced</strong></div>');
    });

    it('forwards user selection changes to the model', async () => {
        const { vm, editor } = await makeBoundEditor();
        typeText(editor, 'hello');
        changeSelection(editor, 0, 5);
        vm.bold();
        expect(editor.innerHTML).toBe('<div><strong>hello</strong></div>');
    });

    it('ignores the selectionchange echo of its own render', async () => {
        const { vm, editor } = await makeBoundEditor();
        typeText(editor, 'hello');
        const select = vi.spyOn(vm, 'select');
        // The browser reports the selection we just set: not a user action.
        document.dispatchEvent(new CustomEvent('selectionchange'));
        expect(select).not.toHaveBeenCalled();
        // A genuinely different selection goes through.
        changeSelection(editor, 1, 1);
        expect(select).toHaveBeenCalledWith(1, 1);
    });

    it('resolves keyboard shortcuts', async () => {
        const { vm, editor } = await makeBoundEditor();
        editor.dispatchEvent(
            new KeyboardEvent('keydown', { ctrlKey: true, key: 'b' }),
        );
        expect(vm.getSnapshot().actionStates.bold).toBe('reversed');
    });

    it('reconciles an edit the model did not intercept', async () => {
        const { vm, editor } = await makeBoundEditor();
        typeText(editor, 'helo');
        // The browser applies an edit of a kind the model has no handler for
        // (e.g. a drop) and then fires `input`.
        const textNode = editor.querySelector('div')!.firstChild!;
        textNode.textContent = 'hello';
        vi.spyOn(console, 'error').mockImplementation(() => undefined);
        editor.dispatchEvent(
            new InputEvent('input', { inputType: 'insertFromDrop' }),
        );
        expect(vm.getSnapshot().content).toBe('hello');
        expect(editor.innerHTML).toBe('<div>hello</div>');
    });

    it('does nothing when an unhandled input left the text unchanged', async () => {
        const { vm, editor } = await makeBoundEditor();
        typeText(editor, 'hello');
        const replaceTextIn = vi.spyOn(vm, 'replaceTextIn');
        vi.spyOn(console, 'error').mockImplementation(() => undefined);
        editor.dispatchEvent(
            new InputEvent('input', { inputType: 'insertFromDrop' }),
        );
        expect(replaceTextIn).not.toHaveBeenCalled();
    });

    it('stops forwarding events after unbind', async () => {
        const vm = new WysiwygViewModel();
        cleanups.push(() => vm.dispose());
        const editor = makeEditor();
        const unbind = bindEditor(vm, editor);
        await vm.ready;
        unbind();
        typeText(editor, 'abc');
        expect(vm.getSnapshot().content).toBe('');
        vm.insertText('x');
        expect(editor.innerHTML).toBe('<br>');
    });
});
