//
// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

@testable import HTMLParser
import XCTest

final class HTMLParserTests: XCTestCase {
    func testBuildAttributedFromHtml() throws {
        let html = "<p>Some <strong>bold and <em>italic</em> text</strong></p>"
        let attributed = try HTMLParser.parse(html: html)
        XCTAssertEqual(attributed.string,
                       "Some bold and italic text")
        // Font at index 6 is bold
        let fontTraits1 = attributed.fontSymbolicTraits(at: 6)
        XCTAssert(fontTraits1.contains(.traitBold))
        XCTAssert(!fontTraits1.contains(.traitItalic))
        // Font at index 15 is bold and italic
        let fontTraits2 = attributed.fontSymbolicTraits(at: 15)
        XCTAssert(fontTraits2.isSuperset(of: [.traitBold, .traitItalic]))
        // Font at index 2 is neither italic, nor bold
        let fontTraits3 = attributed.fontSymbolicTraits(at: 2)
        XCTAssert(fontTraits3.isDisjoint(with: [.traitBold, .traitItalic]))
    }

    func testInvalidEncodingString() throws {
        let invalidString = "\u{F023}"
        let encoding = String.Encoding.ascii
        do {
            _ = try HTMLParser.parse(html: invalidString, encoding: encoding)
        } catch {
            XCTAssertEqual(error as? BuildHtmlAttributedError, BuildHtmlAttributedError.dataError(encoding: encoding))
            XCTAssertEqual(error.localizedDescription,
                           "Unable to encode string with: \(encoding.description) rawValue: \(encoding.rawValue)")
        }
    }

    func testCodeBlockBackgroundStyleIsApplied() throws {
        let html = "<pre><code>code block</code></pre>"
        let attributed = try HTMLParser.parse(html: html)
        XCTAssertEqual(attributed.attribute(.blockStyle, at: 0, effectiveRange: nil) as? BlockStyle,
                       HTMLParserStyle.standard.codeBlockStyle)
    }

    func testQuoteBackgroundStyleIsApplied() throws {
        let html = "<blockquote>quote</blockquote>some text"
        let attributed = try HTMLParser.parse(html: html)
        XCTAssertEqual(attributed.attribute(.blockStyle, at: 0, effectiveRange: nil) as? BlockStyle,
                       HTMLParserStyle.standard.quoteBlockStyle)
    }

    func testInlineCodeBackgroundColorIsApplied() throws {
        let html = "<code>inline code</code>"
        let attributed = try HTMLParser.parse(html: html)
        XCTAssertEqual(attributed.backgroundColor(at: 0),
                       HTMLParserStyle.standard.codeBlockStyle.backgroundColor)
    }

    func testSingleNbspInParagraphIsReplacedByZwsp() throws {
        let html = "<p>\(String.nbsp)</p>"
        let attributed = try HTMLParser.parse(html: html)
        XCTAssertEqual(attributed.string, "\(String.zwsp)")
    }

    func testSingleNbspInCodeBlockIsReplacedByZwsp() throws {
        let html = "<pre><code>\(String.nbsp)</code></pre>"
        let attributed = try HTMLParser.parse(html: html)
        XCTAssertEqual(attributed.string, "\(String.zwsp)")
    }

    func testTrailingCodeBlockNbspWithoutLineFeedIsReplacedByZwsp() throws {
        // Note shouldn't happen in normal circumstances.
        // But this case should definitely not add a lineSeparator
        let html = "<pre><code>Test\(String.nbsp)</code></pre>"
        let attributed = try HTMLParser.parse(html: html)
        XCTAssertEqual(attributed.string, "Test\(String.zwsp)")
    }

    func testTrailingCodeBlockNbspWithLineFeedIsReplacedByLineSeparatorAndZwsp() throws {
        let html = "<pre><code>Test\n\(String.nbsp)</code></pre>"
        let attributed = try HTMLParser.parse(html: html)
        XCTAssertEqual(attributed.string, "Test\(String.lineSeparator)\(String.zwsp)")
    }

    func testLeadingCodeBlockNbspIsReplacedByZwsp() throws {
        let html = "<pre><code>\(String.nbsp)Test</code></pre>"
        let attributed = try HTMLParser.parse(html: html)
        XCTAssertEqual(attributed.string, "\(String.zwsp)Test")
    }
}
