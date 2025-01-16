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
    func testSetBaseStringWithEmoji() {
        ComposerModelWrapper()
            .action { $0.replaceText(newText: TestConstants.testStringWithEmojis) }
            // Text is preserved, including emojis.
            .assertHtml(TestConstants.testStringWithEmojis)
            // Selection is set at the end of the text.
            .assertSelection(start: 14, end: 14)
    }

    func testBackspacingEmoji() {
        ComposerModelWrapper()
            .action { $0.replaceText(newText: TestConstants.testStringWithEmojis) }
            .action { $0.select(startUtf16Codeunit: 7, endUtf16Codeunit: 14) }
            .action { $0.backspace() }
            // Text should remove exactly the last emoji.
            .assertHtml(TestConstants.testStringAfterBackspace)
            .assertSelection(start: 7, end: 7)
    }
}
