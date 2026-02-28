//
// Copyright (c) 2026 Element Creations Ltd
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import HTMLParser
import UIKit
import WysiwygComposerFFI

/// Builds an `NSAttributedString` directly from a `[FfiBlockProjection]`
/// without going through HTML or DTCoreText.
///
/// This is the replacement for the `HTMLParser` → DTCoreText rendering pipeline
/// on iOS.  Unicode offsets in the produced attributed string correspond 1-to-1
/// with the UTF-16 offsets on the Rust `BlockProjection` API.
public struct ProjectionRenderer {
    // MARK: - Properties

    public let style: HTMLParserStyle
    public let mentionReplacer: HTMLMentionReplacer?

    // MARK: - Init

    public init(style: HTMLParserStyle = .standard,
                mentionReplacer: HTMLMentionReplacer? = nil) {
        self.style = style
        self.mentionReplacer = mentionReplacer
    }

    // MARK: - Public

    /// Build a full `NSAttributedString` from all blocks.
    ///
    /// Blocks are separated by a single `\n` code unit, which matches the
    /// virtual inter-block separator that the Rust projection layer counts.
    ///
    /// Returns the attributed string and an array of list markers to be
    /// drawn in the gutter (not part of the text content).
    public func render(projections: [FfiBlockProjection]) -> (NSAttributedString, [ListMarkerInfo]) {
        let result = NSMutableAttributedString()
        var markers: [ListMarkerInfo] = []
        // Track ordinal counters for ordered lists, keyed by depth.
        var ordinalCounters: [UInt32: Int] = [:]

        for (i, block) in projections.enumerated() {
            let prevBlock = i > 0 ? projections[i - 1] : nil
            let nextBlock = i < projections.count - 1 ? projections[i + 1] : nil

            // Determine the position of this block within a consecutive run of
            // the same container kind (e.g. code block lines, quote paragraphs,
            // or any adjacent blocks inside the same blockquote).
            let continuesFromPrev = prevBlock.map {
                areSiblingBlocksInSameContainer($0.kind, $0.inQuote, block.kind, block.inQuote)
            } ?? false
            let continuesToNext = nextBlock.map {
                areSiblingBlocksInSameContainer(block.kind, block.inQuote, $0.kind, $0.inQuote)
            } ?? false

            // Compute list marker info (drawn visually, not inserted into text).
            let listMarkerText: String?
            let listDepth: UInt32
            switch block.kind {
            case .listItemOrdered(let depth):
                let count = (ordinalCounters[depth] ?? 0) + 1
                ordinalCounters[depth] = count
                // Reset deeper counters when we encounter an item at this depth.
                ordinalCounters = ordinalCounters.filter { $0.key <= depth }
                listMarkerText = "\(count)."
                listDepth = depth
            case .listItemUnordered(let depth):
                listMarkerText = "\u{2022}"
                listDepth = depth
            default:
                listMarkerText = nil
                listDepth = 1
                // Reset ordinal counters when we leave list context.
                if !ordinalCounters.isEmpty {
                    ordinalCounters.removeAll()
                }
            }

            // Record the character position where this block starts.
            let blockStart = result.length

            appendBlock(block, to: result,
                        continuesFromPrevious: continuesFromPrev,
                        continuesToNext: continuesToNext)

            // Build the marker info with the now-known character offset.
            if let listMarkerText {
                let paraStyle = listParagraphStyle(depth: listDepth, inQuote: block.inQuote)
                markers.append(ListMarkerInfo(
                    text: listMarkerText,
                    font: UIFont.preferredFont(forTextStyle: .body),
                    color: style.textColor,
                    characterIndex: blockStart,
                    headIndent: paraStyle.headIndent
                ))
            }

            if let nextBlock {
                // Between blocks inside the same structural container, use a
                // plain-line separator (no paragraph padding) so they render as
                // tightly-spaced lines within one visual block.
                let separatorAttrs: [NSAttributedString.Key: Any]
                if areSiblingBlocksInSameContainer(block.kind, block.inQuote, nextBlock.kind, nextBlock.inQuote) {
                    separatorAttrs = intraBlockSeparatorAttributes(for: block.kind, inQuote: block.inQuote)
                } else {
                    separatorAttrs = baseAttributes(for: block.kind, inQuote: block.inQuote)
                }
                result.append(NSAttributedString(
                    string: "\n",
                    attributes: separatorAttrs
                ))
            }
        }
        return (result, markers)
    }

    /// Build the attributed string for a single block projection.
    public func renderBlock(_ block: FfiBlockProjection) -> NSAttributedString {
        let result = NSMutableAttributedString()
        appendBlock(block, to: result, continuesFromPrevious: false, continuesToNext: false)
        return result
    }

    // MARK: - Private helpers

