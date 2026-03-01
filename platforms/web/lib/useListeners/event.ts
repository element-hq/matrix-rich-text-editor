/*
Copyright 2024 New Vector Ltd.
Copyright 2022 The Matrix.org Foundation C.I.C.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

import { type MouseEvent as ReactMouseEvent } from 'react';
import {
    type ComposerModel,
    type MenuStateUpdate,
    type SuggestionPattern,
} from '@vector-im/matrix-wysiwyg-wasm';

import { processEvent, processInput } from '../composer';
import {
    getCurrentSelection,
    refreshComposerView,
    replaceEditor,
    selectContent,
    textNodeNeedsExtraOffset,
} from '../dom';
import { renderProjections, BlockProjection } from '../blockProjection';
import { computePrefixSuffixDiff } from '../inlineReconciliation';
import {
    type BlockType,
    type FormattingFunctions,
    type InputEventProcessor,
    type WysiwygInputEvent,
} from '../types';
import { type TestUtilities } from '../useTestCases/types';
import { type AllActionStates } from '../types';
import { mapToAllActionStates } from './utils';
import {
    type AtRoomSuggestionEvent,
    type LinkEvent,
    type SuggestionEvent,
} from './types';
import { getUserOperatingSystem } from '../utils';

/**
 * Send a custom event named wysiwygInput
 * See {FormatBlockEvent} for the event structure
 * @param {HTMLElement} editor
 * @param {BlockType} blockType
 * @param {ReactMouseEvent<HTMLElement, MouseEvent> | KeyboardEvent} e
 * @param {String} data
 */
export function sendWysiwygInputEvent(
    editor: HTMLElement,
    blockType: BlockType,
    e?: ReactMouseEvent<HTMLElement, MouseEvent> | KeyboardEvent,
    data?:
        | string
        | LinkEvent['data']
        | SuggestionEvent['data']
        | AtRoomSuggestionEvent['data'],
): void {
    e?.preventDefault();
    e?.stopPropagation();
    editor.dispatchEvent(
        new CustomEvent('wysiwygInput', { detail: { blockType, data } }),
    );
}

/**
 * Return the blockType associated to a shortcut
 * @param {KeyboardEvent} e
 * @returns {BlockType | null}
 */
function getInputFromKeyDown(
    e: KeyboardEvent,
    composerModel: ComposerModel,
    formattingFunctions: FormattingFunctions,
    editor: HTMLElement,
    inputEventProcessor?: InputEventProcessor,
): BlockType | null {
    if (e.shiftKey && e.altKey) {
        switch (e.key) {
            case '5':
                return 'formatStrikeThrough';
        }
    }

    const operatingSystem = getUserOperatingSystem();
    if (operatingSystem === 'Windows' || operatingSystem === 'Linux') {
        if (e.ctrlKey) {
            switch (e.key) {
                case 'Backspace':
                    return 'deleteWordBackward';
            }
        }
    }

    if (e.ctrlKey || e.metaKey) {
        switch (e.key) {
            case 'b':
                return 'formatBold';
            case 'i':
                return 'formatItalic';
            case 'u':
                return 'formatUnderline';
            case 'e':
                return 'formatInlineCode';
            case 'y':
                return 'historyRedo';
            case 'z':
                return 'historyUndo';
            case 'Z':
                return 'historyRedo';
            case 'Enter':
                return 'sendMessage';
            case 'Backspace':
                return 'deleteSoftLineBackward';
        }
    }

    processEvent(
        e,
        {
            actions: formattingFunctions,
            content: () => composerModel.get_content_as_html(),
            messageContent: () => composerModel.get_content_as_message_html(),
        },
        editor,
        inputEventProcessor,
    );
    return null;
}

/**
 * Event listener for keydown event
 * @param {KeyboardEvent} e
 * @param {HTMLElement} editor
 */
export function handleKeyDown(
    e: KeyboardEvent,
    editor: HTMLElement,
    composerModel: ComposerModel,
    formattingFunctions: FormattingFunctions,
    inputEventProcessor?: InputEventProcessor,
): void {
    const inputType = getInputFromKeyDown(
        e,
        composerModel,
        formattingFunctions,
        editor,
        inputEventProcessor,
    );
    if (inputType) {
        sendWysiwygInputEvent(editor, inputType, e);
    }
}

