//
// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import HTMLParser
import Testing
import UIKit
@testable import WysiwygComposer

@MainActor
struct UITextViewTests {
    @Test func textViewUTF16Encoding() throws {
        let textView = UITextView()
        textView.attributedText = try HTMLParser.parse(html: TestConstants.testStringWithEmojis)
        // Selection is at the end of the text, with a UTF-16 length of 14.
        #expect(textView.selectedRange == NSRange(location: 14, length: 0))
        // Text count what is perceived as character.
        #expect(textView.text.count == 6)
        #expect(textView.text.utf16.count == 14)
        // AttributedString counts UTF-16 directly
        #expect(textView.attributedText.length == 14)
        // Test deleting the latest emoji.
        textView.deleteBackward()
        #expect(textView.attributedText.length == 7)
        #expect(textView.text == TestConstants.testStringAfterBackspace)
    }
}
