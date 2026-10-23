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

    /// A paragraph starting with a mention (one code unit in the model), optionally followed by text.
    private func mention(followedBy text: String? = nil) -> FfiBlockProjection {
        let url = "https://matrix.to/#/@alice:example.com"
        var runs = [FfiInlineRun(nodeId: "0,0", startUtf16: 0, endUtf16: 1,
                                 kind: .mention(url: url, displayText: "Alice"))]
        var end: UInt32 = 1
        if let text {
            end += UInt32(text.utf16.count)
            let attrs = FfiAttributeSet(bold: false, italic: false, strikeThrough: false,
                                        underline: false, inlineCode: false, linkUrl: nil)
            runs.append(FfiInlineRun(nodeId: "0,1", startUtf16: 1, endUtf16: end,
                                     kind: .text(text: text, attributes: attrs)))
        }
        return FfiBlockProjection(blockId: "0", kind: .paragraph, inQuote: false,
                                  startUtf16: 0, endUtf16: end, inlineRuns: runs)
    }

    @Test func mentionWithoutReplacerIsOneCodeUnit() {
        let result = renderer.renderBlock(mention())
        #expect(result.length == 1, "A mention is one code unit in the model and must stay one code unit in the text")
        #expect(result.attribute(.attachment, at: 0, effectiveRange: nil) != nil,
                "The placeholder draws the display text as an attachment")
        #expect(result.attribute(.link, at: 0, effectiveRange: nil) != nil,
                "The placeholder keeps the mention link")
    }

    @Test func textAfterMentionKeepsItsModelOffsets() {
        let result = renderer.renderBlock(mention(followedBy: " hi"))
        #expect(result.length == 4)
        #expect((result.string as NSString).substring(from: 1) == " hi")
    }

    @Test func oneCodeUnitPillFromReplacerIsUsed() {
        let renderer = ProjectionRenderer(style: .standard, mentionReplacer: OneUnitPillReplacer())
        let result = renderer.renderBlock(mention())
        #expect(result.length == 1)
        #expect(result.attribute(OneUnitPillReplacer.marker, at: 0, effectiveRange: nil) != nil,
                "The host's pill is used as is")
    }
}

/// A host pill: one code unit, marked so a test can tell it apart from the placeholder.
private struct OneUnitPillReplacer: HTMLMentionReplacer {
    static var marker: NSAttributedString.Key { .init("TestPillMarker") }

    func replacementForMention(_ url: String, text: String) -> NSAttributedString? {
        NSAttributedString(string: "\u{FFFC}", attributes: [.attachment: NSTextAttachment(), Self.marker: text])
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
