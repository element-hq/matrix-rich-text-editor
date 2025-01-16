//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import Foundation
import UIKit

/// Describe style for the HTML parser.
public struct HTMLParserStyle {
    // MARK: - Public

    public static let standard = HTMLParserStyle(
        textColor: UIColor.label,
        linkColor: UIColor.link,
        codeBlockStyle: BlockStyle(backgroundColor: UIColor(red: 244 / 255, green: 246 / 255, blue: 250 / 255, alpha: 1.0),
                                   borderColor: UIColor(red: 227 / 255, green: 232 / 255, blue: 240 / 255, alpha: 1.0),
                                   borderWidth: 1.0,
                                   cornerRadius: 4.0,
                                   padding: BlockStyle.Padding(horizontal: 10, vertical: 12),
                                   type: .background),
        quoteBlockStyle: BlockStyle(backgroundColor: UIColor(red: 244 / 255, green: 246 / 255, blue: 250 / 255, alpha: 1.0),
                                    borderColor: UIColor(red: 227 / 255, green: 232 / 255, blue: 240 / 255, alpha: 1.0),
                                    borderWidth: 0,
                                    cornerRadius: 0,
                                    padding: BlockStyle.Padding(horizontal: 25, vertical: 12),
                                    type: .side(offset: 5, width: 4))
    )

    /// Color for standard text.
    public var textColor: UIColor
    /// Color for link text.
    public var linkColor: UIColor
    /// Code Block Style.
    public var codeBlockStyle: BlockStyle
    /// Quote Block Style.
    public var quoteBlockStyle: BlockStyle

    /// Init.
    ///
    /// - Parameters:
    ///   - textColor: Color for standard text.
    ///   - linkColor: Color for link text.
    ///   - codeBlockStyle: Code Block Style.
    ///   - quoteBlockStyle: Quote Block Style.
    public init(textColor: UIColor,
                linkColor: UIColor,
                codeBlockStyle: BlockStyle,
                quoteBlockStyle: BlockStyle) {
        self.textColor = textColor
        self.linkColor = linkColor
        self.codeBlockStyle = codeBlockStyle
        self.quoteBlockStyle = quoteBlockStyle
    }
}
