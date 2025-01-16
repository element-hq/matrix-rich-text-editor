/*
Copyright 2024 New Vector Ltd.
Copyright 2022 The Matrix.org Foundation C.I.C.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

import { ComposerUpdate } from '@vector-im/matrix-wysiwyg-wasm';

import { ACTION_TYPES, SUGGESTIONS } from './constants';
import { AllowedMentionAttributes, LinkEvent } from './useListeners/types';

export type BlockType = InputEvent['inputType'] | 'formatInlineCode' | 'clear';

export type WysiwygInputEvent =
    | ClipboardEvent
    | LinkEvent
    | (InputEvent & {
          inputType: BlockType;
          data?: string | null;
      });

export type WysiwygEvent = WysiwygInputEvent | KeyboardEvent;

export type ActionTypes = (typeof ACTION_TYPES)[number];

export type ActionState = 'enabled' | 'reversed' | 'disabled';

export type AllActionStates = Record<ActionTypes, ActionState>;

export type FormattingFunctions = Record<
    Exclude<ActionTypes, 'link'>,
    () => void
> & {
    insertText: (text: string) => void;
    link: (url: string, text?: string) => void;
    mention: (
        url: string,
        text: string,
        attributes: AllowedMentionAttributes,
    ) => void;
    mentionAtRoom: (attributes: AllowedMentionAttributes) => void;
    command: (text: string) => void;
    removeLinks: () => void;
    getLink: () => string;
};

export type Wysiwyg = {
    actions: FormattingFunctions;
    content: () => string;
    messageContent: () => string;
};

export type InputEventProcessor = (
    event: WysiwygEvent,
    wysiwyg: Wysiwyg,
    editor: HTMLElement,
) => WysiwygEvent | null;

export type SuggestionChar = (typeof SUGGESTIONS)[number] | '';
export type SuggestionType = 'mention' | 'command' | 'custom' | 'unknown';

export type MappedSuggestion = {
    keyChar: SuggestionChar;
    text: string;
    type: SuggestionType;
};
export type TraceAction = (
    update: ComposerUpdate | null,
    name: string,
    value1?: string | number,
    value2?: string | number,
) => ComposerUpdate | null;
