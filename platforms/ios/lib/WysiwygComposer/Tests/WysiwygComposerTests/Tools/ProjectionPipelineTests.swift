//
// Copyright (c) 2026 Element Creations Ltd
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import HTMLParser
import Testing
import UIKit
@testable import WysiwygComposer

// MARK: - computePrefixSuffixDiff tests

@MainActor
struct ComputePrefixSuffixDiffTests {
    @Test func singleCharInsertion() {
        let diff = computePrefixSuffixDiff(old: "helo", new: "hello")
        #expect(diff.replaceStart == 3)
        #expect(diff.replaceEnd == 3)
        #expect(diff.replacement == "l")
    }

    @Test func singleCharDeletion() {
        let diff = computePrefixSuffixDiff(old: "hello", new: "helo")
        #expect(diff.replaceStart == 3)
        #expect(diff.replaceEnd == 4)
        #expect(diff.replacement == "")
    }

    @Test func wordReplacement() {
        let diff = computePrefixSuffixDiff(old: "teh", new: "the")
        #expect(diff.replaceStart == 1)
        #expect(diff.replaceEnd == 3)
        #expect(diff.replacement == "he")
    }

    @Test func emptyOldString() {
        let diff = computePrefixSuffixDiff(old: "", new: "abc")
        #expect(diff.replaceStart == 0)
        #expect(diff.replaceEnd == 0)
        #expect(diff.replacement == "abc")
    }

    @Test func emptyNewString() {
        let diff = computePrefixSuffixDiff(old: "abc", new: "")
        #expect(diff.replaceStart == 0)
        #expect(diff.replaceEnd == 3)
        #expect(diff.replacement == "")
    }

    @Test func identicalStrings() {
        let diff = computePrefixSuffixDiff(old: "unchanged", new: "unchanged")
        #expect(diff.replaceStart == 9)
        #expect(diff.replaceEnd == 9)
        #expect(diff.replacement == "")
    }

    @Test func trailingInsert() {
        let diff = computePrefixSuffixDiff(old: "hello", new: "hello!")
        #expect(diff.replaceStart == 5)
        #expect(diff.replaceEnd == 5)
        #expect(diff.replacement == "!")
    }

    @Test func leadingInsert() {
        let diff = computePrefixSuffixDiff(old: "world", new: "hello world")
        #expect(diff.replaceStart == 0)
        #expect(diff.replaceEnd == 0)
        #expect(diff.replacement == "hello ")
    }
}

// MARK: - ProjectionRenderer tests

@MainActor
struct ProjectionRendererTests {
    private let renderer = ProjectionRenderer(style: .standard, mentionReplacer: nil)

    private func plain(_ text: String,
                       start: UInt32,
                       end: UInt32,
                       bold: Bool = false,
                       italic: Bool = false,
                       inlineCode: Bool = false,
                       linkUrl: String? = nil) -> FfiBlockProjection {
        let attrs = FfiAttributeSet(bold: bold, italic: italic, strikeThrough: false,
                                    underline: false, inlineCode: inlineCode, linkUrl: linkUrl)
        let run = FfiInlineRun(nodeId: "0", startUtf16: start, endUtf16: end,
                               kind: .text(text: text, attributes: attrs))
        return FfiBlockProjection(blockId: "0", kind: .paragraph, inQuote: false,
                                  startUtf16: start, endUtf16: end, inlineRuns: [run])
    }

    @Test func paragraphRendersPlainText() {
        let block = plain("hello", start: 0, end: 5)
        let result = renderer.renderBlock(block)
        #expect(result.string == "hello")
    }

    @Test func boldRunHasBoldFont() throws {
        let block = plain("bold", start: 0, end: 4, bold: true)
        let result = renderer.renderBlock(block)
        let font = try #require(result.attribute(.font, at: 0, effectiveRange: nil) as? UIFont)
        #expect(font.fontDescriptor.symbolicTraits.contains(.traitBold), "Bold run should use a bold font")
    }

    @Test func italicRunHasItalicFont() throws {
        let block = plain("italic", start: 0, end: 6, italic: true)
        let result = renderer.renderBlock(block)
        let font = try #require(result.attribute(.font, at: 0, effectiveRange: nil) as? UIFont)
        #expect(font.fontDescriptor.symbolicTraits.contains(.traitItalic), "Italic run should use an italic font")
    }

