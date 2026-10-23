//
// Copyright (c) 2026 Element Creations Ltd
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import UIKit

// MARK: - Data types

/// The result of a prefix/suffix diff between two strings.
public struct InlineDiff {
    /// Offset (in UTF-16 code units, relative to block start) of the first changed code unit.
    public let replaceStart: Int
    /// Offset (in UTF-16 code units, relative to block start) of the exclusive end of the replaced range.
    public let replaceEnd: Int
    /// The replacement text (UTF-16 code units).
    public let replacement: String
}

// MARK: - Inline diff

/// O(n) prefix/suffix diff over UTF-16 code units.
///
/// Since edits are always block-scoped and typically small (single keystrokes,
/// autocorrect replacements) this is cheaper than a full Myers diff.
///
/// - Parameters:
///   - old: The old block string before the UIKit edit.
///   - new: The new block string after the UIKit edit.
/// - Returns: The minimal replacement range and text.
public func computePrefixSuffixDiff(old: String, new: String) -> InlineDiff {
    let oldUnits = Array(old.utf16)
    let newUnits = Array(new.utf16)

    var prefixLen = 0
    while prefixLen < oldUnits.count, prefixLen < newUnits.count,
          oldUnits[prefixLen] == newUnits[prefixLen] {
        prefixLen += 1
    }

    var suffixLen = 0
    while suffixLen < (oldUnits.count - prefixLen),
          suffixLen < (newUnits.count - prefixLen),
          oldUnits[oldUnits.count - 1 - suffixLen] == newUnits[newUnits.count - 1 - suffixLen] {
        suffixLen += 1
    }

    let replaceEnd = oldUnits.count - suffixLen
    let newEnd = newUnits.count - suffixLen
    let replacementUnits = Array(newUnits[prefixLen..<newEnd])

    let replacement: String
    if replacementUnits.isEmpty {
        replacement = ""
    } else {
        replacement = String(decoding: replacementUnits, as: UTF16.self)
    }

    return InlineDiff(
        replaceStart: prefixLen,
        replaceEnd: replaceEnd,
        replacement: replacement
    )
}

// MARK: - Inline style check

/// Whether `displayed` and `rendered` disagree, anywhere in `range`, on the attributes that
/// visibly mark inline formatting: the link, the inline-code background and the font's
/// bold / italic / monospace traits.
///
/// UIKit gives typed text the attributes of its neighbour, so text typed at the edge of a
/// link is displayed as part of the link although the model puts it outside. Only these
/// attributes are compared, and fonts only by their traits, so that attributes UIKit adds on
/// its own do not register as a mismatch. A missing font on either side is not a mismatch
/// either: it only happens for attributed strings built without attributes.
func hasInlineStyleMismatch(displayed: NSAttributedString,
                            rendered: NSAttributedString,
                            in range: NSRange) -> Bool {
    guard range.length > 0,
          displayed.length == rendered.length,
          range.upperBound <= displayed.length else {
        return false
    }

    var index = range.location
    while index < range.upperBound {
        var displayedRange = NSRange()
        var renderedRange = NSRange()
        let displayedAttrs = displayed.attributes(at: index, longestEffectiveRange: &displayedRange, in: range)
        let renderedAttrs = rendered.attributes(at: index, longestEffectiveRange: &renderedRange, in: range)

        if linkIdentifier(displayedAttrs[.link]) != linkIdentifier(renderedAttrs[.link]) {
            return true
        }
        if (displayedAttrs[.backgroundColor] as? UIColor) != (renderedAttrs[.backgroundColor] as? UIColor) {
            return true
        }
        if let displayedFont = displayedAttrs[.font] as? UIFont,
           let renderedFont = renderedAttrs[.font] as? UIFont,
           inlineTraits(of: displayedFont) != inlineTraits(of: renderedFont) {
            return true
        }

        index = min(displayedRange.upperBound, renderedRange.upperBound)
    }
    return false
}

/// `.link` values can be a `URL` or a `String`; compare them by their text.
private func linkIdentifier(_ value: Any?) -> String? {
    switch value {
    case let url as URL:
        return url.absoluteString
    case let string as String:
        return string
    default:
        return nil
    }
}

private func inlineTraits(of font: UIFont) -> UIFontDescriptor.SymbolicTraits {
    font.fontDescriptor.symbolicTraits.intersection([.traitBold, .traitItalic, .traitMonoSpace])
}
