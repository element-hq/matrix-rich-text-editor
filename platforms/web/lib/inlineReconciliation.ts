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

/**
 * Inline reconciliation utilities: prefix/suffix diff over UTF-16 code units.
 *
 * This is the TypeScript port of `computePrefixSuffixDiff` from the iOS
 * `InlineReconciliation.swift` file.  It produces the minimal replacement
 * needed to turn `oldText` into `newText`, expressed as UTF-16 code unit
 * offsets — the same unit used throughout the Rust model and the
 * `BlockProjection` API.
 *
 * Because `ProjectionRenderer` guarantees that `editor.textContent` maps 1:1
 * to Rust UTF-16 offsets, the offsets returned here can be passed directly to
 * `composerModel.replace_text_in()` without any HTML range translation.
 */

export interface InlineDiff {
    /** UTF-16 offset of the first changed code unit (inclusive). */
    replaceStart: number;
    /** UTF-16 offset of the last changed code unit (exclusive). */
    replaceEnd: number;
    /** The replacement text (may be empty for deletions). */
    replacement: string;
}

/**
 * O(n) prefix/suffix diff over UTF-16 code units.
 *
 * Typical edits (single keystrokes, autocorrect) are small, so this is
 * cheaper than a full Myers diff and simpler to reason about.
 *
 * @param oldText  The committed text before the user's edit.
 * @param newText  The text inside the editor after the user's edit.
 */
export function computePrefixSuffixDiff(
    oldText: string,
    newText: string,
): InlineDiff {
    const oldUnits = toUtf16Units(oldText);
    const newUnits = toUtf16Units(newText);

    // Walk from the front to find the first differing code unit.
    let prefixLen = 0;
    while (
        prefixLen < oldUnits.length &&
        prefixLen < newUnits.length &&
        oldUnits[prefixLen] === newUnits[prefixLen]
    ) {
        prefixLen++;
    }

    // Walk from the back (staying within the non-prefix region) to find the
    // last differing code unit.
    let suffixLen = 0;
    while (
        suffixLen < oldUnits.length - prefixLen &&
        suffixLen < newUnits.length - prefixLen &&
        oldUnits[oldUnits.length - 1 - suffixLen] ===
            newUnits[newUnits.length - 1 - suffixLen]
    ) {
        suffixLen++;
    }

    const replaceEnd = oldUnits.length - suffixLen;
    const newEnd = newUnits.length - suffixLen;
    const replacementUnits = newUnits.slice(prefixLen, newEnd);

    return {
        replaceStart: prefixLen,
        replaceEnd,
        replacement: fromUtf16Units(replacementUnits),
    };
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/**
 * Convert a JS string to an array of UTF-16 code units.
 * JS strings are already UTF-16 internally, so this is simply a
 * code-unit-by-code-unit copy.
 */
function toUtf16Units(s: string): number[] {
    const units: number[] = new Array(s.length);
    for (let i = 0; i < s.length; i++) {
        units[i] = s.charCodeAt(i);
    }
    return units;
}

/**
 * Convert an array of UTF-16 code units back to a JS string.
 */
function fromUtf16Units(units: number[]): string {
    if (units.length === 0) return '';
    return String.fromCharCode(...units);
}
