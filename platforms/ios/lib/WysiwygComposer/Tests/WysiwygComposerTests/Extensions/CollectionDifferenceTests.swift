//
// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

@testable import WysiwygComposer
import XCTest

final class CollectionDifferenceTests: XCTestCase {
    func testNoChanges() {
        let changes = changes(from: "text", to: "text")
        XCTAssertTrue(changes.removals.isEmpty)
        XCTAssertTrue(changes.insertions.isEmpty)
    }

    func testSimpleRemoval() {
        let changes = changes(from: "text", to: "tex")
        XCTAssertEqual(changes.removals,
                       [NSRange(location: 3, length: 1)])
        XCTAssertTrue(changes.insertions.isEmpty)
    }

    func testMultipleRemovals() {
        let changes = changes(from: "text", to: "ex")
        XCTAssertEqual(changes.removals,
                       [NSRange(location: 0, length: 1),
                        NSRange(location: 3, length: 1)])
        XCTAssertTrue(changes.insertions.isEmpty)
    }

    func testSimpleInsertion() {
        let changes = changes(from: "tex", to: "text")
        XCTAssertEqual(changes.insertions.map(\.range),
                       [NSRange(location: 3, length: 1)])
        XCTAssertEqual(changes.insertions.map(\.text),
                       ["t"])
        XCTAssertTrue(changes.removals.isEmpty)
    }

    func testMultipleInsertions() {
        let changes = changes(from: "ex", to: "texts")
        XCTAssertEqual(changes.insertions.map(\.range),
                       [NSRange(location: 0, length: 1),
                        NSRange(location: 3, length: 2)])
        XCTAssertEqual(changes.insertions.map(\.text),
                       ["t", "ts"])
        XCTAssertTrue(changes.removals.isEmpty)
    }

    func testSimpleReplacement() {
        let changes = changes(from: "text", to: "tessst")
        XCTAssertEqual(changes.removals,
                       [NSRange(location: 2, length: 1)])
        XCTAssertEqual(changes.insertions.map(\.range),
                       [NSRange(location: 2, length: 3)])
        XCTAssertEqual(changes.insertions.map(\.text),
                       ["sss"])
    }

    func testMultipleReplacements() {
        let changes = changes(from: "text", to: "wexpf")
        XCTAssertEqual(changes.removals,
                       [NSRange(location: 0, length: 1),
                        NSRange(location: 3, length: 1)])
        XCTAssertEqual(changes.insertions.map(\.range),
                       [NSRange(location: 0, length: 1),
                        NSRange(location: 3, length: 2)])
        XCTAssertEqual(changes.insertions.map(\.text),
                       ["w", "pf"])
    }

    func testMultipleCodeUnitsReplacements() {
        let changes1 = changes(from: "abcde 🥳", to: "abcde")
        XCTAssertEqual(changes1.removals,
                       [NSRange(location: 5, length: 3)])
        let changes2 = changes(from: "abcde", to: "abcde 🥳")
        XCTAssertEqual(changes2.insertions.map(\.range),
                       [NSRange(location: 5, length: 3)])
        XCTAssertEqual(changes2.insertions.map(\.text),
                       [" 🥳"])
    }

    func testRemovalNearMultiCodeUnitsCharacters() {
        let changes = changes(from: "abcde 🥳 ", to: "abcde 🥳")
        XCTAssertEqual(changes.removals,
                       [NSRange(location: 8, length: 1)])
    }
}

private extension CollectionDifferenceTests {
    func removals(from oldText: String, to newText: String) -> UTF16Removals {
        let difference = newText.difference(from: oldText)
        return difference.utf16Removals(in: oldText)
    }

    func insertions(from oldText: String, to newText: String) -> UTF16Insertions {
        let difference = newText.difference(from: oldText)
        return difference.utf16Insertions(in: newText)
    }

    func changes(from oldText: String, to newText: String) -> (removals: UTF16Removals, insertions: UTF16Insertions) {
        let difference = newText.difference(from: oldText)
        return (difference.utf16Removals(in: oldText), difference.utf16Insertions(in: newText))
    }
}
