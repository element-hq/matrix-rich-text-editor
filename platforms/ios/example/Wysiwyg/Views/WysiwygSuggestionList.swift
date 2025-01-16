//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import SwiftUI
import WysiwygComposer

struct WysiwygSuggestionList: View {
    @EnvironmentObject private var viewModel: WysiwygComposerViewModel
    var suggestion: SuggestionPattern

    var body: some View {
        HStack {
            VStack(alignment: .leading) {
                switch suggestion.key {
                case .at:
                    let users = Users.filtered(with: suggestion.text)
                    if !users.isEmpty {
                        Text(Users.title).underline()
                        ForEach(users) { user in
                            Button {
                                viewModel.setMention(url: user.url, name: user.name, mentionType: .user)
                            } label: {
                                HStack(spacing: 4) {
                                    Image(systemName: user.iconSystemName)
                                    Text(user.name)
                                }
                            }
                            .accessibilityIdentifier(user.accessibilityIdentifier)
                        }
                    }
                case .hash:
                    let rooms = Rooms.filtered(with: suggestion.text)
                    if !rooms.isEmpty {
                        Text(Rooms.title).underline()
                        ForEach(rooms) { room in
                            Button {
                                viewModel.setMention(url: room.url, name: room.name, mentionType: .room)
                            } label: {
                                HStack(spacing: 4) {
                                    Image(systemName: room.iconSystemName)
                                    Text(room.name)
                                }
                            }
                            .accessibilityIdentifier(room.accessiblityIdentifier)
                        }
                    }
                case .slash:
                    let commands = Commands.filtered(with: suggestion.text)
                    if !commands.isEmpty {
                        Text(Commands.title).underline()
                        ForEach(Commands.allCases.filter { $0.name.contains("/" + suggestion.text.lowercased()) }) { command in
                            Button {
                                viewModel.setCommand(name: command.name)
                            } label: {
                                HStack(spacing: 4) {
                                    Image(systemName: command.iconSystemName)
                                    Text(command.name)
                                }
                            }
                            .accessibilityIdentifier(command.accessibilityIdentifier)
                        }
                    }
                case .custom:
                    EmptyView()
                }
            }
            .padding(.horizontal, 8)
            Spacer()
        }
        .overlay(Rectangle().stroke(Color.gray, lineWidth: 1))
        .padding(.horizontal, 12)
    }
}
