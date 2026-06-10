//
// Copyright 2025 Element Creations Ltd.
// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import Foundation
import Testing
@testable import WysiwygComposer

@MainActor
struct StringDifferTests {
    @Test func noReplacement() {
        let identicalText = "text"
        #expect(StringDiffer.replacement(from: identicalText, to: identicalText) == nil)
    }

    @Test func simpleRemoval() {
        #expect(StringDiffer.replacement(from: "text", to: "te") ==
            .init(location: 2, length: 2, text: "", hasMore: false))
    }

    @Test func simpleInsertion() {
        #expect(StringDiffer.replacement(from: "te", to: "text") ==
            .init(location: 2, length: 0, text: "xt", hasMore: false))
    }

    @Test func fullReplacement() {
        #expect(StringDiffer.replacement(from: "wa", to: "わ") ==
            .init(location: 0, length: 2, text: "わ", hasMore: false))
    }

    @Test func partialReplacement() {
        #expect(StringDiffer.replacement(from: "わta", to: "わた") ==
            .init(location: 1, length: 2, text: "た", hasMore: false))
    }

    @Test func doubleReplacementIsHandledOneAtTime() {
        #expect(StringDiffer.replacement(from: "text", to: "fexf") ==
            .init(location: 0, length: 1, text: "f", hasMore: true))
        // Simulate the change
        #expect(StringDiffer.replacement(from: "fext", to: "fexf") ==
            .init(location: 3, length: 1, text: "f", hasMore: false))
    }

    @Test func nonMatchingRemovalAndInsertionsAreHandledOneAtTime() {
        #expect(StringDiffer.replacement(from: "text", to: "extab") ==
            .init(location: 0, length: 1, text: "", hasMore: true))
        // Simulate the change
        #expect(StringDiffer.replacement(from: "ext", to: "extab") ==
            .init(location: 3, length: 0, text: "ab", hasMore: false))
    }

    @Test func differentWhitespacesAreEquivalent() {
        let whitespaceCodeUnits = CharacterSet.whitespaces.codePoints()
        let whitespaceString = String(
            String(utf16CodeUnits: whitespaceCodeUnits, count: whitespaceCodeUnits.count)
                // We need to remove unicode characters that are related to whitespaces but have a property `White_space = no`
                .filter(\.isWhitespace)
        )
        #expect(StringDiffer.replacement(from: whitespaceString,
                                         to: String(repeating: Character.nbsp, count: whitespaceString.utf16Length)) == nil)
    }

    @Test func diffingWithLeadingWhitespaces() {
        #expect(StringDiffer.replacement(from: " text", to: " test") ==
            .init(location: 3, length: 1, text: "s", hasMore: false))
    }

    @Test func diffingWithMultipleLeadingWhitespaces() {
        #expect(StringDiffer.replacement(from: " \u{00A0} text", to: " \u{00A0} test") ==
            .init(location: 5, length: 1, text: "s", hasMore: false))
    }

    @Test func doubleSpaceDotConversion() {
        #expect(StringDiffer.replacement(from: "a  ", to: "a.") ==
            .init(location: 1, length: 2, text: ".", hasMore: false))
    }
}

private extension CharacterSet {
    func codePoints() -> [UInt16] {
        var result: [Int] = []
        var plane = 0
        for (i, w) in bitmapRepresentation.enumerated() {
            let k = i % 8193
            if k == 8192 {
                plane = Int(w) << 13
                continue
            }
            let base = (plane + k) << 3
            for j in 0..<8 where w & 1 << j != 0 {
                result.append(base + j)
            }
        }
        return result.map { UInt16($0) }
    }
}