    private func appendBlock(_ block: FfiBlockProjection,
                             to result: NSMutableAttributedString,
                             continuesFromPrevious: Bool,
                             continuesToNext: Bool) {
        // When this block is a middle/last line in a multi-line container
        // (e.g. a code block), strip paragraphSpacingBefore so there's no
        // extra gap above this line. Similarly strip paragraphSpacing
        // when this is a first/middle line.
        let needsStrippedParagraphStyle = continuesFromPrevious || continuesToNext

        // Compute the effective paragraph style once – used by every text
        // run so the whole paragraph is consistent.
        let effectiveParagraphStyle: NSParagraphStyle? = needsStrippedParagraphStyle
            ? strippedParagraphStyle(
                for: block.kind,
                inQuote: block.inQuote,
                isFirst: !continuesFromPrevious,
                isLast: !continuesToNext
            )
            : nil

        for run in block.inlineRuns {
            switch run.kind {
            case let .text(text, attrs):
                var runAttrs = inlineAttributes(for: attrs, blockKind: block.kind, inQuote: block.inQuote)
                if let effectiveParagraphStyle {
                    runAttrs[.paragraphStyle] = effectiveParagraphStyle
                }
                // Code blocks use \n in the text for line breaks, but UIKit treats
                // \n as a paragraph break (with paragraph spacing). Replace with
                // \u{2028} (LINE SEPARATOR) which is a soft line break within the
                // same paragraph — no paragraph spacing applied.
                let displayText: String
                if case .codeBlock = block.kind {
                    displayText = text.replacingOccurrences(of: "\n", with: "\u{2028}")
                } else {
                    displayText = text
                }
                result.append(NSAttributedString(string: displayText, attributes: runAttrs))

            case let .mention(url, displayText):
                let pill = mentionReplacer?.replacementForMention(url, text: displayText)
                    ?? NSAttributedString(
                        string: displayText,
                        attributes: [.link: URL(string: url) as Any,
                                     .foregroundColor: style.linkColor]
                    )
                result.append(pill)

            case .lineBreak:
                result.append(NSAttributedString(
                    string: "\n",
                    attributes: baseAttributes(for: block.kind, inQuote: block.inQuote)
                ))
            }
        }
    }

    // MARK: - Attribute builders

    private func baseAttributes(for kind: FfiBlockKind, inQuote: Bool = false) -> [NSAttributedString.Key: Any] {
        var attrs: [NSAttributedString.Key: Any] = [
            .foregroundColor: style.textColor,
            .font: UIFont.preferredFont(forTextStyle: .body),
        ]

        switch kind {
        case .codeBlock:
            attrs[.font] = monospacedFont()
            attrs[.blockStyle] = style.codeBlockStyle
            attrs[.paragraphStyle] = style.codeBlockStyle.paragraphStyle
        case .quote:
            attrs[.blockStyle] = style.quoteBlockStyle
            attrs[.paragraphStyle] = style.quoteBlockStyle.paragraphStyle
        case .listItemOrdered(let depth):
            attrs[.paragraphStyle] = listParagraphStyle(depth: depth, inQuote: inQuote)
            if inQuote {
                attrs[.blockStyle] = style.quoteBlockStyle
            }
        case .listItemUnordered(let depth):
            attrs[.paragraphStyle] = listParagraphStyle(depth: depth, inQuote: inQuote)
            if inQuote {
                attrs[.blockStyle] = style.quoteBlockStyle
            }
        default:
            attrs[.paragraphStyle] = defaultParagraphStyle()
        }
        return attrs
    }

    private func inlineAttributes(for attrs: FfiAttributeSet,
                                  blockKind: FfiBlockKind,
                                  inQuote: Bool = false) -> [NSAttributedString.Key: Any] {
        var result = baseAttributes(for: blockKind, inQuote: inQuote)

        let baseFont: UIFont
        if case .codeBlock = blockKind {
            baseFont = monospacedFont()
        } else {
            baseFont = UIFont.preferredFont(forTextStyle: .body)
        }

        result[.font] = resolvedFont(base: baseFont, bold: attrs.bold, italic: attrs.italic)

        if attrs.strikeThrough {
            result[.strikethroughStyle] = NSUnderlineStyle.single.rawValue
        }
        if attrs.underline {
            result[.underlineStyle] = NSUnderlineStyle.single.rawValue
        }
        if attrs.inlineCode {
            result[.font] = monospacedFont()
            result[.backgroundColor] = style.codeBlockStyle.backgroundColor
        }
        if let urlString = attrs.linkUrl, let url = URL(string: urlString) {
            result[.link] = url
            result[.foregroundColor] = style.linkColor
            // Remove underline to match old pipeline (UIKit adds underline to links by default)
            result[.underlineStyle] = 0
            result[.underlineColor] = UIColor.clear
        }
        return result
    }

