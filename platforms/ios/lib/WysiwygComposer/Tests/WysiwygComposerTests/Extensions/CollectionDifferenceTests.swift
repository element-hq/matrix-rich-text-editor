//
// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import Foundation
import Testing
@testable import WysiwygComposer

struct CollectionDifferenceTests {
    @Test func noChanges() {
        let changes = changes(from: "text", to: "text")
        #expect(changes.removals.isEmpty)
        #expect(changes.insertions.isEmpty)
    }

    @Test func simpleRemoval() {
        let changes = changes(from: "text", to: "tex")
        #expect(changes.removals ==
            [NSRange(location: 3, length: 1)])
        #expect(changes.insertions.isEmpty)
    }

    @Test func multipleRemovals() {
        let changes = changes(from: "text", to: "ex")
        #expect(changes.removals ==
            [NSRange(location: 0, length: 1),
             NSRange(location: 3, length: 1)])
        #expect(changes.insertions.isEmpty)
    }

    @Test func simpleInsertion() {
        let changes = changes(from: "tex", to: "text")
        #expect(changes.insertions.map(\.range) ==
            [NSRange(location: 3, length: 1)])
        #expect(changes.insertions.map(\.text) ==
            ["t"])
        #expect(changes.removals.isEmpty)
    }

    @Test func multipleInsertions() {
        let changes = changes(from: "ex", to: "texts")
        #expect(changes.insertions.map(\.range) ==
            [NSRange(location: 0, length: 1),
             NSRange(location: 3, length: 2)])
        #expect(changes.insertions.map(\.text) ==
            ["t", "ts"])
        #expect(changes.removals.isEmpty)
    }

    @Test func simpleReplacement() {
        let changes = changes(from: "text", to: "tessst")
        #expect(changes.removals ==
            [NSRange(location: 2, length: 1)])
        #expect(changes.insertions.map(\.range) ==
            [NSRange(location: 2, length: 3)])
        #expect(changes.insertions.map(\.text) ==
            ["sss"])
    }

    @Test func multipleReplacements() {
        let changes = changes(from: "text", to: "wexpf")
        #expect(changes.removals ==
            [NSRange(location: 0, length: 1),
             NSRange(location: 3, length: 1)])
        #expect(changes.insertions.map(\.range) ==
            [NSRange(location: 0, length: 1),
             NSRange(location: 3, length: 2)])
        #expect(changes.insertions.map(\.text) ==
            ["w", "pf"])
    }

    @Test func multipleCodeUnitsReplacements() {
        let changes1 = changes(from: "abcde 🥳", to: "abcde")
        #expect(changes1.removals ==
            [NSRange(location: 5, length: 3)])
        let changes2 = changes(from: "abcde", to: "abcde 🥳")
        #expect(changes2.insertions.map(\.range) ==
            [NSRange(location: 5, length: 3)])
        #expect(changes2.insertions.map(\.text) ==
            [" 🥳"])
    }

    @Test func removalNearMultiCodeUnitsCharacters() {
        let changes = changes(from: "abcde 🥳 ", to: "abcde 🥳")
        #expect(changes.removals ==
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
