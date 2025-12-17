//
// Copyright 2025 Element Creations Ltd.
// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

@testable import WysiwygComposer
import XCTest

final class StringDifferTests: XCTestCase {
    func testNoReplacement() throws {
        let identicalText = "text"
        XCTAssertNil(StringDiffer.replacement(from: identicalText, to: identicalText))
    }

    func testSimpleRemoval() throws {
        XCTAssertEqual(StringDiffer.replacement(from: "text", to: "te"),
                       .init(location: 2, length: 2, text: "", hasMore: false))
    }

    func testSimpleInsertion() throws {
        XCTAssertEqual(StringDiffer.replacement(from: "te", to: "text"),
                       .init(location: 2, length: 0, text: "xt", hasMore: false))
    }

    func testFullReplacement() throws {
        XCTAssertEqual(StringDiffer.replacement(from: "wa", to: "わ"),
                       .init(location: 0, length: 2, text: "わ", hasMore: false))
    }

    func testPartialReplacement() throws {
        XCTAssertEqual(StringDiffer.replacement(from: "わta", to: "わた"),
                       .init(location: 1, length: 2, text: "た", hasMore: false))
    }

    func testDoubleReplacementIsHandledOneAtTime() throws {
        XCTAssertEqual(StringDiffer.replacement(from: "text", to: "fexf"),
                       .init(location: 0, length: 1, text: "f", hasMore: true))
        // Simulate the change
        XCTAssertEqual(StringDiffer.replacement(from: "fext", to: "fexf"),
                       .init(location: 3, length: 1, text: "f", hasMore: false))
    }

    func testNonMatchingRemovalAndInsertionsAreHandledOneAtTime() throws {
        XCTAssertEqual(StringDiffer.replacement(from: "text", to: "extab"),
                       .init(location: 0, length: 1, text: "", hasMore: true))
        // Simulate the change
        XCTAssertEqual(StringDiffer.replacement(from: "ext", to: "extab"),
                       .init(location: 3, length: 0, text: "ab", hasMore: false))
    }

    func testDifferentWhitespacesAreEquivalent() throws {
        let whitespaceCodeUnits = CharacterSet.whitespaces.codePoints()
        let whitespaceString = String(
            String(utf16CodeUnits: whitespaceCodeUnits, count: whitespaceCodeUnits.count)
                // We need to remove unicode characters that are related to whitespaces but have a property `White_space = no`
                .filter(\.isWhitespace)
        )
        XCTAssertNil(StringDiffer.replacement(from: whitespaceString,
                                              to: String(repeating: Character.nbsp, count: whitespaceString.utf16Length)))
    }

    func testDiffingWithLeadingWhitespaces() throws {
        XCTAssertEqual(StringDiffer.replacement(from: " text", to: " test"),
                       .init(location: 3, length: 1, text: "s", hasMore: false))
    }

    func testDiffingWithMultipleLeadingWhitespaces() throws {
        XCTAssertEqual(StringDiffer.replacement(from: " \u{00A0} text", to: " \u{00A0} test"),
                       .init(location: 5, length: 1, text: "s", hasMore: false))
    }

    func testDoubleSpaceDotConversion() throws {
        XCTAssertEqual(StringDiffer.replacement(from: "a  ", to: "a."),
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
