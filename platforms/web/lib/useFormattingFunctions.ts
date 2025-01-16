/*
Copyright 2024 New Vector Ltd.
Copyright 2022 The Matrix.org Foundation C.I.C.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

import { RefObject, useMemo } from 'react';
import { ComposerModel } from '@vector-im/matrix-wysiwyg-wasm';

import { BlockType, FormattingFunctions } from './types';
import { sendWysiwygInputEvent } from './useListeners';
import {
    AllowedMentionAttributes,
    AtRoomSuggestionEvent,
    LinkEvent,
    SuggestionEvent,
} from './useListeners/types';

export function useFormattingFunctions(
    editorRef: RefObject<HTMLElement | null>,
    composerModel: ComposerModel | null,
): FormattingFunctions {
    const formattingFunctions = useMemo<FormattingFunctions>(() => {
        // The formatting action like inline code doesn't have an input type
        // Safari does not keep the inputType in an input event
        // when the input event is fired manually, so we send a custom event
        // and we do not use the browser input event handling
        const sendEvent = (
            blockType: BlockType,
            data?:
                | string
                | LinkEvent['data']
                | SuggestionEvent['data']
                | AtRoomSuggestionEvent['data'],
        ): void => {
            if (editorRef.current) {
                sendWysiwygInputEvent(
                    editorRef.current,
                    blockType,
                    undefined,
                    data,
                );
            }
        };

        return {
            bold: (): void => sendEvent('formatBold'),
            italic: (): void => sendEvent('formatItalic'),
            strikeThrough: (): void => sendEvent('formatStrikeThrough'),
            underline: (): void => sendEvent('formatUnderline'),
            undo: (): void => sendEvent('historyUndo'),
            redo: (): void => sendEvent('historyRedo'),
            orderedList: (): void => sendEvent('insertOrderedList'),
            unorderedList: (): void => sendEvent('insertUnorderedList'),
            inlineCode: (): void => sendEvent('formatInlineCode'),
            clear: (): void => sendEvent('clear'),
            insertText: (text: string): void => sendEvent('insertText', text),
            link: (url: string, text?: string): void =>
                sendEvent('insertLink', { url, text }),
            removeLinks: (): void => sendEvent('removeLinks'),
            getLink: (): string =>
                composerModel?.get_link_action()?.edit_link?.url || '',
            codeBlock: (): void => sendEvent('insertCodeBlock'),
            quote: (): void => sendEvent('insertQuote'),
            indent: (): void => sendEvent('formatIndent'),
            unindent: (): void => sendEvent('formatOutdent'),
            mention: (
                url: string,
                text: string,
                attributes: AllowedMentionAttributes,
            ): void => sendEvent('insertSuggestion', { url, text, attributes }),
            command: (text: string): void => sendEvent('insertCommand', text),
            mentionAtRoom: (attributes: AllowedMentionAttributes): void =>
                sendEvent('insertAtRoomSuggestion', { attributes }),
        };
    }, [editorRef, composerModel]);

    return formattingFunctions;
}
