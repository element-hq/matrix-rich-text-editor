//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

public extension PatternKey {
    /// Associated mention type, if any.
    var mentionType: WysiwygMentionType? {
        switch self {
        case .at:
            return .user
        case .hash:
            return .room
        case .slash, .custom:
            return nil
        }
    }
}