    @Test func linkRunHasLinkAttribute() {
        let block = plain("link", start: 0, end: 4, linkUrl: "https://example.com")
        let result = renderer.renderBlock(block)
        #expect(result.attribute(.link, at: 0, effectiveRange: nil) != nil, "Link run should have .link attribute")
    }

    @Test func multiBlockDocumentHasNewlineSeparator() {
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
        #expect(result.string == "hi\nyo")
    }

    @Test func codeBlockUsesMonospacedFont() throws {
        let attrs = FfiAttributeSet(bold: false, italic: false, strikeThrough: false,
                                    underline: false, inlineCode: false, linkUrl: nil)
        let run = FfiInlineRun(nodeId: "0", startUtf16: 0, endUtf16: 4,
                               kind: .text(text: "code", attributes: attrs))
        let block = FfiBlockProjection(blockId: "0", kind: .codeBlock, inQuote: false,
                                       startUtf16: 0, endUtf16: 4, inlineRuns: [run])
        let result = renderer.renderBlock(block)
        let font = try #require(result.attribute(.font, at: 0, effectiveRange: nil) as? UIFont)
        #expect(font.fontDescriptor.symbolicTraits.contains(.traitMonoSpace), "Code block should use a monospaced font")
    }

    @Test func mentionFallbackProducesLinkAttribute() {
        let run = FfiInlineRun(nodeId: "0", startUtf16: 0, endUtf16: 1,
                               kind: .mention(url: "https://matrix.to/#/@alice:example.com", displayText: "Alice"))
        let block = FfiBlockProjection(blockId: "0", kind: .paragraph, inQuote: false,
                                       startUtf16: 0, endUtf16: 1, inlineRuns: [run])
        let result = renderer.renderBlock(block)
        #expect(result.string == "Alice")
        #expect(result.attribute(.link, at: 0, effectiveRange: nil) != nil,
                "Mention without replacer should fall back to a link attribute")
    }
}

// MARK: - ComposerModelWrapper projection API tests

@MainActor
struct ComposerModelWrapperProjectionTests {
    let wrapper = ComposerModelWrapper()

    @Test func getBlockProjectionsOnEmptyModel() {
        // Empty model returns 0 projections (no content yet)
        #expect(wrapper.getBlockProjections().isEmpty)
    }

    @Test func getBlockProjectionsAfterSettingContent() {
        _ = wrapper.setContentFromHtml(html: "<p>hello</p><p>world</p>")
        #expect(wrapper.getBlockProjections().count == 2)
    }
}

// MARK: - WysiwygComposerViewModel projection pipeline tests

@MainActor
struct WysiwygComposerViewModelProjectionTests {
    let viewModel = WysiwygComposerViewModel()

    init() {
        viewModel.clearContent()
    }

    @Test func setContentProducesAttributedText() {
        viewModel.setHtmlContent("<p>hello</p>")
        #expect(viewModel.attributedContent.text.string == "hello")
    }

    @Test func twoBlocksProduceNewlineSeparator() {
        viewModel.setHtmlContent("<p>hello</p><p>world</p>")
        #expect(viewModel.attributedContent.text.string == "hello\nworld")
    }

    @Test func selectionIsDirectUtf16Offset() {
        // Set content and select characters 1–3 (0-indexed)
        viewModel.setHtmlContent("<p>hello</p>")
        viewModel.select(range: NSRange(location: 1, length: 2))
        #expect(viewModel.attributedContent.selection.location == 1)
        #expect(viewModel.attributedContent.selection.length == 2,
                "With projection renderer, selection should be direct UTF-16 offsets")
    }

    @Test func isContentEmptyOnEmptyModel() {
        #expect(viewModel.isContentEmpty)
    }

    @Test func isContentEmptyFalseAfterInput() {
        #expect(viewModel.isContentEmpty)
        viewModel.setHtmlContent("<p>hello</p>")
        #expect(!viewModel.attributedContent.text.string.isEmpty,
                "attributedContent should have text after setting content")
        #expect(!viewModel.isContentEmpty, "isContentEmpty should be false when there is content")
    }

    @Test func clearContentProducesEmptyAttributedText() {
        viewModel.setHtmlContent("<p>hello</p>")
        viewModel.clearContent()
        #expect(viewModel.isContentEmpty)
    }
}
