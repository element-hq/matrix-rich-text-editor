/*
Copyright 2024 New Vector Ltd.
Copyright 2022 The Matrix.org Foundation C.I.C.

SPDX-License-Identifier: AGPL-3.0-only
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
            editorRef.current &&
                sendWysiwygInputEvent(
                    editorRef.current,
                    blockType,
                    undefined,
                    data,
                );
        };

        return {
            bold: () => sendEvent('formatBold'),
            italic: () => sendEvent('formatItalic'),
            strikeThrough: () => sendEvent('formatStrikeThrough'),
            underline: () => sendEvent('formatUnderline'),
            undo: () => sendEvent('historyUndo'),
            redo: () => sendEvent('historyRedo'),
            orderedList: () => sendEvent('insertOrderedList'),
            unorderedList: () => sendEvent('insertUnorderedList'),
            inlineCode: () => sendEvent('formatInlineCode'),
            clear: () => sendEvent('clear'),
            insertText: (text: string) => sendEvent('insertText', text),
            link: (url: string, text?: string) =>
                sendEvent('insertLink', { url, text }),
            removeLinks: () => sendEvent('removeLinks'),
            getLink: () =>
                composerModel?.get_link_action()?.edit_link?.url || '',
            codeBlock: () => sendEvent('insertCodeBlock'),
            quote: () => sendEvent('insertQuote'),
            indent: () => sendEvent('formatIndent'),
            unindent: () => sendEvent('formatOutdent'),
            mention: (
                url: string,
                text: string,
                attributes: AllowedMentionAttributes,
            ) => sendEvent('insertSuggestion', { url, text, attributes }),
            command: (text: string) => sendEvent('insertCommand', text),
            mentionAtRoom: (attributes: AllowedMentionAttributes) =>
                sendEvent('insertAtRoomSuggestion', { attributes }),
        };
    }, [editorRef, composerModel]);

    return formattingFunctions;
}
