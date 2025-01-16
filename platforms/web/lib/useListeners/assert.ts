/*
Copyright 2024 New Vector Ltd.
Copyright 2022 The Matrix.org Foundation C.I.C.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

import { LinkEvent, SuggestionEvent } from './types';

export function isInputEvent(e: Event): e is InputEvent {
    return 'inputType' in e;
}

export function isClipboardEvent(e: Event): e is ClipboardEvent {
    return 'clipboardData' in e;
}

export function isSuggestionEvent(e: Event): e is SuggestionEvent {
    return isInputEvent(e) && e.inputType === 'insertSuggestion';
}

export function isAtRoomSuggestionEvent(e: Event): e is SuggestionEvent {
    return isInputEvent(e) && e.inputType === 'insertAtRoomSuggestion';
}

export function isLinkEvent(e: Event): e is LinkEvent {
    return isInputEvent(e) && e.inputType == 'insertLink';
}
