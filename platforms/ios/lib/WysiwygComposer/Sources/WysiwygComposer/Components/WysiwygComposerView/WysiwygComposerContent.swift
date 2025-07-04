//
// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import Foundation

/// Defines message content displayed in the composer.
@objcMembers
public class WysiwygComposerContent: NSObject {
    // MARK: - Public

    /// Markdown representation of the displayed text.
    public let markdown: String
    /// HTML representation of the displayed text.
    public let html: String

    // MARK: - Internal

    /// Init.
    ///
    /// - Parameters:
    ///   - markdown: Markdown representation of the displayed text.
    ///   - html: HTML representation of the displayed text.
    init(markdown: String = "",
         html: String = "") {
        self.markdown = markdown
        self.html = html
    }
}

public struct WysiwygComposerAttributedContent {
    /// Attributed string representation of the displayed text.
    public let text: NSAttributedString
    /// Range of the selected text within the attributed representation.
    public var selection: NSRange
    /// Plain text variant of the content saved for recovery.
    public let plainText: String

    // MARK: - Internal

    /// Init.
    ///
    /// - Parameters:
    ///   - text: Attributed string representation of the displayed text.
    ///   - selection: Range of the selected text within the attributed representation.
    ///   - plainText: Plain text variant of the content saved for recovery.
    init(text: NSAttributedString = .init(string: ""),
         selection: NSRange = .zero,
         plainText: String = "") {
        self.text = text
        self.selection = selection
        self.plainText = plainText
    }
}
