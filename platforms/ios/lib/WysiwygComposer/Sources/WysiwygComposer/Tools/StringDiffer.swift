//
// Copyright 2025 Element Creations Ltd.
// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import Foundation
import OSLog

/// Describes the output replacement from string diffing.
struct StringDifferReplacement: Equatable {
    let range: NSRange
    let text: String
    let hasMore: Bool

    init(range: NSRange, text: String, hasMore: Bool) {
        self.range = range
        self.text = text
        self.hasMore = hasMore
    }

    init(location: Int, length: Int, text: String, hasMore: Bool) {
        range = NSRange(location: location, length: length)
        self.text = text
        self.hasMore = hasMore
    }
}

final class StringDiffer {
    // MARK: - Private

    private init() { }

    // MARK: - Internal

    /// Creates a simple diff replacement between the two provided strings. This differ treats all
    /// types of whitespaces as non-breaking spaces in order to avoid unneccessary positives due to
    /// Rust usage of non-breaking spaces in HTML output. Throws if the changes
    /// between the two strings are more than just a single located replacement.
    ///
    /// - Parameters:
    ///   - oldText: The old (previous) text.
    ///   - newText: The new text.
    /// - Returns: Replacement that have occured in order to get from `oldText` to `newText`. Nil if strings are identical.
    static func replacement(from oldText: String, to newText: String) -> StringDifferReplacement? {
        let difference = newText.withNBSP.utf16Difference(from: oldText.withNBSP)

        guard !difference.isEmpty else {
            return nil
        }

        if let removal = difference.removals.first {
            if let insertion = difference.insertions.first(where: { $0.range.location == removal.location }) {
                return StringDifferReplacement(range: removal, text: insertion.text, hasMore: difference.isComplex)
            } else {
                // Simple removal
                return StringDifferReplacement(range: removal, text: "", hasMore: difference.isComplex || !difference.insertions.isEmpty)
            }
        } else if let insertion = difference.insertions.first {
            // Simple insertion
            return StringDifferReplacement(location: insertion.range.location, length: 0, text: insertion.text, hasMore: difference.isComplex)
        } else {
            fatalError("Should never happen => difference is empty")
        }
    }
}

// MARK: - Private

/// Describes a diff from a string to another, with UTF16 locations and lengths.
private struct StringDiff {
    let removals: UTF16Removals
    let insertions: UTF16Insertions

    /// Returns true if there is no actual changes in the diff.
    var isEmpty: Bool {
        removals.isEmpty && insertions.isEmpty
    }

    /// Returns true if the diff includes multiple removals or insertions.
    var isComplex: Bool {
        removals.count > 1 || insertions.count > 1
    }
}

private extension String {
    /// Computes the diff from provided string to self. Outputs UTF16 locations and lengths.
    ///
    /// - Parameter otherString: Other string to compare.
    /// - Returns: Result diff (UTF16).
    func utf16Difference(from otherString: String) -> StringDiff {
        let difference = difference(from: otherString)
        return StringDiff(removals: difference.utf16Removals(in: otherString),
                          insertions: difference.utf16Insertions(in: self))
    }
}
