//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

@testable import WysiwygComposer
import XCTest

extension WysiwygComposerTests {
    func testSuggestionForAtPattern() {
        let model = ComposerModelWrapper()
        let update = model.replaceText(newText: "@alic")

        guard case .suggestion(suggestionPattern: let suggestionPattern) = update.menuAction(),
              let attributes = suggestionPattern.key.mentionType?.attributes
        else {
            XCTFail("No user suggestion found")
            return
        }

        model
            .action {
                $0.insertMentionAtSuggestion(
                    url: "https://matrix.to/#/@alice:matrix.org",
                    text: "Alice",
                    suggestion: suggestionPattern,
                    attributes: attributes
                )
            }
            .assertHtml(
                """
                <a data-mention-type="user" href="https://matrix.to/#/@alice:matrix.org" contenteditable="false">Alice</a>\(String.nbsp)
                """
            )
            .assertSelection(start: 2, end: 2)
    }

    func testNonLeadingSuggestionForAtPattern() {
        let model = ComposerModelWrapper()
        let update = model.replaceText(newText: "Hello @alic")

        guard case .suggestion(suggestionPattern: let suggestionPattern) = update.menuAction(),
              let attributes = suggestionPattern.key.mentionType?.attributes
        else {
            XCTFail("No user suggestion found")
            return
        }

        model
            .action {
                $0.insertMentionAtSuggestion(
                    url: "https://matrix.to/#/@alice:matrix.org",
                    text: "Alice",
                    suggestion: suggestionPattern,
                    attributes: attributes
                )
            }
            .assertHtml(
                """
                Hello <a data-mention-type="user" \
                href="https://matrix.to/#/@alice:matrix.org" \
                contenteditable="false">Alice</a>\(String.nbsp)
                """
            )
            .assertSelection(start: 8, end: 8)
    }
    
    func testSuggestionForAtRoomPattern() {
        let model = ComposerModelWrapper()
        let update = model.replaceText(newText: "@roo")

        guard case .suggestion(suggestionPattern: let suggestionPattern) = update.menuAction() else {
            XCTFail("No user suggestion found")
            return
        }

        model
            .action {
                $0.insertAtRoomMentionAtSuggestion(suggestionPattern)
            }
            .assertHtml("<a data-mention-type=\"at-room\" href=\"#\" contenteditable=\"false\">@room</a> ")
            .assertSelection(start: 2, end: 2)
    }
    
    func testForNonLeadingSuggestionForAtRoomPattern() {
        let model = ComposerModelWrapper()
        let update = model.replaceText(newText: "Hello @roo")

        guard case .suggestion(suggestionPattern: let suggestionPattern) = update.menuAction() else {
            XCTFail("No user suggestion found")
            return
        }

        model
            .action {
                $0.insertAtRoomMentionAtSuggestion(suggestionPattern)
            }
            .assertHtml("Hello <a data-mention-type=\"at-room\" href=\"#\" contenteditable=\"false\">@room</a> ")
            .assertSelection(start: 8, end: 8)
    }

    func testSuggestionForHashPattern() {
        let model = ComposerModelWrapper()
        let update = model.replaceText(newText: "#roo")

        guard case .suggestion(suggestionPattern: let suggestionPattern) = update.menuAction(),
              let attributes = suggestionPattern.key.mentionType?.attributes
        else {
            XCTFail("No room suggestion found")
            return
        }

        model
            .action {
                $0.insertMentionAtSuggestion(
                    url: "https://matrix.to/#/#room1:matrix.org",
                    text: "Room 1",
                    suggestion: suggestionPattern,
                    attributes: attributes
                )
            }
            .assertHtml(
                """
                <a data-mention-type="room" href="https://matrix.to/#/#room1:matrix.org" contenteditable="false">Room 1</a>\(String.nbsp)
                """
            )
    }

    func testSuggestionForSlashPattern() {
        let model = ComposerModelWrapper()
        let update = model.replaceText(newText: "/")

        guard case .suggestion(suggestionPattern: let suggestionPattern) = update.menuAction() else {
            XCTFail("No suggestion found")
            return
        }

        model
            .action {
                $0.replaceTextSuggestion(newText: "/invite", suggestion: suggestionPattern)
            }
            .assertHtml("/invite\(String.nbsp)")
    }
}
