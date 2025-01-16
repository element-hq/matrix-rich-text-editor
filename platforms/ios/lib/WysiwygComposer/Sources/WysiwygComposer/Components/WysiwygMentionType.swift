//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import Foundation

// MARK: - Public

/// Defines a mention type available in the Rich Text Editor.
public enum WysiwygMentionType: String {
    case user
    case room
}

public extension WysiwygMentionType {
    /// Associated pattern key.
    var patternKey: PatternKey {
        switch self {
        case .user:
            return .at
        case .room:
            return .hash
        }
    }
}

// MARK: - Internal

extension WysiwygMentionType {
    /// Default attributes.
    var attributes: [Attribute] {
        [
            Attribute(key: "data-mention-type", value: rawValue),
        ]
    }
}
