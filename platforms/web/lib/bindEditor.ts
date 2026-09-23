/*
Copyright 2026 Element Creations Ltd.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

import {
    type EditorRender,
    type EditorSelection,
    type WysiwygViewModel,
} from './WysiwygViewModel.js';
import { getCurrentSelection, selectContent } from './dom.js';
import { renderProjections } from './blockProjection.js';
import {
    computePrefixSuffixDiff,
    domTextOffsetToModelOffset,
} from './inlineReconciliation.js';
import { isClipboardEvent, isInputEvent } from './useListeners/assert.js';
import { type WysiwygInputEvent } from './types.js';

/**
 * Connect a contenteditable element to a {@link WysiwygViewModel}.
 *
 * This is the only place that touches the editor's DOM: it forwards browser
 * events to the view model, renders `snapshot.render` into the element from
 * the model's block projection, and pushes edits the model did not intercept
 * back into it with a prefix/suffix diff of the editor's text.
 *
 * @returns a function that removes the listeners and subscription.
 */
export function bindEditor(
    vm: WysiwygViewModel,
    editor: HTMLElement,
): () => void {
    let lastRender: EditorRender | null = null;
    let lastSelection: EditorSelection | null = null;
    // Plain text of the editor as last rendered from the model. Comparing
    // the live text against it yields the edit to reconcile.
    let committedText = '';
    // True while we are rewriting the editor, so that a selectionchange
    // dispatched synchronously mid-rewrite cannot feed half-written DOM
    // state back into the model.
    let isRendering = false;
    // Selection we just applied. Browsers deliver the selectionchange caused
    // by a rewrite after the current task; the first one matching this is
    // that echo, not a user action.
    let expectedSelection: [number, number] | null = null;

    const applySelection = (
        selection: EditorSelection,
        focus: boolean,
    ): void => {
        isRendering = true;
        expectedSelection = [selection.start, selection.end];
        try {
            selectContent(editor, selection.start, selection.end);
            if (focus && document.activeElement !== editor) {
                editor.focus();
            }
        } finally {
            isRendering = false;
        }
    };

    const applyRender = (render: EditorRender, focus: boolean): void => {
        lastRender = render;
        isRendering = true;
        try {
            committedText = renderProjections(render.blocks, editor);
        } finally {
            isRendering = false;
        }
        applySelection(render, focus);
    };

    /**
     * The browser changed the editor's text without the model being
     * involved. Work out the minimal edit and push it into the model, which
     * re-renders the editor from its own state.
     */
    const reconcile = (): void => {
        const liveText = editor.textContent ?? '';
        if (liveText === committedText) return;
        const diff = computePrefixSuffixDiff(committedText, liveText);
        vm.replaceTextIn(
            diff.replacement,
            domTextOffsetToModelOffset(editor, diff.replaceStart),
            domTextOffsetToModelOffset(editor, diff.replaceEnd),
        );
    };

    const onInput = (e: Event): void => {
        if (isInputEvent(e) && !e.isComposing) {
            if (!vm.handleInput(e as WysiwygInputEvent)) reconcile();
        }
    };

    // Called by onPaste and onBeforeInput.
    const onPaste = (e: ClipboardEvent | InputEvent): void => {
        // Safari reports image pastes as an InputEvent carrying a
        // dataTransfer, see https://github.com/vector-im/element-web/issues/25327
        const isSpecialCaseInputEvent =
            isInputEvent(e) &&
            e.inputType === 'insertFromPaste' &&
            e.dataTransfer !== null;

        if (isClipboardEvent(e) || isSpecialCaseInputEvent) {
            e.preventDefault();
            e.stopPropagation();
            vm.handleInput(e as WysiwygInputEvent);
        }
    };

    const onBeforeInput = (e: ClipboardEvent | InputEvent): void => {
        if (isInputEvent(e) && e.isComposing) return;
        onPaste(e);
    };

    const onKeyDown = (e: KeyboardEvent): void => vm.handleKeyDown(e);

    const onCompositionEnd = (e: CompositionEvent): void => {
        // Re-dispatch the composed text as a regular input event.
        onInput(
            new InputEvent('input', {
                data: e.data,
                inputType: 'insertCompositionText',
            }),
        );
    };

    const onSelectionChange = (): void => {
        if (isRendering) return;
        let start: number;
        let end: number;
        try {
            [start, end] = getCurrentSelection(editor, document.getSelection());
        } catch {
            // The selection cannot be mapped onto the editor; ignore it.
            return;
        }
        if (expectedSelection) {
            const [expectedStart, expectedEnd] = expectedSelection;
            expectedSelection = null;
            if (start === expectedStart && end === expectedEnd) return;
        }
        vm.select(start, end);
    };

    editor.addEventListener('input', onInput);
    editor.addEventListener('paste', onPaste);
    editor.addEventListener('beforeinput', onBeforeInput);
    editor.addEventListener('keydown', onKeyDown);
    editor.addEventListener('compositionend', onCompositionEnd);
    document.addEventListener('selectionchange', onSelectionChange);

    // The editor needs at least one child for the caret to be placeable.
    if (!editor.childElementCount) {
        editor.appendChild(document.createElement('br'));
    }

    const onRenderChanged = (render: EditorRender): void => {
        if (lastRender === null) {
            // First sight of the model's content, whether it was ready before
            // we bound or has just become ready. Show it without stealing
            // focus; an empty model has nothing to show, so leave the
            // element alone entirely.
            lastRender = render;
            if (render.html !== '') applyRender(render, false);
        } else {
            // The model rewrote the editor in response to an action: make
            // sure the caret ends up in it, as the user would expect.
            applyRender(render, true);
        }
    };

    const initial = vm.getSnapshot().render;
    if (initial) onRenderChanged(initial);
    lastSelection = vm.getSnapshot().selection;

    const unsubscribe = vm.subscribe(() => {
        const { render, selection } = vm.getSnapshot();
        if (render && render !== lastRender) onRenderChanged(render);
        if (selection && selection !== lastSelection) {
            lastSelection = selection;
            applySelection(selection, true);
        }
    });

    return (): void => {
        unsubscribe();
        editor.removeEventListener('input', onInput);
        editor.removeEventListener('paste', onPaste);
        editor.removeEventListener('beforeinput', onBeforeInput);
        editor.removeEventListener('keydown', onKeyDown);
        editor.removeEventListener('compositionend', onCompositionEnd);
        document.removeEventListener('selectionchange', onSelectionChange);
    };
}
