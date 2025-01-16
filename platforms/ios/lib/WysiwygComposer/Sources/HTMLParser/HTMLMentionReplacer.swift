//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import Foundation

/// Defines an API for mention replacement with other objects (e.g. pills)
public protocol HTMLMentionReplacer {
    /// Called when the parser of the composer steps upon a mention.
    /// This can be used to provide custom attributed string parts, such
    /// as a pillified representation of a mention.
    /// If nothing is provided, the composer will use a standard link.
    ///
    /// - Parameters:
    ///   - url: URL of the mention's permalink
    ///   - text: Display text of the mention
    /// - Returns: Replacement for the mention.
    func replacementForMention(_ url: String, text: String) -> NSAttributedString?
}
