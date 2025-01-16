//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

enum Commands: Identifiable, CaseIterable {
    case join
    case invite
    case me

    var id: String {
        name
    }

    var iconSystemName: String {
        "terminal"
    }

    var name: String {
        switch self {
        case .join:
            return "/join"
        case .invite:
            return "/invite"
        case .me:
            return "/me"
        }
    }

    var accessibilityIdentifier: WysiwygSharedAccessibilityIdentifier {
        switch self {
        case .join:
            return .joinCommandButton
        case .invite:
            return .inviteCommandButton
        case .me:
            return .meCommandButton
        }
    }

    static let title = "Commands"

    static func filtered(with text: String) -> [Commands] {
        allCases.filter { $0.name.lowercased().contains("/" + text.lowercased()) }
    }
}
