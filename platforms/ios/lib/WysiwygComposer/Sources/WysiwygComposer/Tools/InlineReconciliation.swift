//
// Copyright (c) 2026 Element Creations Ltd
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import Foundation
import WysiwygComposerFFI

// MARK: - Data types

/// The result of a prefix/suffix diff between two strings.
public struct InlineDiff {
    /// Offset (in UTF-16 code units, relative to block start) of the first changed code unit.
    public let replaceStart: Int
    /// Offset (in UTF-16 code units, relative to block start) of the exclusive end of the replaced range.
    public let replaceEnd: Int
    /// The replacement text (UTF-16 code units).
    public let replacement: String
}

// MARK: - Inline diff

/// O(n) prefix/suffix diff over UTF-16 code units.
///
/// Since edits are always block-scoped and typically small (single keystrokes,
/// autocorrect replacements) this is cheaper than a full Myers diff.
///
/// - Parameters:
///   - old: The old block string before the UIKit edit.
///   - new: The new block string after the UIKit edit.
/// - Returns: The minimal replacement range and text.
public func computePrefixSuffixDiff(old: String, new: String) -> InlineDiff {
    let oldUnits = Array(old.utf16)
    let newUnits = Array(new.utf16)

    var prefixLen = 0
    while prefixLen < oldUnits.count, prefixLen < newUnits.count,
          oldUnits[prefixLen] == newUnits[prefixLen] {
        prefixLen += 1
    }

    var suffixLen = 0
    while suffixLen < (oldUnits.count - prefixLen),
          suffixLen < (newUnits.count - prefixLen),
          oldUnits[oldUnits.count - 1 - suffixLen] == newUnits[newUnits.count - 1 - suffixLen] {
        suffixLen += 1
    }

    let replaceEnd = oldUnits.count - suffixLen
    let newEnd = newUnits.count - suffixLen
    let replacementUnits = Array(newUnits[prefixLen..<newEnd])

    let replacement: String
    if replacementUnits.isEmpty {
        replacement = ""
    } else {
        replacement = String(decoding: replacementUnits, as: UTF16.self)
    }

    return InlineDiff(
        replaceStart: prefixLen,
        replaceEnd: replaceEnd,
        replacement: replacement
    )
}
