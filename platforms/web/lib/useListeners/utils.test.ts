/*
Copyright 2024 New Vector Ltd.
Copyright 2022 The Matrix.org Foundation C.I.C.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

import { createDefaultActionStates, mapToAllActionStates } from './utils';

describe('createDefaultActionStates', () => {
    it('Should return all the action state with enabled as value', () => {
        // When
        const states = createDefaultActionStates();

        // Then
        expect(states).toStrictEqual({
            bold: 'enabled',
            italic: 'enabled',
            strikeThrough: 'enabled',
            underline: 'enabled',
            clear: 'enabled',
            inlineCode: 'enabled',
            undo: 'enabled',
            redo: 'enabled',
            orderedList: 'enabled',
            unorderedList: 'enabled',
            link: 'enabled',
            codeBlock: 'enabled',
            quote: 'enabled',
            indent: 'enabled',
            unindent: 'enabled',
        });
    });
});

describe('mapToAllActionStates', () => {
    it('Should convert the map to an AllActionStates object', () => {
        // When
        const map = new Map([
            ['Bold', 'eNabled'],
            ['Italic', 'reversed'],
            ['Underline', 'DISABLED'],
            ['InlineCode', 'ENABLED'],
            ['OrderedList', 'ENABLED'],
            ['UnorderedList', 'ENABLED'],
            ['StrikeThrough', 'ENABLED'],
        ]);
        const states = mapToAllActionStates(map);

        // Then
        expect(states).toStrictEqual({
            bold: 'enabled',
            italic: 'reversed',
            underline: 'disabled',
            inlineCode: 'enabled',
            orderedList: 'enabled',
            unorderedList: 'enabled',
            strikeThrough: 'enabled',
        });
    });
});
