/*
Copyright 2024 New Vector Ltd.
Copyright 2022 The Matrix.org Foundation C.I.C.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

import { initAsync } from '@vector-im/matrix-wysiwyg-wasm';

import { Actions } from './types';
import {
    escapeHtml,
    generateTestCase,
    getSelectionAccordingToActions,
} from './utils';

beforeAll(initAsync);

describe('getSelectionAccordingToActions', () => {
    it('Should return -1, -1 for selection when there are no actions', () => {
        // When
        const actions: Actions = [];
        const selection = getSelectionAccordingToActions(actions)();

        // Then
        expect(selection).toStrictEqual([-1, -1]);
    });

    it('Should find selection from the last action', () => {
        // When
        const actions: Actions = [
            ['foo', 'bar', 'baz'],
            ['select', 10, 10],
            ['foo', 'bar', 'baz'],
            ['select', 12, 13],
            ['foo', 'bar', 'baz'],
        ];
        const selection = getSelectionAccordingToActions(actions)();

        // Then
        expect(selection).toStrictEqual([12, 13]);
    });
});

describe('generateTestCase', () => {
    it('Should generate test case of 1 character and selection', () => {
        // When
        const actions: Actions = [
            ['replace_text', 'a', undefined],
            ['select', 1, 1],
        ];

        const expected =
            'let mut model = cm("a|");\n' + 'assert_eq!(tx(&model), "a|");\n';

        const testCase = generateTestCase(actions, 'a|');

        // Then
        expect(testCase).toBe(expected);
    });

    it('Should Generate test case with cursor at the beginning', () => {
        // When
        const actions: Actions = [
            ['replace_text', 'a', undefined],
            ['select', 0, 0],
        ];

        const expected =
            'let mut model = cm("|a");\n' + 'assert_eq!(tx(&model), "|a");\n';

        const testCase = generateTestCase(actions, '|a');

        // Then
        expect(testCase).toBe(expected);
    });

    it('Should generate test case from multiple typed characters', () => {
        // When
        const actions: Actions = [
            ['replace_text', 'a', undefined],
            ['replace_text', 'b', undefined],
            ['replace_text', 'c', undefined],
            ['replace_text', 'd', undefined],
            ['select', 4, 4],
        ];

        const expected =
            'let mut model = cm("abcd|");\n' +
            'assert_eq!(tx(&model), "abcd|");\n';

        const testCase = generateTestCase(actions, 'abcd|');

        // Then
        expect(testCase).toBe(expected);
    });

    it('Should generate test case collecting initial selections', () => {
        // When
        const actions: Actions = [
            ['replace_text', 'a', undefined],
            ['select', 1, 1],
            ['replace_text', 'b', undefined],
            ['select', 2, 2],
            ['replace_text', 'c', undefined],
            ['select', 3, 3],
            ['replace_text', 'd', undefined],
            ['select', 4, 4],
        ];

        const expected =
            'let mut model = cm("abcd|");\n' +
            'assert_eq!(tx(&model), "abcd|");\n';

        const testCase = generateTestCase(actions, 'abcd|');

        // Then
        expect(testCase).toBe(expected);
    });

    it('Should generate test case with pasted start', () => {
        // When
        const actions: Actions = [
            ['replace_text', 'abcd', undefined],
            ['select', 4, 4],
        ];

        const expected =
            'let mut model = cm("abcd|");\n' +
            'assert_eq!(tx(&model), "abcd|");\n';

        const testCase = generateTestCase(actions, 'abcd|');

        // Then
        expect(testCase).toBe(expected);
    });

    it('Should generate test case by typing and bolding', () => {
        // When
        const actions: Actions = [
            ['replace_text', 'a', undefined],
            ['replace_text', 'b', undefined],
            ['replace_text', 'c', undefined],
            ['replace_text', 'd', undefined],
            ['select', 1, 3],
            ['bold'],
        ];

        const expected =
            'let mut model = cm("a{bc}|d");\n' +
            'model.bold();\n' +
            'assert_eq!(tx(&model), "a<strong>{bc}|</strong>d");\n';

        const testCase = generateTestCase(actions, 'a<strong>{bc}|</strong>d');

        // Then
        expect(testCase).toBe(expected);
    });

    it('Should generate test case with backward selection', () => {
        // When
        const actions: Actions = [
            ['replace_text', 'a', undefined],
            ['replace_text', 'b', undefined],
            ['replace_text', 'c', undefined],
            ['replace_text', 'd', undefined],
            ['select', 3, 1],
            ['bold'],
        ];

        const expected =
            'let mut model = cm("a|{bc}d");\n' +
            'model.bold();\n' +
            'assert_eq!(tx(&model), "a<strong>|{bc}</strong>d");\n';

        const testCase = generateTestCase(actions, 'a<strong>|{bc}</strong>d');

        // Then
        expect(testCase).toBe(expected);
    });

    it('Should generate test case with backward to beginning', () => {
        // When
        const actions: Actions = [
            ['replace_text', 'a', undefined],
            ['replace_text', 'b', undefined],
            ['replace_text', 'c', undefined],
            ['replace_text', 'd', undefined],
            ['select', 3, 0],
            ['bold'],
        ];

        const expected =
            'let mut model = cm("|{abc}d");\n' +
            'model.bold();\n' +
            'assert_eq!(tx(&model), "<strong>|{abc}</strong>d");\n';

        const testCase = generateTestCase(actions, '<strong>|{abc}</strong>d');

        // Then
        expect(testCase).toBe(expected);
    });

    it('Should generate test case with backward from end', () => {
        // When
        const actions: Actions = [
            ['replace_text', 'abc', undefined],
            ['select', 3, 2],
        ];

        const expected =
            'let mut model = cm("ab|{c}");\n' +
            'assert_eq!(tx(&model), "ab|{c}");\n';

        const testCase = generateTestCase(actions, 'ab|{c}');

        // Then
        expect(testCase).toBe(expected);
    });

    it('Should generate test case with tags on selection boundary', () => {
        // When
        const actions: Actions = [
            ['replace_text', 'aa<strong>bbbb</strong>cc', undefined],
            ['select', 2, 6],
        ];

        const expected =
            'let mut model = ' +
            'cm("aa{&lt;}|strong&gt;bbbb&lt;/strong&gt;cc");\n' +
            'assert_eq!(tx(&model), ' +
            '"aa&lt;strong&gt;{bbbb}|&lt;/strong&gt;cc");\n';

        const testCase = generateTestCase(
            actions,
            'aa&lt;strong&gt;{bbbb}|&lt;/strong&gt;cc',
        );

        // Then
        expect(testCase).toBe(expected);
    });

    it('Should generate test case with multiple later selections', () => {
        // When
        const actions: Actions = [
            ['replace_text', 'aa<strong>bbbb</strong>cc', undefined],
            ['select', 2, 6],
            ['bold'],
            ['select', 3, 3],
            ['select', 3, 5],
            ['select', 4, 4],
            ['select', 3, 6],
        ];

        const expected =
            'let mut model = ' +
            'cm("aa{&lt;}|strong&gt;bbbb&lt;/strong&gt;cc");\n' +
            'model.bold();\n' +
            'model.select(Location::from(3), Location::from(6));\n' +
            'assert_eq!(tx(&model), ' +
            '"aa{bbbb}|cc");\n';

        const testCase = generateTestCase(actions, 'aa{bbbb}|cc');

        // Then
        expect(testCase).toBe(expected);
    });

    it('Should generate test case later selections to beginning', () => {
        // When
        const actions: Actions = [
            ['replace_text', 'aa<strong>bbbb</strong>cc', undefined],
            ['select', 2, 6],
            ['bold'],
            ['select', 3, 0],
        ];

        const expected =
            'let mut model = ' +
            'cm("aa{&lt;}|strong&gt;bbbb&lt;/strong&gt;cc");\n' +
            'model.bold();\n' +
            'model.select(Location::from(3), Location::from(0));\n' +
            'assert_eq!(tx(&model), ' +
            '"|{aab}bbbcc");\n';

        const testCase = generateTestCase(actions, '|{aab}bbbcc');

        // Then
        expect(testCase).toBe(expected);
    });

    it('Should generate test with cursor after backspace', () => {
        // When
        const actions: Actions = [
            ['replace_text', 'aa<strong>bbbb</strong>cc', undefined],
            ['select', 8, 8],
            ['backspace'],
            ['backspace'],
        ];

        const expected =
            'let mut model = cm("aa&lt;st|rong&gt;bbbb&lt;/strong&gt;cc");\n' +
            'model.backspace();\n' +
            'model.backspace();\n' +
            'assert_eq!(tx(&model), "aa&lt;stron|g&gt;bbbb&lt;/strong&gt;");\n';

        const testCase = generateTestCase(
            actions,
            'aa&lt;stron|g&gt;bbbb&lt;/strong&gt;',
        );

        // Then
        expect(testCase).toBe(expected);
    });
});

describe('escapeHtml', () => {
    it('should return empty string for undefined input', () => {
        expect(escapeHtml(undefined)).toBe('');
    });

    it('should leave plain strings unmodified', () => {
        expect(escapeHtml('foo bar\nbaz')).toBe('foo bar\nbaz');
    });

    it('should escape HTML tags', () => {
        expect(escapeHtml('a <b>B</b> c')).toBe('a &lt;b&gt;B&lt;/b&gt; c');
    });
});
