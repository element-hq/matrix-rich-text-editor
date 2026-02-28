//
// Copyright (c) 2026 Element Creations Ltd
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import HTMLParser
@testable import WysiwygComposer
import XCTest

// MARK: - computePrefixSuffixDiff tests

final class ComputePrefixSuffixDiffTests: XCTestCase {
    func testSingleCharInsertion() {
        let diff = computePrefixSuffixDiff(old: "helo", new: "hello")
        XCTAssertEqual(diff.replaceStart, 3)
        XCTAssertEqual(diff.replaceEnd, 3)
        XCTAssertEqual(diff.replacement, "l")
    }

    func testSingleCharDeletion() {
        let diff = computePrefixSuffixDiff(old: "hello", new: "helo")
        XCTAssertEqual(diff.replaceStart, 3)
        XCTAssertEqual(diff.replaceEnd, 4)
        XCTAssertEqual(diff.replacement, "")
    }

    func testWordReplacement() {
        let diff = computePrefixSuffixDiff(old: "teh", new: "the")
        XCTAssertEqual(diff.replaceStart, 1)
        XCTAssertEqual(diff.replaceEnd, 3)
        XCTAssertEqual(diff.replacement, "he")
    }

    func testEmptyOldString() {
        let diff = computePrefixSuffixDiff(old: "", new: "abc")
        XCTAssertEqual(diff.replaceStart, 0)
        XCTAssertEqual(diff.replaceEnd, 0)
        XCTAssertEqual(diff.replacement, "abc")
    }

    func testEmptyNewString() {
        let diff = computePrefixSuffixDiff(old: "abc", new: "")
        XCTAssertEqual(diff.replaceStart, 0)
        XCTAssertEqual(diff.replaceEnd, 3)
        XCTAssertEqual(diff.replacement, "")
    }

    func testIdenticalStrings() {
        let diff = computePrefixSuffixDiff(old: "unchanged", new: "unchanged")
        XCTAssertEqual(diff.replaceStart, 9)
        XCTAssertEqual(diff.replaceEnd, 9)
        XCTAssertEqual(diff.replacement, "")
    }

    func testTrailingInsert() {
        let diff = computePrefixSuffixDiff(old: "hello", new: "hello!")
        XCTAssertEqual(diff.replaceStart, 5)
        XCTAssertEqual(diff.replaceEnd, 5)
        XCTAssertEqual(diff.replacement, "!")
    }

    func testLeadingInsert() {
        let diff = computePrefixSuffixDiff(old: "world", new: "hello world")
        XCTAssertEqual(diff.replaceStart, 0)
        XCTAssertEqual(diff.replaceEnd, 0)
        XCTAssertEqual(diff.replacement, "hello ")
    }
}

// MARK: - ProjectionRenderer tests

final class ProjectionRendererTests: XCTestCase {
    private let renderer = ProjectionRenderer(style: .standard, mentionReplacer: nil)

    private func plain(_ text: String, start: UInt32, end: UInt32, bold: Bool = false, italic: Bool = false, inlineCode: Bool = false, linkUrl: String? = nil) -> FfiBlockProjection {
        let attrs = FfiAttributeSet(bold: bold, italic: italic, strikeThrough: false,
                                    underline: false, inlineCode: inlineCode, linkUrl: linkUrl)
        let run = FfiInlineRun(nodeId: "0", startUtf16: start, endUtf16: end,
                               kind: .text(text: text, attributes: attrs))
        return FfiBlockProjection(blockId: "0", kind: .paragraph, inQuote: false, startUtf16: start, endUtf16: end, inlineRuns: [run])
    }

    func testParagraphRendersPlainText() {
        let block = plain("hello", start: 0, end: 5)
        let result = renderer.renderBlock(block)
        XCTAssertEqual(result.string, "hello")
    }

    func testBoldRunHasBoldFont() throws {
        let block = plain("bold", start: 0, end: 4, bold: true)
        let result = renderer.renderBlock(block)
        let font = result.attribute(.font, at: 0, effectiveRange: nil) as? UIFont
        XCTAssertNotNil(font)
        XCTAssertTrue(try XCTUnwrap(font?.fontDescriptor.symbolicTraits.contains(.traitBold)),
                      "Bold run should use a bold font")
    }

    func testItalicRunHasItalicFont() throws {
        let block = plain("italic", start: 0, end: 6, italic: true)
        let result = renderer.renderBlock(block)
        let font = result.attribute(.font, at: 0, effectiveRange: nil) as? UIFont
        XCTAssertNotNil(font)
        XCTAssertTrue(try XCTUnwrap(font?.fontDescriptor.symbolicTraits.contains(.traitItalic)),
                      "Italic run should use an italic font")
    }

    func testLinkRunHasLinkAttribute() {
        let block = plain("link", start: 0, end: 4, linkUrl: "https://example.com")
        let result = renderer.renderBlock(block)
        let link = result.attribute(.link, at: 0, effectiveRange: nil)
        XCTAssertNotNil(link, "Link run should have .link attribute")
    }

