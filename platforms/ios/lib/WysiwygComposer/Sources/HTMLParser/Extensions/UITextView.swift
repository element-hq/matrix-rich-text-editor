//
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
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
                    // Extend horizontally to the enclosing frame, and extend to half of the vertical  padding.
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

    /// Draw list markers (bullets, ordinals) in the paragraph gutter.
    ///
    /// Markers are provided as a side-channel array (not in the attributed string)
    /// so the text offsets stay in 1-to-1 correspondence with the Rust model.
    /// Each marker carries its character index so we can find the correct
    /// line-fragment rect even for empty list-item paragraphs.
    ///
    /// Uses a dedicated container CALayer so markers are reliably removed
    /// and recreated on every update — no possibility of stale layers.
    func drawListMarkers(_ markers: [ListMarkerInfo]) {
        // Disable implicit Core Animation actions so layers appear/disappear
        // instantly without crossfade or position animations.
        CATransaction.begin()
        CATransaction.setDisableActions(true)
        defer { CATransaction.commit() }

        // --- DIAGNOSTIC LOGGING (remove after debugging) ---
        let existingCount = (layer.sublayers?.first(where: { $0.name == "ListMarkerContainer" })?.sublayers?.count) ?? 0
        print("[ListMarkers] drawListMarkers called: \(markers.count) markers, existing container sublayers: \(existingCount), textLen: \(attributedText.length)")
        for (i, m) in markers.enumerated() {
            print("[ListMarkers]   marker[\(i)]: text='\(m.text)' charIdx=\(m.characterIndex) headIndent=\(m.headIndent)")
        }

        // Add markers to the scrollable text-container layer so they move with
        // the text content during scrolling.
        let hostLayer = layer.sublayers?.first ?? layer

        // Lazy-create a dedicated container layer the first time.
        let container: CALayer
        if let existing = hostLayer.sublayers?.first(where: { $0.name == "ListMarkerContainer" }) {
            container = existing
            // Remove ALL previous marker sublayers.
            container.sublayers?.forEach { $0.removeFromSuperlayer() }
        } else {
            container = CALayer()
            container.name = "ListMarkerContainer"
            // Make the container non-interactive and covering the full view.
            container.zPosition = 10
            hostLayer.addSublayer(container)
        }

        // Keep the container frame up-to-date with the text view bounds.
        container.frame = hostLayer.bounds

        guard !markers.isEmpty else { return }

        // Force a full layout pass so line-fragment rects reflect the current
        // attributed text (important right after attributedText has been set).
        layoutIfNeeded()
        layoutManager.ensureLayout(for: textContainer)

        let textLen = attributedText.length

        for marker in markers {
            let lineRect: CGRect

            if marker.characterIndex >= textLen {
                // Empty trailing paragraph — use the extra line fragment rect
                // which represents the line after a trailing newline.
                let extra = layoutManager.extraLineFragmentRect
                if extra != .zero {
                    lineRect = extra
                } else {
                    continue
                }
            } else {
                let glyphIndex = layoutManager.glyphIndexForCharacter(at: marker.characterIndex)
                lineRect = layoutManager.lineFragmentRect(forGlyphAt: glyphIndex, effectiveRange: nil)
            }

            // Build the marker layer.
            let markerStr = NSAttributedString(
                string: marker.text,
                attributes: [
                    .font: marker.font,
                    .foregroundColor: marker.color,
                ]
            )
            let markerSize = markerStr.size()

            // Right-align the marker just before the headIndent boundary,
            // with a small gap (4pt) between marker and content.
            // Coordinates are in text-container space (same as lineRect).
            let x = marker.headIndent - markerSize.width - 4
            let y = lineRect.origin.y

            let markerLayer = ListMarkerLayer(string: markerStr,
                                              frame: CGRect(x: x, y: y,
                                                            width: markerSize.width,
                                                            height: markerSize.height))
            print("[ListMarkers]   placed '\(marker.text)' at (\(x), \(y)) lineRect=\(lineRect)")
            container.addSublayer(markerLayer)
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

/// A text layer that renders a list marker (bullet or ordinal) in the gutter.
private final class ListMarkerLayer: CATextLayer {
    override init() {
        super.init()
    }

    init(string: NSAttributedString, frame: CGRect) {
        super.init()
        self.frame = frame
        self.string = string
        isWrapped = false
        contentsScale = UIScreen.main.scale
    }

    required init?(coder: NSCoder) {
        super.init(coder: coder)
    }
}
