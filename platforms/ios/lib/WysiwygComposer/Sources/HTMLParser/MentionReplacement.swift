//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import Foundation

/// Represents a replacement in an instance of `NSAttributedString`
struct MentionReplacement {
    /// Range of the `NSAttributedString` where the replacement is located.
    let range: NSRange
    /// Data of the original content of the `NSAttributedString`.
    let content: MentionContent
}

// MARK: - Helpers

extension MentionReplacement {
    /// Computes the offset between the replacement and the original part (i.e. if the original length
    /// is greater than the replacement range, this offset will be negative).
    var offset: Int {
        range.length - content.rustLength
    }
}
