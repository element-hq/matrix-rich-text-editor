/*
Copyright 2026 The Matrix.org Foundation C.I.C.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

import { describe, it, expect } from 'vitest';
import { computePrefixSuffixDiff } from './inlineReconciliation';

describe('computePrefixSuffixDiff', () => {
    it('returns empty diff for identical strings', () => {
        const result = computePrefixSuffixDiff('hello', 'hello');
        expect(result).toEqual({
            replaceStart: 5,
            replaceEnd: 5,
            replacement: '',
        });
    });

    it('detects a single character insertion at end', () => {
        const result = computePrefixSuffixDiff('hello', 'helloo');
        expect(result).toEqual({
            replaceStart: 5,
            replaceEnd: 5,
            replacement: 'o',
        });
    });

    it('detects a single character insertion at start', () => {
        const result = computePrefixSuffixDiff('ello', 'hello');
        expect(result).toEqual({
            replaceStart: 0,
            replaceEnd: 0,
            replacement: 'h',
        });
    });

    it('detects a single character insertion in the middle', () => {
        const result = computePrefixSuffixDiff('helo', 'hello');
        expect(result).toEqual({
            replaceStart: 3,
            replaceEnd: 3,
            replacement: 'l',
        });
    });

    it('detects a single character deletion at end', () => {
        const result = computePrefixSuffixDiff('hello', 'hell');
        expect(result).toEqual({
            replaceStart: 4,
            replaceEnd: 5,
            replacement: '',
        });
    });

    it('detects a single character deletion in the middle', () => {
        const result = computePrefixSuffixDiff('hello', 'helo');
        expect(result).toEqual({
            replaceStart: 3,
            replaceEnd: 4,
            replacement: '',
        });
    });

    it('handles complete replacement', () => {
        const result = computePrefixSuffixDiff('foo', 'bar');
        expect(result).toEqual({
            replaceStart: 0,
            replaceEnd: 3,
            replacement: 'bar',
        });
    });

    it('handles insertion into empty string', () => {
        const result = computePrefixSuffixDiff('', 'abc');
        expect(result).toEqual({
            replaceStart: 0,
            replaceEnd: 0,
            replacement: 'abc',
        });
    });

    it('handles deletion to empty string', () => {
        const result = computePrefixSuffixDiff('abc', '');
        expect(result).toEqual({
            replaceStart: 0,
            replaceEnd: 3,
            replacement: '',
        });
    });

    it('handles surrogate pairs (emoji) correctly', () => {
        // 😀 is U+1F600, encoded as two UTF-16 code units: 0xD83D 0xDE00
        const emoji = '\uD83D\uDE00';
        const result = computePrefixSuffixDiff(`hello${emoji}`, `hello${emoji}world`);
        expect(result).toEqual({
            replaceStart: 7, // 'hello' (5) + emoji (2 UTF-16 units)
            replaceEnd: 7,
            replacement: 'world',
        });
    });

    it('handles autocorrect-style word replacement', () => {
        const result = computePrefixSuffixDiff('teh ', 'the ');
        expect(result).toEqual({
            replaceStart: 1,
            replaceEnd: 3,
            replacement: 'he',
        });
    });
});
