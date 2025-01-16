//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import UIKit

public extension UITextView {
    /// Draw layers for all the HTML elements that require special background.
    func drawBackgroundStyleLayers() {
        layer
            .sublayers?[0]
            .sublayers?
            .compactMap { $0 as? BackgroundStyleLayer }
            .forEach { $0.removeFromSuperlayer() }

        attributedText.enumerateTypedAttribute(.blockStyle) { (style: BlockStyle, range: NSRange, _) in
            let styleLayer: BackgroundStyleLayer
            let glyphRange = layoutManager.glyphRange(forCharacterRange: range, actualCharacterRange: nil)
            switch style.type {
            case .background:
                let rect = layoutManager
                    .boundingRect(forGlyphRange: glyphRange, in: self.textContainer)
                    /// Extend horizontally to the enclosing frame, and extend to half of the vertical  padding.
                    .extendHorizontally(in: frame, withVerticalPadding: style.padding.vertical / 2.0)

                styleLayer = BackgroundStyleLayer(style: style, frame: rect)
            case let .side(offset, width):
                let textRect = layoutManager
                    .boundingRect(forGlyphRange: glyphRange, in: self.textContainer)
                let rect = CGRect(x: offset, y: textRect.origin.y, width: width, height: textRect.size.height)
                styleLayer = BackgroundStyleLayer(style: style, frame: rect)
            }
            layer.sublayers?[0].insertSublayer(styleLayer, at: UInt32(layer.sublayers?.count ?? 0))
        }
    }
}

private final class BackgroundStyleLayer: CALayer {
    override init() {
        super.init()
    }

    init(style: BlockStyle, frame: CGRect) {
        super.init()

        self.frame = frame
        backgroundColor = style.backgroundColor.cgColor
        borderWidth = style.borderWidth
        borderColor = style.borderColor.cgColor
        cornerRadius = style.cornerRadius
    }

    required init?(coder: NSCoder) {
        super.init(coder: coder)
    }
}
