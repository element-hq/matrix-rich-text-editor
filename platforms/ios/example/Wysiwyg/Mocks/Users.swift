//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

enum Users: Identifiable, CaseIterable {
    case alice
    case bob
    case charlie

    var id: String {
        url
    }

    var iconSystemName: String {
        "person.circle"
    }

    var name: String {
        switch self {
        case .alice:
            return "Alice"
        case .bob:
            return "Bob"
        case .charlie:
            return "Charlie"
        }
    }

    var url: String {
        switch self {
        case .alice:
            return "https://matrix.to/#/@alice:matrix.org"
        case .bob:
            return "https://matrix.to/#/@bob:matrix.org"
        case .charlie:
            return "https://matrix.to/#/@charlie:matrix.org"
        }
    }

    var accessibilityIdentifier: WysiwygSharedAccessibilityIdentifier {
        switch self {
        case .alice:
            return .aliceButton
        case .bob:
            return .bobButton
        case .charlie:
            return .charlieButton
        }
    }

    static let title = "Users"

    static func filtered(with text: String) -> [Users] {
        allCases.filter { $0.name.lowercased().contains(text.lowercased()) }
    }
}
