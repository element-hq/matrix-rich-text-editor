//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

@testable import HTMLParser
import Testing
import UIKit

extension HTMLParserTests {
    @Test func replaceLinks() throws {
        let html = "<a data-mention-type=\"user\" href=\"https://matrix.to/#/@alice:matrix.org\">Alice</a>:\(String.nbsp)"
        let attributed = try HTMLParser.parse(html: html, mentionReplacer: CustomHTMLMentionReplacer())
        // A text attachment is added.
        #expect(attributed.attribute(.attachment, at: 0, effectiveRange: nil) is NSTextAttachment)
        // The original content is added to the new part of the attributed string.
        let originalContent = attributed.attribute(.mention, at: 0, effectiveRange: nil) as? MentionContent
        #expect(originalContent?.rustLength == 1)
        // HTML and attributed range matches
        let htmlRange = NSRange(location: 0, length: 1)
        let attributedRange = NSRange(location: 0, length: 1)
        #expect(try attributed.attributedRange(from: htmlRange) == attributedRange)
        #expect(try attributed.htmlRange(from: attributedRange) == htmlRange)
        // HTML chars match content.
        #expect(attributed.htmlChars == "\(String.object):\(String.nbsp)")
    }

    @Test func mentionsAreNotReplaced() throws {
        let html = "<a data-mention-type=\"user\" href=\"https://matrix.to/#/@alice:matrix.org\">Alice</a>:\(String.nbsp)"
        let attributed = try HTMLParser.parse(html: html, mentionReplacer: nil)
        // No text attachment.
        #expect(!(attributed.attribute(.attachment, at: 0, effectiveRange: nil) is NSTextAttachment))
        // The original content is still added to the new part of the attributed string.
        let originalContent = attributed.attribute(.mention, at: 0, effectiveRange: nil) as? MentionContent
        #expect(originalContent?.rustLength == 1)
        // HTML and attributed range matches
        let htmlRange = NSRange(location: 0, length: 1)
        let attributedRange = NSRange(location: 0, length: 5)
        #expect(try attributed.attributedRange(from: htmlRange) == attributedRange)
        #expect(try attributed.htmlRange(from: attributedRange) == htmlRange)

        // Positions in the middle of the mention should translate to the end of it
        #expect(try attributed.htmlPosition(at: 1) == 1)
        #expect(try attributed.htmlPosition(at: 2) == 1)
        #expect(try attributed.htmlPosition(at: 3) == 1)
        #expect(try attributed.htmlPosition(at: 4) == 1)

        // HTML chars match content.
        #expect(attributed.htmlChars == "\(String.object):\(String.nbsp)")
    }

    @Test func replaceMultipleLinks() throws {
        let html = """
        <a data-mention-type=\"user\" href=\"https://matrix.to/#/@alice:matrix.org\">Alice</a> \
        <a data-mention-type=\"user\" href=\"https://matrix.to/#/@alice:matrix.org\">Alice</a>\(String.nbsp)
        """
        let attributed = try HTMLParser.parse(html: html, mentionReplacer: CustomHTMLMentionReplacer())
        // HTML position matches exactly (Rust model mention length is 1, and so is the length of a pill).
        #expect(try attributed.htmlPosition(at: 0) == 0)
        #expect(try attributed.htmlPosition(at: 1) == 1)
        #expect(try attributed.htmlPosition(at: 2) == 2)
        #expect(try attributed.htmlPosition(at: 3) == 3)
        #expect(try attributed.htmlPosition(at: 4) == 4)
        // Out of bound attributed position throws
        do {
            _ = try attributed.htmlPosition(at: 5)
            Issue.record("HTML position call should have thrown")
        } catch {
            #expect(error as? AttributedRangeError == AttributedRangeError.outOfBoundsAttributedIndex(index: 5))
        }

        // Attributed position matches exactly (Rust model mention length is 1, and so is the length of a pill).
        #expect(try attributed.attributedPosition(at: 0) == 0)
        #expect(try attributed.attributedPosition(at: 1) == 1)
        #expect(try attributed.attributedPosition(at: 2) == 2)
        #expect(try attributed.attributedPosition(at: 3) == 3)
        #expect(try attributed.attributedPosition(at: 4) == 4)

        let firstLinkHtmlRange = NSRange(location: 0, length: 1)
        let firstLinkAttributedRange = NSRange(location: 0, length: 1)
        #expect(try attributed.attributedRange(from: firstLinkHtmlRange) == firstLinkAttributedRange)
        #expect(try attributed.htmlRange(from: firstLinkAttributedRange) == firstLinkHtmlRange)

        let secondLinkHtmlRange = NSRange(location: 2, length: 1)
        let secondLinkAttributedRange = NSRange(location: 2, length: 1)
        #expect(try attributed.attributedRange(from: secondLinkHtmlRange) == secondLinkAttributedRange)
        #expect(try attributed.htmlRange(from: secondLinkAttributedRange) == secondLinkHtmlRange)
        // HTML chars match content.
        #expect(attributed.htmlChars == "\(String.object) \(String.object)\(String.nbsp)")
    }

