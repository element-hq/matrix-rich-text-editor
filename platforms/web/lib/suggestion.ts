/*
Copyright 2024 New Vector Ltd.
Copyright 2023 The Matrix.org Foundation C.I.C.

SPDX-License-Identifier: AGPL-3.0-only
Please see LICENSE in the repository root for full details.
*/

import { SuggestionPattern } from '@vector-im/matrix-wysiwyg-wasm';

import { SUGGESTIONS } from './constants';
import { MappedSuggestion, SuggestionChar, SuggestionType } from './types';

export function getSuggestionChar(
    suggestion: SuggestionPattern,
): SuggestionChar {
    return SUGGESTIONS[suggestion.key.key_type] || '';
}

export function getSuggestionType(
    suggestion: SuggestionPattern,
): SuggestionType {
    switch (suggestion.key.key_type) {
        case 0:
        case 1:
            return 'mention';
        case 2:
            return 'command';
        case 3:
            return 'custom';
        default:
            return 'unknown';
    }
}

export function mapSuggestion(
    suggestion: SuggestionPattern | null,
): MappedSuggestion | null {
    if (suggestion === null) return suggestion;
    return {
        text: suggestion.text,
        keyChar: getSuggestionChar(suggestion),
        type: getSuggestionType(suggestion),
    };
}
