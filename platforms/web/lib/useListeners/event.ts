/*
Copyright 2024 New Vector Ltd.
Copyright 2022 The Matrix.org Foundation C.I.C.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

import {
    type ComposerModel,
    type MenuStateUpdate,
} from '@vector-im/matrix-wysiwyg-wasm';

import { processEvent } from '../composer';
import {
    type BlockType,
    type FormattingFunctions,
    type InputEventProcessor,
} from '../types';
import { type AllActionStates } from '../types';
import { mapToAllActionStates } from './utils';
import { getUserOperatingSystem } from '../utils';

/**
 * Return the blockType associated to a keyboard shortcut, or null when the
 * key press is not a shortcut. Non-shortcut key presses are offered to the
 * `inputEventProcessor` on the way through.
 * @param {KeyboardEvent} e
 * @returns {BlockType | null}
 */
export function getInputFromKeyDown(
    e: KeyboardEvent,
    composerModel: ComposerModel,
    formattingFunctions: FormattingFunctions,
    editor: HTMLElement | null,
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
 * Extract the action states from the menu state of the composer
 * @param {MenuStateUpdate} menuStateUpdate menu state update from the composer
 * @returns {AllActionStates}
 */
export function extractActionStates(
    menuStateUpdate: MenuStateUpdate,
): AllActionStates {
    return mapToAllActionStates(menuStateUpdate.action_states);
}
