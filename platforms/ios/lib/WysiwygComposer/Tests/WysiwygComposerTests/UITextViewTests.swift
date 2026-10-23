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
    @Test func textViewUTF16Encoding() {
        let textView = UITextView()
        textView.attributedText = NSAttributedString(string: TestConstants.testStringWithEmojis)
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

    // MARK: - List markers and typing attributes

    @Test func listMarkerIndentsTypingInItsParagraphOnly() {
        let textView = WysiwygTextView()
        textView.attributedText = NSAttributedString(string: "item\npara")
        let marker = listMarker(at: 0)

        textView.selectedRange = NSRange(location: 4, length: 0)
        textView.updateListMarkers([marker])
        #expect(typingIndent(of: textView) == marker.headIndent)

        textView.selectedRange = NSRange(location: 7, length: 0)
        textView.updateListMarkers([marker])
        #expect(typingIndent(of: textView) == 0, "A marker only indents the paragraph it starts")
    }

    @Test func listMarkerIndentsEmptyTrailingItem() {
        let textView = WysiwygTextView()
        textView.attributedText = NSAttributedString(string: "item\n")
        textView.selectedRange = NSRange(location: 5, length: 0)
        textView.updateListMarkers([listMarker(at: 0), listMarker(at: 5)])
        #expect(typingIndent(of: textView) == 26)
    }

    @Test func updateListMarkersKeepsParagraphStyleSetElsewhere() {
        let textView = WysiwygTextView()
        textView.attributedText = NSAttributedString(string: "quote\n")
        textView.selectedRange = NSRange(location: 6, length: 0)
        // `apply` installs the quote style for a caret on an empty quote line.
        let quoteStyle = HTMLParserStyle.standard.quoteBlockStyle.paragraphStyle
        textView.typingAttributes[.paragraphStyle] = quoteStyle
        textView.updateListMarkers([])
        #expect(typingIndent(of: textView) == quoteStyle.firstLineHeadIndent)
    }

    @Test func updateListMarkersOnlyRemovesItsOwnParagraphStyle() {
        let textView = WysiwygTextView()
        textView.attributedText = NSAttributedString(string: "item")
        textView.selectedRange = NSRange(location: 4, length: 0)
        textView.updateListMarkers([listMarker(at: 0)])
        #expect(typingIndent(of: textView) == 26)

        // Leaving the list: the style installed above goes away.
        textView.updateListMarkers([])
        #expect(typingIndent(of: textView) == 0)

        // Leaving the list into a quote: the quote style `apply` set for the caret survives.
        textView.updateListMarkers([listMarker(at: 0)])
        let quoteStyle = HTMLParserStyle.standard.quoteBlockStyle.paragraphStyle
        textView.typingAttributes[.paragraphStyle] = quoteStyle
        textView.updateListMarkers([])
        #expect(typingIndent(of: textView) == quoteStyle.firstLineHeadIndent)
    }

    private func listMarker(at index: Int) -> ListMarkerInfo {
        ListMarkerInfo(text: "\u{2022}",
                       font: .preferredFont(forTextStyle: .body),
                       color: .label,
                       characterIndex: index,
                       headIndent: 26)
    }

    private func typingIndent(of textView: UITextView) -> CGFloat {
        (textView.typingAttributes[.paragraphStyle] as? NSParagraphStyle)?.firstLineHeadIndent ?? 0
    }
}
