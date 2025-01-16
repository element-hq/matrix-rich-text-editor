//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

enum Rooms: Identifiable, CaseIterable {
    case room1
    case room2
    case room3

    var id: String {
        url
    }

    var iconSystemName: String {
        "character.bubble"
    }

    var name: String {
        switch self {
        case .room1:
            return "Room 1"
        case .room2:
            return "Room 2"
        case .room3:
            return "Room 3"
        }
    }

    var url: String {
        switch self {
        case .room1:
            return "https://matrix.to/#/#room1:matrix.org"
        case .room2:
            return "https://matrix.to/#/#room2:matrix.org"
        case .room3:
            return "https://matrix.to/#/#room3:matrix.org"
        }
    }

    var accessiblityIdentifier: WysiwygSharedAccessibilityIdentifier {
        switch self {
        case .room1:
            return .room1Button
        case .room2:
            return .room2Button
        case .room3:
            return .room3Button
        }
    }

    static let title = "Rooms"

    static func filtered(with text: String) -> [Rooms] {
        allCases.filter { $0.name.lowercased().contains(text.lowercased()) }
    }
}
