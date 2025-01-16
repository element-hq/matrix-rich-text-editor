//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import UIKit

/// An class that describes key commands that can be handled by the hosting application wth their associated action
public struct WysiwygKeyCommand {
    /// A default initialiser for the enter command which is most commonly used
    public static func enter(action: @escaping () -> Void) -> WysiwygKeyCommand {
        WysiwygKeyCommand(input: "\r", modifierFlags: [], action: action)
    }
    
    let input: String
    let modifierFlags: UIKeyModifierFlags
    let action: () -> Void
}
