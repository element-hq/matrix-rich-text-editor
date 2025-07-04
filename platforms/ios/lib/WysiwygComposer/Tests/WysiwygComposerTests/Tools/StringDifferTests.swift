//
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
        XCTAssertNil(try StringDiffer.replacement(from: identicalText, to: identicalText))
    }

    func testSimpleRemoval() throws {
        XCTAssertEqual(try StringDiffer.replacement(from: "text", to: "te"),
                       .init(location: 2, length: 2, text: ""))
    }

    func testSimpleInsertion() throws {
        XCTAssertEqual(try StringDiffer.replacement(from: "te", to: "text"),
                       .init(location: 2, length: 0, text: "xt"))
    }

    func testFullReplacement() throws {
        XCTAssertEqual(try StringDiffer.replacement(from: "wa", to: "わ"),
                       .init(location: 0, length: 2, text: "わ"))
    }

    func testPartialReplacement() throws {
        XCTAssertEqual(try StringDiffer.replacement(from: "わta", to: "わた"),
                       .init(location: 1, length: 2, text: "た"))
    }

    func testDoubleReplacementIsNotHandled() throws {
        XCTAssertThrowsError(try StringDiffer.replacement(from: "text", to: "fexf"), "doubleReplacementIsNotHandled") { error in
            XCTAssertEqual(error as? StringDifferError,
                           StringDifferError.tooComplicated)
            XCTAssertEqual(error.localizedDescription,
                           StringDifferError.tooComplicated.localizedDescription)
        }
    }

    func testInsertionsDontMatchRemovalsLocation() throws {
        XCTAssertThrowsError(try StringDiffer.replacement(from: "text", to: "extab"), "insertionsDontMatchRemovalsLocation") { error in
            XCTAssertEqual(error as? StringDifferError,
                           StringDifferError.insertionsDontMatchRemovals)
            XCTAssertEqual(error.localizedDescription,
                           StringDifferError.insertionsDontMatchRemovals.localizedDescription)
        }
    }

    func testDifferentWhitespacesAreEquivalent() throws {
        let whitespaceCodeUnits = CharacterSet.whitespaces.codePoints()
        let whitespaceString = String(
            String(utf16CodeUnits: whitespaceCodeUnits, count: whitespaceCodeUnits.count)
                // We need to remove unicode characters that are related to whitespaces but have a property `White_space = no`
                .filter(\.isWhitespace)
        )
        XCTAssertNil(try StringDiffer.replacement(from: whitespaceString,
                                                  to: String(repeating: Character.nbsp, count: whitespaceString.utf16Length)))
    }

    func testDiffingWithLeadingWhitespaces() throws {
        XCTAssertEqual(try StringDiffer.replacement(from: " text", to: " test"),
                       .init(location: 3, length: 1, text: "s"))
    }

    func testDiffingWithMultipleLeadingWhitespaces() throws {
        XCTAssertEqual(try StringDiffer.replacement(from: " \u{00A0} text", to: " \u{00A0} test"),
                       .init(location: 5, length: 1, text: "s"))
    }
    
    func testDoubleSpaceDotConversion() throws {
        XCTAssertEqual(try StringDiffer.replacement(from: "a  ", to: "a."),
                       .init(location: 1, length: 2, text: "."))
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
