//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import UIKit

extension UITextView {
    /// Toggles  autocorrection if needed. It should always be enabled,
    /// unless current text starts with exactly one slash.
    func toggleAutocorrectionIfNeeded() {
        let newValue: UITextAutocorrectionType = attributedText.string.prefix(while: { $0 == .slash }).count == 1 ? .no : .yes
        if newValue != autocorrectionType {
            autocorrectionType = newValue
            reloadInputViews()
        }
    }
}