/**
 * Extract the action states from the menu state of the composer
 * @param {MenuStateUpdate} menuStateUpdate menu state update from the composer
 * @returns {AllActionStates}
 */
export function extractActionStates(
    menuStateUpdate: MenuStateUpdate,
): AllActionStates {
    return mapToAllActionStates(menuStateUpdate.action_states);
}

/**
 * Event listener for WysiwygInputEvent
 * @param {WysiwygInputEvent} e
 * @param {HTMLElement} editor
 * @param {ComposerModel} composerModel
 * @param {HTMLElement | null} modelNode
 * @param {TestUtilities} testUtilities
 * @param {FormattingFunctions} formattingFunctions
 * @param {string} committedText - last plain text committed to the editor via renderProjections
 * @param {InputEventProcessor} inputEventProcessor
 * @returns
 */
export function handleInput(
    e: WysiwygInputEvent,
    editor: HTMLElement,
    composerModel: ComposerModel,
    modelNode: HTMLElement | null,
    testUtilities: TestUtilities,
    formattingFunctions: FormattingFunctions,
    suggestion: SuggestionPattern | null,
    committedTextRef: { current: string },
    inputEventProcessor?: InputEventProcessor,
    emojiSuggestions?: Map<string, string>,
):
    | {
          content?: string;
          actionStates: AllActionStates | null;
          suggestion: SuggestionPattern | null;
      }
    | undefined {
    const update = processInput(
        e,
        composerModel,
        testUtilities.traceAction,
        formattingFunctions,
        editor,
        suggestion,
        inputEventProcessor,
        emojiSuggestions,
    );
    if (update) {
        const textUpdate = update.text_update();
        const repl = textUpdate.replace_all;
        const sel = textUpdate.select;

        if (repl) {
            // Use projection-based rendering instead of innerHTML assignment.
            const projections = (composerModel as any).get_block_projections?.() as BlockProjection[] | undefined;
            if (projections) {
                committedTextRef.current = renderProjections(projections, editor);
                selectContent(editor, repl.start_utf16_codeunit, repl.end_utf16_codeunit);
            } else {
                // Fallback to legacy HTML path if projection API unavailable.
                replaceEditor(
                    editor,
                    repl.replacement_html,
                    repl.start_utf16_codeunit,
                    repl.end_utf16_codeunit,
                );
            }
            testUtilities.setEditorHtml(repl.replacement_html);
        } else if (sel) {
            // Selection-only update: just move the cursor.
            selectContent(editor, sel.start_utf16_codeunit, sel.end_utf16_codeunit);
        }
        editor.focus();

        if (modelNode) {
            refreshComposerView(modelNode, composerModel);
        }

        const menuStateUpdate = update.menu_state().update();
        const menuActionUpdate = update
            .menu_action()
            .suggestion()?.suggestion_pattern;

        const actionStates = menuStateUpdate
            ? extractActionStates(menuStateUpdate)
            : null;

        const suggestion = menuActionUpdate || null;

        const res = {
            content: repl?.replacement_html,
            actionStates,
            suggestion,
        };

        return res;
    }
}

/**
 * Reconcile the browser's current `editor.textContent` with the last
 * committed text, then feed the minimal diff into `composerModel.replace_text_in()`.
 *
 * This is the equivalent of `WysiwygComposerViewModel.reconcileNative()` on iOS.
 * It is called after the browser has applied a plain-text edit that wasn't
 * intercepted by a structural handler (enter, backspace, formatting).
 *
 * @returns updated HTML content string if the model was changed, or undefined
 */
