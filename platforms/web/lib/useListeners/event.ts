/*
Copyright 2024 New Vector Ltd.
Copyright 2022 The Matrix.org Foundation C.I.C.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

import { MouseEvent as ReactMouseEvent } from 'react';
import {
    ComposerModel,
    MenuStateUpdate,
    SuggestionPattern,
} from '@vector-im/matrix-wysiwyg-wasm';

import { processEvent, processInput } from '../composer';
import {
    getCurrentSelection,
    refreshComposerView,
    replaceEditor,
} from '../dom';
import {
    BlockType,
    FormattingFunctions,
    InputEventProcessor,
    WysiwygInputEvent,
} from '../types';
import { TestUtilities } from '../useTestCases/types';
import { AllActionStates } from '../types';
import { mapToAllActionStates } from './utils';
import { AtRoomSuggestionEvent, LinkEvent, SuggestionEvent } from './types';
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
        const repl = update.text_update().replace_all;
        if (repl) {
            replaceEditor(
                editor,
                repl.replacement_html,
                repl.start_utf16_codeunit,
                repl.end_utf16_codeunit,
            );
            testUtilities.setEditorHtml(repl.replacement_html);
        }
        editor.focus();

        // Only when
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
