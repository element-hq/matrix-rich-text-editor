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

@MainActor
struct NSAttributedStringRangeTests {
    @Test func attributedNumberedLists() throws {
        let html = "<ol><li>Item 1</li><li>Item 2</li></ol><p>Some Text</p>"
        let attributed = try HTMLParser.parse(html: html)

        // A textual representation of the numbered list is displayed
        #expect(attributed.string == "\t1.\tItem 1\n\t2.\tItem 2\nSome Text")

        // Ranges that are not part of the raw HTML text (excluding tags) are detected
        #expect(attributed.discardableTextRanges() == [NSRange(location: 0, length: 4),
                                                       NSRange(location: 11, length: 4)])

        // Converting back and forth from HTML to attributed postions
        #expect(try attributed.htmlPosition(at: 4) == 0)
        #expect(try attributed.attributedPosition(at: 1) == 5)
        #expect(try attributed.htmlPosition(at: 10) == 6)
        #expect(try attributed.attributedPosition(at: 7) == 15)
        #expect(try attributed.htmlPosition(at: 15) == 7)
        #expect(try attributed.attributedPosition(at: 8) == 16)

        // Attributed index inside a prefix should return a valid index in the HTML raw text
        #expect(attributed.character(at: 11) == "\t")
        #expect(try attributed.htmlPosition(at: 11) == 7)
        #expect(attributed.character(at: 12) == "2")
        #expect(try attributed.htmlPosition(at: 12) == 7)

        // Converting back and forth from HTML to attributed ranges
        // Both expected range for "Item 1"
        let htmlRange = NSRange(location: 0, length: 6)
        let attributedRange = NSRange(location: 4, length: 6)
        #expect(try attributed.attributedRange(from: htmlRange) == attributedRange)
        #expect(try attributed.htmlRange(from: attributedRange) == htmlRange)
        #expect(attributed.attributedSubstring(from: attributedRange).string == "Item 1")

        // Cross list items range
        let crossHtmlRange = NSRange(location: 0, length: 8)
        let crossAttributedRange = NSRange(location: 4, length: 12)
        #expect(try attributed.attributedRange(from: crossHtmlRange) == crossAttributedRange)
        #expect(try attributed.htmlRange(from: crossAttributedRange) == crossHtmlRange)
        #expect(attributed.attributedSubstring(from: crossAttributedRange).string == "Item 1\n\t2.\tI")
        assertHtmlCharsLengthMatchLastPosition(in: attributed)
    }

    @Test func attributedBulletedLists() throws {
        let html = "<ul><li>Item 1</li><li>Item 2</li></ul><p>Some Text</p>"
        let attributed = try HTMLParser.parse(html: html)
        #expect(attributed.string == "\t•\tItem 1\n\t•\tItem 2\nSome Text")
        #expect(attributed.discardableTextRanges() == [NSRange(location: 0, length: 3),
                                                       NSRange(location: 10, length: 3)])
        #expect(try attributed.attributedPosition(at: 1) == 4)
        #expect(try attributed.attributedPosition(at: 8) == 14)
        #expect(try attributed.htmlPosition(at: 13) == 7)
        #expect(try attributed.htmlPosition(at: 3) == 0)
        assertHtmlCharsLengthMatchLastPosition(in: attributed)
    }

    @Test func multipleAttributedLists() throws {
        let html = "<ol><li>Item 1</li><li>Item 2</li></ol><ul><li>Item 1</li><li>Item 2</li></ul>"
        let attributed = try HTMLParser.parse(html: html)
        #expect(attributed.string == "\t1.\tItem 1\n\t2.\tItem 2\n\t•\tItem 1\n\t•\tItem 2")
        #expect(attributed.discardableTextRanges() == [NSRange(location: 0, length: 4),
                                                       NSRange(location: 11, length: 4),
                                                       NSRange(location: 22, length: 3),
                                                       NSRange(location: 32, length: 3)])
        #expect(try attributed.attributedPosition(at: 14) == 25)
        #expect(try attributed.htmlPosition(at: 21) == 13)
        #expect(try attributed.attributedRange(from: .init(location: 0, length: 12)) == NSRange(location: 4, length: 16))
        #expect(try attributed.htmlRange(from: .init(location: 4, length: 17)) == NSRange(location: 0, length: 13))
        assertHtmlCharsLengthMatchLastPosition(in: attributed)
    }

    @Test func multipleDigitsNumberedLists() throws {
        // Note: DTCoreText won't display most prefixes after 19 because of DTListItemHTMLElement
        //
        // // if the non-whitespace characters are too wide then we omit the prefix
        // if ((width+5.0)>_margins.left)
        // {
        //     return nil;
        // }
        var html = "<ol>"
        for _ in 1...19 {
            html.append(contentsOf: "<li>abcd</li>")
        }
        html.append(contentsOf: "</ol>")
        let attributed = try HTMLParser.parse(html: html)
        #expect(attributed.discardableTextRanges().count == 19)
        assertHtmlCharsLengthMatchLastPosition(in: attributed)
    }

    @Test func positionAfterList() throws {
        let html = "<ol><li>test</li></ol><p>\(Character.nbsp)</p><p>\(Character.nbsp)</p>"
        let attributed = try HTMLParser.parse(html: html)
        #expect(try attributed.htmlRange(from: .init(location: 12, length: 0)) == NSRange(location: 6, length: 0))
        #expect(try attributed.attributedRange(from: .init(location: 6, length: 0)) == NSRange(location: 12, length: 0))
        assertHtmlCharsLengthMatchLastPosition(in: attributed)
    }

    @Test func positionAfterListWithInput() throws {
        let html = "<ol><li>test</li></ol><p>\(Character.nbsp)</p><p>a</p>"
        let attributed = try HTMLParser.parse(html: html)
        #expect(try attributed.htmlRange(from: .init(location: 12, length: 0)) == NSRange(location: 7, length: 0))
        #expect(try attributed.attributedRange(from: .init(location: 7, length: 0)) == NSRange(location: 12, length: 0))
        assertHtmlCharsLengthMatchLastPosition(in: attributed)
    }

    @Test func positionAfterDoubleLineBreak() throws {
        let html = "<p>Test</p><p></p><p>T</p>"
        let attributed = try HTMLParser.parse(html: html)
        #expect(try attributed.htmlRange(from: .init(location: 7, length: 0)) == NSRange(location: 6, length: 0))
        #expect(try attributed.attributedRange(from: .init(location: 6, length: 0)) == NSRange(location: 7, length: 0))
        assertHtmlCharsLengthMatchLastPosition(in: attributed)
    }

    /// Whitespace-only paragraphs must behave like empty ones: recent SDKs' libxml2 drops
    /// whitespace-only text nodes during parsing, which used to break the position mapping.
    @Test func positionAfterDoubleLineBreakWithWhitespaceParagraph() throws {
        let html = "<p>Test</p><p> </p><p>T</p>"
        let attributed = try HTMLParser.parse(html: html)
        #expect(try attributed.htmlRange(from: .init(location: 7, length: 0)) == NSRange(location: 6, length: 0))
        #expect(try attributed.attributedRange(from: .init(location: 6, length: 0)) == NSRange(location: 7, length: 0))
        assertHtmlCharsLengthMatchLastPosition(in: attributed)
    }

    @Test func outOfBoundsIndexes() throws {
        let html = "<ol><li>Item 1</li><li>Item 2</li></ol>Some Text"
        let attributed = try HTMLParser.parse(html: html)
        // Out of bounds indexes return errors
        do {
            _ = try attributed.attributedPosition(at: 40)
        } catch {
            #expect(error as? AttributedRangeError == AttributedRangeError.outOfBoundsHtmlIndex(index: 40))
            #expect(error.localizedDescription == "Provided HTML index is out of expected bounds (40)")
        }
        do {
            _ = try attributed.htmlPosition(at: 50)
        } catch {
            #expect(error as? AttributedRangeError == AttributedRangeError.outOfBoundsAttributedIndex(index: 50))
            #expect(error.localizedDescription == "Provided attributed index is out of bounds (50)")
        }
    }

    /// Assert that the last computed HTML index inside given `NSAttributedString` matches the length of `htmlChars`.
    ///
    /// - Parameter attributedString: the attributed string to test.
    private func assertHtmlCharsLengthMatchLastPosition(in attributedString: NSAttributedString) {
        let lastHtmlIndex = try? attributedString.htmlPosition(at: attributedString.length)
        #expect(attributedString.htmlChars.count == lastHtmlIndex)
    }
}
