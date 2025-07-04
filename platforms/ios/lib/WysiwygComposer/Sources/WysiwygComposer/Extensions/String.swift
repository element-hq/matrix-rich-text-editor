//
// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import Foundation

extension String {
    /// Returns length of the string in UTF16 code units.
    var utf16Length: Int {
        (self as NSString).length
    }
    
    /// Converts all whitespaces to NBSP to avoid diffs caused by HTML translations.
    var withNBSP: String {
        String(map { $0.isWhitespace ? Character.nbsp : $0 })
    }
    
    var containsLatinAndCommonCharactersOnly: Bool {
        range(of: "[^\\p{Latin}\\p{Common}]", options: .regularExpression) == nil
    }
}