    func testMultiBlockDocumentHasNewlineSeparator() {
        let attrs = FfiAttributeSet(bold: false, italic: false, strikeThrough: false,
                                    underline: false, inlineCode: false, linkUrl: nil)
        let blocks = [
            FfiBlockProjection(blockId: "0", kind: .paragraph, inQuote: false, startUtf16: 0, endUtf16: 2, inlineRuns: [
                FfiInlineRun(nodeId: "0,0", startUtf16: 0, endUtf16: 2, kind: .text(text: "hi", attributes: attrs)),
            ]),
            FfiBlockProjection(blockId: "1", kind: .paragraph, inQuote: false, startUtf16: 3, endUtf16: 5, inlineRuns: [
                FfiInlineRun(nodeId: "1,0", startUtf16: 3, endUtf16: 5, kind: .text(text: "yo", attributes: attrs)),
            ]),
        ]
        let (result, _) = renderer.render(projections: blocks)
        XCTAssertEqual(result.string, "hi\nyo")
    }

    func testCodeBlockUsesMonospacedFont() throws {
        let attrs = FfiAttributeSet(bold: false, italic: false, strikeThrough: false,
                                    underline: false, inlineCode: false, linkUrl: nil)
        let run = FfiInlineRun(nodeId: "0", startUtf16: 0, endUtf16: 4,
                               kind: .text(text: "code", attributes: attrs))
        let block = FfiBlockProjection(blockId: "0", kind: .codeBlock, inQuote: false, startUtf16: 0, endUtf16: 4, inlineRuns: [run])
        let result = renderer.renderBlock(block)
        let font = result.attribute(.font, at: 0, effectiveRange: nil) as? UIFont
        XCTAssertNotNil(font)
        XCTAssertTrue(try XCTUnwrap(font?.fontDescriptor.symbolicTraits.contains(.traitMonoSpace)),
                      "Code block should use a monospaced font")
    }

    func testMentionFallbackProducesLinkAttribute() {
        let run = FfiInlineRun(nodeId: "0", startUtf16: 0, endUtf16: 1,
                               kind: .mention(url: "https://matrix.to/#/@alice:example.com", displayText: "Alice"))
        let block = FfiBlockProjection(blockId: "0", kind: .paragraph, inQuote: false, startUtf16: 0, endUtf16: 1, inlineRuns: [run])
        let result = renderer.renderBlock(block)
        XCTAssertEqual(result.string, "Alice")
        let link = result.attribute(.link, at: 0, effectiveRange: nil)
        XCTAssertNotNil(link, "Mention without replacer should fall back to a link attribute")
    }
}

// MARK: - ComposerModelWrapper new method tests

final class ComposerModelWrapperProjectionTests: XCTestCase {
    private var wrapper: ComposerModelWrapper!

    override func setUp() {
        wrapper = ComposerModelWrapper()
    }

    func testGetBlockProjectionsOnEmptyModel() {
        let projections = wrapper.getBlockProjections()
        // Empty model returns 0 projections (no content yet)
        XCTAssertEqual(projections.count, 0)
    }

    func testGetBlockProjectionsAfterSettingContent() {
        _ = wrapper.setContentFromHtml(html: "<p>hello</p><p>world</p>")
        let projections = wrapper.getBlockProjections()
        XCTAssertEqual(projections.count, 2)
    }
}

// MARK: - WysiwygComposerViewModel projection pipeline tests

final class WysiwygComposerViewModelProjectionTests: XCTestCase {
    private var viewModel: WysiwygComposerViewModel!

    override func setUp() {
        viewModel = WysiwygComposerViewModel()
        viewModel.clearContent()
    }

    func testSetContentProducesAttributedText() {
        viewModel.setHtmlContent("<p>hello</p>")
        XCTAssertEqual(viewModel.attributedContent.text.string, "hello")
    }

    func testTwoBlocksProduceNewlineSeparator() {
        viewModel.setHtmlContent("<p>hello</p><p>world</p>")
        XCTAssertEqual(viewModel.attributedContent.text.string, "hello\nworld")
    }

    func testSelectionIsDirectUtf16Offset() {
        // Set content and select characters 1–3 (0-indexed)
        viewModel.setHtmlContent("<p>hello</p>")
        viewModel.select(range: NSRange(location: 1, length: 2))
        XCTAssertEqual(viewModel.attributedContent.selection.location, 1)
        XCTAssertEqual(viewModel.attributedContent.selection.length, 2,
                       "With projection renderer, selection should be direct UTF-16 offsets")
    }

    func testIsContentEmptyOnEmptyModel() {
        XCTAssertTrue(viewModel.isContentEmpty)
    }

    func testIsContentEmptyFalseAfterInput() {
        XCTAssertTrue(viewModel.isContentEmpty)
        viewModel.setHtmlContent("<p>hello</p>")
        XCTAssertFalse(viewModel.attributedContent.text.string.isEmpty,
                       "attributedContent should have text after setting content")
        XCTAssertFalse(viewModel.isContentEmpty, "isContentEmpty should be false when there is content")
    }

    func testClearContentProducesEmptyAttributedText() {
        viewModel.setHtmlContent("<p>hello</p>")
        viewModel.clearContent()
        XCTAssertTrue(viewModel.isContentEmpty)
    }
}