    private func resolvedFont(base: UIFont, bold: Bool, italic: Bool) -> UIFont {
        var descriptor = base.fontDescriptor
        var traits = descriptor.symbolicTraits
        if bold { traits.insert(.traitBold) }
        if italic { traits.insert(.traitItalic) }
        if let updated = descriptor.withSymbolicTraits(traits) {
            descriptor = updated
        }
        return UIFont(descriptor: descriptor, size: base.pointSize)
    }

    private func monospacedFont() -> UIFont {
        UIFont.monospacedSystemFont(ofSize: UIFont.preferredFont(forTextStyle: .body).pointSize,
                                    weight: .regular)
    }

    /// Paragraph style for list items with indentation to leave gutter space
    /// for the visually-drawn marker (bullet or ordinal).
    private func listParagraphStyle(depth: UInt32, inQuote: Bool) -> NSParagraphStyle {
        let paraStyle = NSMutableParagraphStyle()
        let baseIndent: CGFloat = inQuote ? style.quoteBlockStyle.padding.horizontal : 0
        let markerGutterWidth: CGFloat = 26.0
        let depthOffset = CGFloat(depth - 1) * markerGutterWidth

        paraStyle.firstLineHeadIndent = baseIndent + depthOffset + markerGutterWidth
        paraStyle.headIndent = baseIndent + depthOffset + markerGutterWidth

        if inQuote {
            paraStyle.tailIndent = -style.quoteBlockStyle.padding.horizontal
            paraStyle.paragraphSpacingBefore = style.quoteBlockStyle.padding.vertical
            paraStyle.paragraphSpacing = style.quoteBlockStyle.padding.vertical
        } else {
            paraStyle.paragraphSpacing = 0
            paraStyle.paragraphSpacingBefore = 0
        }
        return paraStyle
    }

    /// Default paragraph style with zeroed vertical spacing (matches old pipeline's
    /// `removeParagraphVerticalSpacing`).
    private func defaultParagraphStyle() -> NSParagraphStyle {
        let paraStyle = NSMutableParagraphStyle()
        paraStyle.paragraphSpacing = 0
        paraStyle.paragraphSpacingBefore = 0
        return paraStyle
    }

    /// Whether two consecutive block kinds are siblings inside the same structural
    /// container (e.g. two CodeBlock lines from a single `<pre>`, or two Quote
    /// paragraphs from a single `<blockquote>`).
    private func areSiblingBlocksInSameContainer(_ a: FfiBlockKind, _ aInQuote: Bool,
                                                 _ b: FfiBlockKind, _ bInQuote: Bool) -> Bool {
        switch (a, b) {
        case (.quote, .quote):
            return true
        default:
            // Consecutive blocks inside the same blockquote are siblings.
            return aInQuote && bInQuote
        }
    }

    /// Attributes for the `"\n"` separator between lines within the same visual
    /// block (e.g. code block or quote). Uses the same font and block style but
    /// strips paragraph vertical padding so lines are tightly spaced.
    private func intraBlockSeparatorAttributes(for kind: FfiBlockKind, inQuote: Bool = false) -> [NSAttributedString.Key: Any] {
        var attrs = baseAttributes(for: kind, inQuote: inQuote)
        let paraStyle = NSMutableParagraphStyle()
        paraStyle.paragraphSpacing = 0
        paraStyle.paragraphSpacingBefore = 0
        // Preserve horizontal indent from any block style or list paragraph style.
        if let listStyle = attrs[.paragraphStyle] as? NSParagraphStyle,
           listStyle.headIndent > 0 || listStyle.firstLineHeadIndent > 0 {
            paraStyle.firstLineHeadIndent = listStyle.firstLineHeadIndent
            paraStyle.headIndent = listStyle.headIndent
            paraStyle.tailIndent = listStyle.tailIndent
        } else if let blockStyle = attrs[.blockStyle] as? BlockStyle {
            paraStyle.firstLineHeadIndent = blockStyle.padding.horizontal
            paraStyle.headIndent = blockStyle.padding.horizontal
            paraStyle.tailIndent = -blockStyle.padding.horizontal
        }
        attrs[.paragraphStyle] = paraStyle
        return attrs
    }

    /// Return a paragraph style that keeps the block's indentation but selectively
    /// strips vertical spacing depending on the line's position in a multi-line
    /// container run.
    ///
    /// - `isFirst`: true when this line is the first in the run (keep `paragraphSpacingBefore`)
    /// - `isLast`:  true when this line is the last in the run (keep `paragraphSpacing`)
    private func strippedParagraphStyle(for kind: FfiBlockKind,
                                        inQuote: Bool = false,
                                        isFirst: Bool,
                                        isLast: Bool) -> NSParagraphStyle {
        let base = baseAttributes(for: kind, inQuote: inQuote)[.paragraphStyle] as? NSParagraphStyle
        let para = (base?.mutableCopy() as? NSMutableParagraphStyle) ?? NSMutableParagraphStyle()
        if !isFirst {
            para.paragraphSpacingBefore = 0
        }
        if !isLast {
            para.paragraphSpacing = 0
        }
        return para
    }
}
