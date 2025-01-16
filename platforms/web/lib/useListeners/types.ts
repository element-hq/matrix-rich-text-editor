/*
Copyright 2024 New Vector Ltd.
Copyright 2022 The Matrix.org Foundation C.I.C.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

import { BlockType } from '../types';

export type FormatBlockEvent = CustomEvent<{
    blockType: BlockType;
    data?: string;
}>;

export type LinkEvent = Omit<InputEvent, 'data'> & {
    inputType: 'insertLink';
    data: { url: string; text?: string };
};

export type AllowedMentionAttributes = Map<
    'style' | 'data-mention-type',
    string
>;

export type SuggestionEvent = Omit<InputEvent, 'data'> & {
    inputType: 'insertSuggestion';
    data: { url: string; text: string; attributes: AllowedMentionAttributes };
};

export type AtRoomSuggestionEvent = Omit<InputEvent, 'data'> & {
    inputType: 'insertAtRoomSuggestion';
    data: { attributes: AllowedMentionAttributes };
};
