//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import Foundation

/// Describe an error occurring during HTML string build.
public enum BuildHtmlAttributedError: LocalizedError, Equatable {
    /// Encoding data from raw HTML input failed.
    case dataError(encoding: String.Encoding)

    public var errorDescription: String? {
        switch self {
        case let .dataError(encoding: encoding):
            return "Unable to encode string with: \(encoding.description) rawValue: \(encoding.rawValue)"
        }
    }
}
