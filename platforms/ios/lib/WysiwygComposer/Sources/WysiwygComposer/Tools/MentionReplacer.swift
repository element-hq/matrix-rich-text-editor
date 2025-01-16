//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import Foundation
import HTMLParser

/// Extension protocol for HTMLParser's `MentionReplacer` that handles replacement for markdown.
public protocol MentionReplacer: HTMLMentionReplacer {
    /// Called when the composer switches to plain text mode or when
    /// the client sets an HTML body as the current content of the composer
    /// in plain text mode. Provides the ability for the client to replace
    /// e.g. markdown links with a pillified representation.
    ///
    /// - Parameter attributedString: An attributed string containing the parsed markdown.
    /// - Returns: An attributed string with replaced content.
    func postProcessMarkdown(in attributedString: NSAttributedString) -> NSAttributedString

    /// Called when the composer switches out of plain text mode.
    /// Provides the ability for the client to restore a markdown-valid content
    /// for items altered using `postProcessMarkdown`.
    ///
    /// - Parameter attributedString: An attributed string containing the current content of the text view.
    /// - Returns: A valid markdown string.
    func restoreMarkdown(in attributedString: NSAttributedString) -> String
}
