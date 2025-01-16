//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

@testable import WysiwygComposer
import XCTest

extension WysiwygComposerViewModelTests {
    func testSetAtRooMentionsState() {
        viewModel.setAtRoomMention()
        XCTAssertEqual(viewModel.getMentionsState(), MentionsState(userIds: [], roomIds: [], roomAliases: [], hasAtRoomMention: true))
    }
    
    func testAtRooMentionsStateBySettingContent() {
        viewModel.setHtmlContent("@room")
        XCTAssertEqual(viewModel.getMentionsState(), MentionsState(userIds: [], roomIds: [], roomAliases: [], hasAtRoomMention: true))
    }
    
    func testMentionsStatBySettingUserMention() {
        viewModel.setMention(url: "https://matrix.to/#/@alice:matrix.org", name: "Alice", mentionType: .user)
        XCTAssertEqual(viewModel.getMentionsState(),
                       MentionsState(userIds: ["@alice:matrix.org"], roomIds: [], roomAliases: [], hasAtRoomMention: false))
    }
    
    func testMentionsStateBySettingUserMentionFromContent() {
        let result = MentionsState(userIds: ["@alice:matrix.org"], roomIds: [], roomAliases: [], hasAtRoomMention: false)
        viewModel.setHtmlContent("<a href=\"https://matrix.to/#/@alice:matrix.org\">Alice</a>")
        XCTAssertEqual(viewModel.getMentionsState(), result)
        
        viewModel.setMarkdownContent("[Alice](https://matrix.to/#/@alice:matrix.org)")
        XCTAssertEqual(viewModel.getMentionsState(), result)
    }
    
    func testMentionsStatBySettingRoomAliasMention() {
        viewModel.setMention(url: "https://matrix.to/#/#room:matrix.org", name: "Room", mentionType: .room)
        XCTAssertEqual(viewModel.getMentionsState(),
                       MentionsState(userIds: [], roomIds: [], roomAliases: ["#room:matrix.org"], hasAtRoomMention: false))
    }
    
    func testMentionsStateBySettingRoomAliasMentionFromContent() {
        let result = MentionsState(userIds: [], roomIds: [], roomAliases: ["#room:matrix.org"], hasAtRoomMention: false)
        viewModel.setHtmlContent("<a href=\"https://matrix.to/#/#room:matrix.org\">Room</a>")
        XCTAssertEqual(viewModel.getMentionsState(), result)
        
        viewModel.setMarkdownContent("[Room](https://matrix.to/#/#room:matrix.org)")
        XCTAssertEqual(viewModel.getMentionsState(), result)
    }
        
    func testMentionsStatBySettingRoomIDMention() {
        viewModel.setMention(url: "https://matrix.to/#/!room:matrix.org", name: "Room", mentionType: .room)
        XCTAssertEqual(viewModel.getMentionsState(),
                       MentionsState(userIds: [],
                                     roomIds: ["!room:matrix.org"],
                                     roomAliases: [],
                                     hasAtRoomMention: false))
    }
    
    func testMentionsStateBySettingRoomIDMentionFromContent() {
        let result = MentionsState(userIds: [], roomIds: ["!room:matrix.org"], roomAliases: [], hasAtRoomMention: false)
        viewModel.setHtmlContent("<a href=\"https://matrix.to/#/!room:matrix.org\">Room</a>")
        XCTAssertEqual(viewModel.getMentionsState(), result)
        
        viewModel.setMarkdownContent("[Room](https://matrix.to/#/!room:matrix.org)")
        XCTAssertEqual(viewModel.getMentionsState(), result)
    }
    
    func testMultipleMentionsBySettingThemIndividually() {
        viewModel.setMention(url: "https://matrix.to/#/@alice:matrix.org", name: "Alice", mentionType: .user)
        viewModel.setMention(url: "https://matrix.to/#/@bob:matrix.org", name: "Bob", mentionType: .user)
        viewModel.setAtRoomMention()
        
        let mentionsState = viewModel.getMentionsState()
        XCTAssertEqual(mentionsState.userIds.count, 2)
        XCTAssertEqual(Set(mentionsState.userIds), ["@alice:matrix.org", "@bob:matrix.org"])
        XCTAssertTrue(mentionsState.hasAtRoomMention)
        XCTAssertTrue(mentionsState.roomIds.isEmpty)
        XCTAssertTrue(mentionsState.roomAliases.isEmpty)
    }
    
    func testMultipleDuplicateMentionsBySettingThemIndividually() {
        viewModel.setMention(url: "https://matrix.to/#/@alice:matrix.org", name: "Alice", mentionType: .user)
        viewModel.setMention(url: "https://matrix.to/#/@alice:matrix.org", name: "Alice", mentionType: .user)
        
        XCTAssertEqual(viewModel.getMentionsState(),
                       MentionsState(userIds: ["@alice:matrix.org"], roomIds: [], roomAliases: [], hasAtRoomMention: false))
    }
    
    func testMultipleMentionsBySettingThemWithHtmlContent() {
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
        XCTAssertEqual(Set(mentionState.userIds), ["@alice:matrix.org", "@bob:matrix.org"])
        XCTAssertEqual(mentionState.roomAliases, ["#room:matrix.org"])
        XCTAssertEqual(mentionState.roomIds, ["!room:matrix.org"])
        XCTAssertTrue(mentionState.hasAtRoomMention)
    }
    
    func testMultipleMentionsBySettingThemWithMarkdownContent() {
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
        XCTAssertEqual(Set(mentionState.userIds), ["@alice:matrix.org", "@bob:matrix.org"])
        XCTAssertEqual(mentionState.roomAliases, ["#room:matrix.org"])
        XCTAssertEqual(mentionState.roomIds, ["!room:matrix.org"])
        XCTAssertTrue(mentionState.hasAtRoomMention)
    }
}
