//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import Testing
@testable import WysiwygComposer

extension WysiwygComposerViewModelTests {
    @Test func setAtRooMentionsState() {
        viewModel.setAtRoomMention()
        #expect(viewModel.getMentionsState() == MentionsState(userIds: [], roomIds: [], roomAliases: [], hasAtRoomMention: true))
    }

    @Test func atRooMentionsStateBySettingContent() {
        viewModel.setHtmlContent("@room")
        #expect(viewModel.getMentionsState() == MentionsState(userIds: [], roomIds: [], roomAliases: [], hasAtRoomMention: true))
    }

    @Test func mentionsStatBySettingUserMention() {
        viewModel.setMention(url: "https://matrix.to/#/@alice:matrix.org", name: "Alice", mentionType: .user)
        #expect(viewModel.getMentionsState()
            == MentionsState(userIds: ["@alice:matrix.org"], roomIds: [], roomAliases: [], hasAtRoomMention: false))
    }

    @Test func mentionsStateBySettingUserMentionFromContent() {
        let result = MentionsState(userIds: ["@alice:matrix.org"], roomIds: [], roomAliases: [], hasAtRoomMention: false)
        viewModel.setHtmlContent("<a href=\"https://matrix.to/#/@alice:matrix.org\">Alice</a>")
        #expect(viewModel.getMentionsState() == result)

        viewModel.setMarkdownContent("[Alice](https://matrix.to/#/@alice:matrix.org)")
        #expect(viewModel.getMentionsState() == result)
    }

    @Test func mentionsStatBySettingRoomAliasMention() {
        viewModel.setMention(url: "https://matrix.to/#/#room:matrix.org", name: "Room", mentionType: .room)
        #expect(viewModel.getMentionsState()
            == MentionsState(userIds: [], roomIds: [], roomAliases: ["#room:matrix.org"], hasAtRoomMention: false))
    }

    @Test func mentionsStateBySettingRoomAliasMentionFromContent() {
        let result = MentionsState(userIds: [], roomIds: [], roomAliases: ["#room:matrix.org"], hasAtRoomMention: false)
        viewModel.setHtmlContent("<a href=\"https://matrix.to/#/#room:matrix.org\">Room</a>")
        #expect(viewModel.getMentionsState() == result)

        viewModel.setMarkdownContent("[Room](https://matrix.to/#/#room:matrix.org)")
        #expect(viewModel.getMentionsState() == result)
    }

    @Test func mentionsStatBySettingRoomIDMention() {
        viewModel.setMention(url: "https://matrix.to/#/!room:matrix.org", name: "Room", mentionType: .room)
        #expect(viewModel.getMentionsState()
            == MentionsState(userIds: [],
                             roomIds: ["!room:matrix.org"],
                             roomAliases: [],
                             hasAtRoomMention: false))
    }

    @Test func mentionsStateBySettingRoomIDMentionFromContent() {
        let result = MentionsState(userIds: [], roomIds: ["!room:matrix.org"], roomAliases: [], hasAtRoomMention: false)
        viewModel.setHtmlContent("<a href=\"https://matrix.to/#/!room:matrix.org\">Room</a>")
        #expect(viewModel.getMentionsState() == result)

        viewModel.setMarkdownContent("[Room](https://matrix.to/#/!room:matrix.org)")
        #expect(viewModel.getMentionsState() == result)
    }

    @Test func multipleMentionsBySettingThemIndividually() {
        viewModel.setMention(url: "https://matrix.to/#/@alice:matrix.org", name: "Alice", mentionType: .user)
        viewModel.setMention(url: "https://matrix.to/#/@bob:matrix.org", name: "Bob", mentionType: .user)
        viewModel.setAtRoomMention()

        let mentionsState = viewModel.getMentionsState()
        #expect(mentionsState.userIds.count == 2)
        #expect(Set(mentionsState.userIds) == ["@alice:matrix.org", "@bob:matrix.org"])
        #expect(mentionsState.hasAtRoomMention)
        #expect(mentionsState.roomIds.isEmpty)
        #expect(mentionsState.roomAliases.isEmpty)
    }

    @Test func multipleDuplicateMentionsBySettingThemIndividually() {
        viewModel.setMention(url: "https://matrix.to/#/@alice:matrix.org", name: "Alice", mentionType: .user)
        viewModel.setMention(url: "https://matrix.to/#/@alice:matrix.org", name: "Alice", mentionType: .user)

        #expect(viewModel.getMentionsState()
            == MentionsState(userIds: ["@alice:matrix.org"], roomIds: [], roomAliases: [], hasAtRoomMention: false))
    }

    @Test func multipleMentionsBySettingThemWithHtmlContent() {
        viewModel.setHtmlContent(
            """
            <p><a href=\"https://matrix.to/#/@alice:matrix.org\">Alice</a>, \
            <a href=\"https://matrix.to/#/!room:matrix.org\">Room</a>, \
            <a href=\"https://matrix.to/#/@bob:matrix.org\">Bob</a>, \
            <a href=\"https://matrix.to/#/#room:matrix.org\">Room</a>, \
            @room</p>
            """
        )
        let mentionState = viewModel.getMentionsState()
        #expect(Set(mentionState.userIds) == ["@alice:matrix.org", "@bob:matrix.org"])
        #expect(mentionState.roomAliases == ["#room:matrix.org"])
        #expect(mentionState.roomIds == ["!room:matrix.org"])
        #expect(mentionState.hasAtRoomMention)
    }

    @Test func multipleMentionsBySettingThemWithMarkdownContent() {
        viewModel.setMarkdownContent(
            """
            [Room](https://matrix.to/#/!room:matrix.org), \
            [Room](https://matrix.to/#/#room:matrix.org), \
            [Alice](https://matrix.to/#/@alice:matrix.org), \
            [Bob](https://matrix.to/#/@bob:matrix.org), \
            @room
            """
        )
        let mentionState = viewModel.getMentionsState()
        #expect(Set(mentionState.userIds) == ["@alice:matrix.org", "@bob:matrix.org"])
        #expect(mentionState.roomAliases == ["#room:matrix.org"])
        #expect(mentionState.roomIds == ["!room:matrix.org"])
        #expect(mentionState.hasAtRoomMention)
    }
}