    @Test func multipleMentionsAreNotReplaced() throws {
        let html = """
        <a data-mention-type=\"user\" href=\"https://matrix.to/#/@alice:matrix.org\">Alice</a> \
        <a data-mention-type=\"user\" href=\"https://matrix.to/#/@alice:matrix.org\">Alice</a>\(String.nbsp)
        """
        let attributed = try HTMLParser.parse(html: html, mentionReplacer: nil)
        // HTML position matches.
        #expect(try attributed.htmlPosition(at: 0) == 0)
        #expect(try attributed.htmlPosition(at: 5) == 1)
        #expect(try attributed.htmlPosition(at: 6) == 2)
        #expect(try attributed.htmlPosition(at: 11) == 3)
        #expect(try attributed.htmlPosition(at: 12) == 4)
        // Out of bound attributed position throws
        do {
            _ = try attributed.htmlPosition(at: 13)
            Issue.record("HTML position call should have thrown")
        } catch {
            #expect(error as? AttributedRangeError == AttributedRangeError.outOfBoundsAttributedIndex(index: 13))
        }

        // Attributed position matches.
        #expect(try attributed.attributedPosition(at: 0) == 0)
        #expect(try attributed.attributedPosition(at: 1) == 5)
        #expect(try attributed.attributedPosition(at: 2) == 6)
        #expect(try attributed.attributedPosition(at: 3) == 11)
        #expect(try attributed.attributedPosition(at: 4) == 12)

        let firstLinkHtmlRange = NSRange(location: 0, length: 1)
        let firstLinkAttributedRange = NSRange(location: 0, length: 5)
        #expect(try attributed.attributedRange(from: firstLinkHtmlRange) == firstLinkAttributedRange)
        #expect(try attributed.htmlRange(from: firstLinkAttributedRange) == firstLinkHtmlRange)

        let secondLinkHtmlRange = NSRange(location: 2, length: 1)
        let secondLinkAttributedRange = NSRange(location: 6, length: 5)
        #expect(try attributed.attributedRange(from: secondLinkHtmlRange) == secondLinkAttributedRange)
        #expect(try attributed.htmlRange(from: secondLinkAttributedRange) == secondLinkHtmlRange)
        // HTML chars match content.
        #expect(attributed.htmlChars == "\(String.object) \(String.object)\(String.nbsp)")
    }
}

private class CustomHTMLMentionReplacer: HTMLMentionReplacer {
    func replacementForMention(_ url: String, text: String) -> NSAttributedString? {
        if url.starts(with: "https://matrix.to/#/"),
           let image = UIImage(systemName: "link") {
            // Set a text attachment with an arbitrary image.
            return NSAttributedString(attachment: NSTextAttachment(image: image))
        } else {
            return nil
        }
    }
}
