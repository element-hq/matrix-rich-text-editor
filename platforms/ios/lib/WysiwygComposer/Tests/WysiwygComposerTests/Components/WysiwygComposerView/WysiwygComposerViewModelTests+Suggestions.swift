//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import Combine
import Testing
@testable import WysiwygComposer

extension WysiwygComposerViewModelTests {
    @Test func atSuggestionsArePublished() async {
        let publisher = viewModel.$suggestionPattern.removeDuplicates().dropFirst()
        let pattern = await nextValue(of: publisher) {
            _ = viewModel.replaceText(range: .zero, replacementText: "@ali")
        }
        #expect(pattern == SuggestionPattern(key: .at, text: "ali", start: 0, end: 4))

        let pattern2 = await nextValue(of: publisher) {
            _ = viewModel.replaceText(range: .init(location: 4, length: 0), replacementText: "ce")
        }
        #expect(pattern2 == SuggestionPattern(key: .at, text: "alice", start: 0, end: 6))
    }

    @Test func hashSuggestionsArePublished() async {
        let publisher = viewModel.$suggestionPattern.removeDuplicates().dropFirst()
        let pattern = await nextValue(of: publisher) {
            _ = viewModel.replaceText(range: .zero, replacementText: "#room")
        }
        #expect(pattern == SuggestionPattern(key: .hash, text: "room", start: 0, end: 5))
    }

    @Test func slashSuggestionArePublished() async {
        let publisher = viewModel.$suggestionPattern.removeDuplicates().dropFirst()
        let pattern = await nextValue(of: publisher) {
            _ = viewModel.replaceText(range: .zero, replacementText: "/inv")
        }
        #expect(pattern == SuggestionPattern(key: .slash, text: "inv", start: 0, end: 4))
    }

    @Test func atSuggestionCanBeUsed() {
        _ = viewModel.replaceText(range: .zero, replacementText: "@ali")
        viewModel.setMention(url: "https://matrix.to/#/@alice:matrix.org", name: "Alice", mentionType: .user)
        #expect(viewModel.content.html == """
        <a href="https://matrix.to/#/@alice:matrix.org">Alice</a>\u{00A0}
        """)
    }

    @Test func atRoomSuggestionCanBeUsed() {
        _ = viewModel.replaceText(range: .zero, replacementText: "@ro")
        viewModel.setAtRoomMention()
        #expect(viewModel.content.html == """
        @room\u{00A0}
        """)
    }

    @Test func atMentionWithNoSuggestion() {
        _ = viewModel.replaceText(range: .zero, replacementText: "Text")
        viewModel.select(range: .init(location: 0, length: 4))
        viewModel.setMention(url: "https://matrix.to/#/@alice:matrix.org", name: "Alice", mentionType: .user)
        // Text is not removed, and the
        // mention is added after the text
        #expect(viewModel.content.html == """
        Text<a href="https://matrix.to/#/@alice:matrix.org">Alice</a>\u{00A0}
        """)
    }

    @Test func atRoomMentionWithNoSuggestion() {
        _ = viewModel.replaceText(range: .zero, replacementText: "Text")
        viewModel.select(range: .init(location: 0, length: 4))
        viewModel.setAtRoomMention()
        // Text is not removed, and the
        // mention is added after the text
        #expect(viewModel.content.html == """
        Text@room\u{00A0}
        """)
    }

    @Test func atMentionWithNoSuggestionAtLeading() {
        _ = viewModel.replaceText(range: .zero, replacementText: "Text")
        viewModel.select(range: .init(location: 0, length: 0))
        viewModel.setMention(url: "https://matrix.to/#/@alice:matrix.org", name: "Alice", mentionType: .user)
        // Text is not removed, and the mention is added before the text
        #expect(viewModel.content.html == """
        <a href="https://matrix.to/#/@alice:matrix.org">Alice</a>Text
        """)
    }

    @Test func atRoomMentionWithNoSuggestionAtLeading() {
        _ = viewModel.replaceText(range: .zero, replacementText: "Text")
        viewModel.select(range: .init(location: 0, length: 0))
        viewModel.setAtRoomMention()
        // Text is not removed, and the mention is added before the text
        #expect(viewModel.content.html == """
        @roomText
        """)
    }

    @Test func hashSuggestionCanBeUsed() {
        _ = viewModel.replaceText(range: .zero, replacementText: "#roo")
        viewModel.setMention(url: "https://matrix.to/#/#room1:matrix.org", name: "Room 1", mentionType: .room)
        #expect(viewModel.content.html == """
        <a href="https://matrix.to/#/#room1:matrix.org">#room1:matrix.org</a>\u{00A0}
        """)
    }

    @Test func hashMentionWithNoSuggestion() {
        _ = viewModel.replaceText(range: .zero, replacementText: "Text")
        viewModel.select(range: .init(location: 0, length: 4))
        viewModel.setMention(url: "https://matrix.to/#/#room1:matrix.org", name: "Room 1", mentionType: .room)
        #expect(viewModel.content.html == """
        Text<a href="https://matrix.to/#/#room1:matrix.org">#room1:matrix.org</a>\u{00A0}
        """)
    }

    @Test func hashMentionWithNoSuggestionAtLeading() {
        _ = viewModel.replaceText(range: .zero, replacementText: "Text")
        viewModel.select(range: .init(location: 0, length: 0))
        viewModel.setMention(url: "https://matrix.to/#/#room1:matrix.org", name: "Room 1", mentionType: .room)
        #expect(viewModel.content.html == """
        <a href="https://matrix.to/#/#room1:matrix.org">#room1:matrix.org</a>Text
        """)
    }

    @Test func slashSuggestionCanBeUsed() {
        _ = viewModel.replaceText(range: .zero, replacementText: "/inv")
        viewModel.setCommand(name: "/invite")
        #expect(viewModel.content.html == """
        /invite\u{00A0}
        """)
    }
}