export function reconcileNative(
    editor: HTMLElement,
    composerModel: ComposerModel,
    committedTextRef: { current: string },
): { content?: string } | undefined {
    const newText = editor.textContent ?? '';
    const oldText = committedTextRef.current;

    if (oldText === newText) return undefined;

    const diff = computePrefixSuffixDiff(oldText, newText);

    // Translate DOM text offsets to Rust model offsets (block separators are
    // implicit in editor.textContent but explicit in Rust's UTF-16 model).
    const modelStart = domTextOffsetToModelOffset(editor, diff.replaceStart);
    const modelEnd = domTextOffsetToModelOffset(editor, diff.replaceEnd);

    // Push the diff into Rust.
    const rustUpdate = (composerModel as any).replace_text_in?.(
        diff.replacement,
        modelStart,
        modelEnd,
    );

    // Re-render from the updated model and sync cursor from Rust's selection.
    const projections = (composerModel as any).get_block_projections?.() as BlockProjection[] | undefined;
    let content: string | undefined;
    if (projections) {
        committedTextRef.current = renderProjections(projections, editor);
    }
    if (rustUpdate) {
        const textUpdate = rustUpdate.text_update();
        const repl = textUpdate.replace_all;
        const sel = textUpdate.select;
        const cursorStart = repl?.start_utf16_codeunit ?? sel?.start_utf16_codeunit;
        const cursorEnd = repl?.end_utf16_codeunit ?? sel?.end_utf16_codeunit;
        if (cursorStart !== undefined && cursorEnd !== undefined) {
            selectContent(editor, cursorStart, cursorEnd);
        }
        content = repl?.replacement_html;
    }

    return { content };
}

/**
 * Convert an offset within `editor.textContent` (which omits block-boundary
 * separators) to the equivalent Rust UTF-16 model offset (which includes them).
 *
 * For each block boundary we cross, the model offset is +1 larger than the
 * DOM text offset.  A boundary is crossed when we move past a text node that
 * has a block-level ancestor (`<p>`, `<li>`, `<pre>`, `<blockquote>`).
 */
function domTextOffsetToModelOffset(editor: HTMLElement, textOffset: number): number {
    // Collect text nodes in document order.
    const textNodes: Node[] = [];
    (function collect(n: Node): void {
        if (n.nodeType === Node.TEXT_NODE) textNodes.push(n);
        else for (const ch of n.childNodes) collect(ch);
    })(editor);

    let pos = 0; // DOM text position cursor
    let extra = 0; // accumulated block-separator adjustments

    for (let i = 0; i < textNodes.length; i++) {
        const node = textNodes[i];
        const len = node.textContent?.length ?? 0;
        const nodeEnd = pos + len;

        if (textOffset < nodeEnd) {
            // Target is strictly inside this text node — no extra separators
            break;
        }

        if (textOffset === nodeEnd) {
            // Exactly at the end of this node.  If the *next* text node is in a
            // different block (needs an extra offset) and this is not the last
            // node, account for the implicit separator.
            if (
                i < textNodes.length - 1 &&
                textNodeNeedsExtraOffset(textNodes[i + 1])
            ) {
                extra += 1;
            }
            break;
        }

        // Target is past this node — consume it and account for the boundary.
        pos = nodeEnd;
        if (
            i < textNodes.length - 1 &&
            textNodeNeedsExtraOffset(textNodes[i + 1])
        ) {
            extra += 1;
        }
    }

    return textOffset + extra;
}

/**
 * Event listener for selectionChange event
 * @param {Editor} editor
 * @param {ComposerModel} composeModel
 * @param {TestUtilities}
 * @returns
 */
export function handleSelectionChange(
    editor: HTMLElement,
    composeModel: ComposerModel,
    { traceAction, getSelectionAccordingToActions }: TestUtilities,
): AllActionStates | undefined {
    const [start, end] = getCurrentSelection(editor, document.getSelection());

    const prevStart = composeModel.selection_start();
    const prevEnd = composeModel.selection_end();

    const [actStart, actEnd] = getSelectionAccordingToActions();

    // Ignore selection changes that do nothing
    if (
        start === prevStart &&
        start === actStart &&
        end === prevEnd &&
        end === actEnd
    ) {
        return;
    }

    // Ignore selection changes that just reverse the selection - all
    // backwards selections actually do this, because the browser can't
    // support backwards selections.
    if (
        start === prevEnd &&
        start === actEnd &&
        end === prevStart &&
        end === actStart
    ) {
        return;
    }
    const update = composeModel.select(start, end);
    traceAction(null, 'select', start, end);

    const menuStateUpdate = update.menu_state().update();

    if (menuStateUpdate) {
        return extractActionStates(menuStateUpdate);
    }
}
