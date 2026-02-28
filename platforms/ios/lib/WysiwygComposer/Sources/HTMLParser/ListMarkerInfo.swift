//
// Copyright (c) 2026 Element Creations Ltd
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import UIKit

/// Information needed to draw a list marker (bullet or ordinal) in the paragraph gutter.
/// The marker is drawn visually but is NOT part of the editable text content,
/// preserving 1-to-1 offset mapping with the Rust model.
public struct ListMarkerInfo: Equatable {
    /// The marker text, e.g. "1." or "•".
    public let text: String
    /// The font used to render the marker.
    public let font: UIFont
    /// The foreground color of the marker text.
    public let color: UIColor
    /// The character index (in the attributed string) where this list item's
    /// content begins. Used to locate the correct line fragment for drawing.
    /// For empty list items this equals the block's start position (which may
    /// be the `\n` separator or the end of the string).
    public let characterIndex: Int
    /// `headIndent` from the list item's paragraph style — used to position
    /// the marker in the gutter.
    public let headIndent: CGFloat

    public init(text: String, font: UIFont, color: UIColor,
                characterIndex: Int = 0, headIndent: CGFloat = 26) {
        self.text = text
        self.font = font
        self.color = color
        self.characterIndex = characterIndex
        self.headIndent = headIndent
    }
}
