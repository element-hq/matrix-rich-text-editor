/*
Copyright 2024 New Vector Ltd.
Copyright 2022 The Matrix.org Foundation C.I.C.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

import {
    ComposerModel,
    ComposerUpdate,
    SuggestionPattern,
} from '@vector-im/matrix-wysiwyg-wasm';

import {
    WysiwygInputEvent,
    InputEventProcessor,
    Wysiwyg,
    FormattingFunctions,
    WysiwygEvent,
} from './types';
import {
    isAtRoomSuggestionEvent,
    isClipboardEvent,
    isLinkEvent,
    isSuggestionEvent,
} from './useListeners/assert';
import { TestUtilities } from './useTestCases/types';

export function processEvent<T extends WysiwygEvent>(
    e: T,
    wysiwyg: Wysiwyg,
    editor: HTMLElement,
    inputEventProcessor?: InputEventProcessor,
): T | null {
    if (inputEventProcessor) {
        return inputEventProcessor(e, wysiwyg, editor) as T | null;
    } else {
        return e;
    }
}

export function processInput(
    e: WysiwygInputEvent,
    composerModel: ComposerModel,
    action: TestUtilities['traceAction'],
    formattingFunctions: FormattingFunctions,
    editor: HTMLElement,
    suggestion: SuggestionPattern | null,
    inputEventProcessor?: InputEventProcessor,
    emojiSuggestions?: Map<string, string>,
): ComposerUpdate | null | undefined {
    const event = processEvent(
        e,
        {
            actions: formattingFunctions,
            content: () => composerModel.get_content_as_html(),
            messageContent: () => composerModel.get_content_as_message_html(),
        },
        editor,
        inputEventProcessor,
    );
    if (!event) {
        return;
    }

    if (isClipboardEvent(event)) {
        const data = event.clipboardData?.getData('text/plain') ?? '';
        return action(composerModel.replace_text(data), 'paste');
    }

    switch (event.inputType) {
        case 'insertAtRoomSuggestion': {
            if (suggestion && isAtRoomSuggestionEvent(event)) {
                const { attributes } = event.data;
                // we need to track data-mention-type in element web, ensure we do not pass
                // it in as rust model can handle this automatically
                if (attributes.has('data-mention-type')) {
                    attributes.delete('data-mention-type');
                }
                return action(
                    composerModel.insert_at_room_mention_at_suggestion(
                        suggestion,
                        attributes,
                    ),
                    'insert_at_room_mention_at_suggestion',
                );
            }
            break;
        }
        case 'insertSuggestion': {
            if (suggestion && isSuggestionEvent(event)) {
                const { text, url, attributes } = event.data;
                // we need to track data-mention-type in element web, ensure we do not pass
                // it in as rust model can handle this automatically
                if (attributes.has('data-mention-type')) {
                    attributes.delete('data-mention-type');
                }
                return action(
                    composerModel.insert_mention_at_suggestion(
                        url,
                        text,
                        suggestion,
                        attributes,
                    ),
                    'insert_mention_at_suggestion',
                );
            }
            break;
        }
        case 'insertCommand': {
            if (suggestion && event.data) {
                return action(
                    composerModel.replace_text_suggestion(
                        event.data,
                        suggestion,
                        true,
                    ),
                    'replace_text_suggestion',
                );
            }
            break;
        }
        case 'clear':
            return action(composerModel.clear(), 'clear');
        case 'deleteContentBackward':
            return action(composerModel.backspace(), 'backspace');
        case 'deleteWordBackward':
            return action(composerModel.backspace_word(), 'backspace_word');
        case 'deleteSoftLineBackward': {
            const selection = document.getSelection();
            if (selection) {
                selection.modify('extend', 'backward', 'lineboundary');
                document.dispatchEvent(new CustomEvent('selectionchange'));
            }
            return action(composerModel.delete(), 'backspace_line');
        }
        case 'deleteContentForward':
            return action(composerModel.delete(), 'delete');
        case 'deleteWordForward':
            return action(composerModel.delete_word(), 'delete_word');
        case 'deleteByCut':
            return action(composerModel.delete(), 'delete');
        case 'formatBold':
            return action(composerModel.bold(), 'bold');
        case 'formatItalic':
            return action(composerModel.italic(), 'italic');
        case 'formatStrikeThrough':
            return action(composerModel.strike_through(), 'strike_through');
        case 'formatUnderline':
            return action(composerModel.underline(), 'underline');
        case 'formatInlineCode':
            return action(composerModel.inline_code(), 'inline_code');
        case 'historyRedo':
            return action(composerModel.redo(), 'redo');
        case 'historyUndo':
            return action(composerModel.undo(), 'undo');
        case 'insertCodeBlock':
            return action(composerModel.code_block(), 'code_block');
        case 'insertQuote':
            return action(composerModel.quote(), 'quote');
        case 'insertFromPaste':
            // Paste is already handled by catching the 'paste' event, which
            // results in a ClipboardEvent, handled above. Ideally, we would
            // do it here, but Chrome does not provide data inside this
            // InputEvent, only in the original ClipboardEvent.
            return;
        case 'insertOrderedList':
            return action(composerModel.ordered_list(), 'ordered_list');
        case 'insertLineBreak':
        case 'insertParagraph':
            insertAnyEmojiSuggestions(
                composerModel,
                suggestion,
                emojiSuggestions,
            );
            return action(composerModel.enter(), 'enter');
        case 'insertReplacementText': {
            // Remove br tag
            const newContent = editor.innerHTML.slice(
                0,
                editor.innerHTML.length - 4,
            );
            return action(
                composerModel.set_content_from_html(newContent),
                'set_content_from_html',
                newContent,
            );
        }
        case 'insertCompositionText':
        case 'insertFromComposition':
        case 'insertText':
            if (event.data) {
                if (event.data == ' ') {
                    insertAnyEmojiSuggestions(
                        composerModel,
                        suggestion,
                        emojiSuggestions,
                    );
                }
                return action(
                    composerModel.replace_text(event.data),
                    'replace_text',
                    event.data,
                );
            }
            break;
        case 'insertUnorderedList':
            return action(composerModel.unordered_list(), 'unordered_list');
        case 'insertLink':
            if (isLinkEvent(event)) {
                const { text, url } = event.data;
                return action(
                    text
                        ? composerModel.set_link_with_text(url, text, new Map())
                        : composerModel.set_link(url, new Map()),
                    'insertLink',
                );
            }
            break;
        case 'removeLinks':
            return action(composerModel.remove_links(), 'remove_links');
        case 'formatIndent':
            return action(composerModel.indent(), 'indent');
        case 'formatOutdent':
            return action(composerModel.unindent(), 'unindent');
        case 'sendMessage':
            // We create this event type when the user presses Ctrl+Enter.
            // We don't do anythign here, but the user may want to hook in
            // using inputEventProcessor to perform behaviour here.
            return null;
        default:
            // We should cover all of
            // eslint-disable-next-line max-len
            // https://rawgit.com/w3c/input-events/v1/index.html#interface-InputEvent-Attributes
            // Internal task to make sure we cover all inputs: PSU-740
            console.error(`Unknown input type: ${event.inputType}`);
            console.error(e);
            return null;
    }

    function insertAnyEmojiSuggestions(
        composerModel: ComposerModel,
        suggestion: SuggestionPattern | null,
        emojiSuggestions?: Map<string, string>,
    ): void {
        if (
            emojiSuggestions &&
            suggestion &&
            suggestion.key.key_type == 3 &&
            suggestion.key.custom_key_value
        ) {
            const emoji = emojiSuggestions.get(suggestion.key.custom_key_value);
            if (emoji) {
                composerModel.replace_text_suggestion(emoji, suggestion, false);
            }
        }
    }
}
