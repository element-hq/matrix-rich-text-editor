//
// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import SwiftUI
import WysiwygComposer

struct ComposerActionToolbar: View {
    @EnvironmentObject private var viewModel: WysiwygComposerViewModel
    var toolbarAction: (ComposerAction) -> Void
    @State private var isShowingUrlAlert = false
    @State private var linkAttributedRange = NSRange.zero
    @State private var linkAction: LinkAction?
    
    var body: some View {
        HStack {
            ForEach(ComposerAction.allCases.filter { $0.isVisible(viewModel) }) { action in
                Button {
                    if action == .link {
                        linkAttributedRange = viewModel.attributedContent.selection
                        linkAction = viewModel.getLinkAction()
                        isShowingUrlAlert = true
                    } else {
                        toolbarAction(action)
                    }
                } label: {
                    Image(systemName: action.iconName)
                        .renderingMode(.template)
                        .foregroundColor(action.color(viewModel))
                }
                .disabled(action.isDisabled(viewModel))
                .buttonStyle(.automatic)
                .accessibilityIdentifier(action.accessibilityIdentifier)
            }
        }
        .alert(isPresented: $isShowingUrlAlert, makeAlertConfig())
        .frame(width: nil, height: 50, alignment: .center)
    }
    
    func makeAlertConfig() -> AlertConfig {
        var actions: [AlertConfig.Action] = [.cancel(title: "Cancel")]
        let createLinkTitle = "Create Link"
        let singleTextAction: ([String]) -> Void = { strings in
            let urlString = strings[0]
            viewModel.select(range: linkAttributedRange)
            viewModel.applyLinkOperation(.setLink(urlString: urlString))
        }
        switch linkAction {
        case .create:
            actions.append(createAction(singleTextAction: singleTextAction))
            return AlertConfig(title: createLinkTitle, actions: actions)
        case .createWithText:
            let doubleTextAction: ([String]) -> Void = { strings in
                let urlString = strings[0]
                let text = strings[1]
                viewModel.select(range: linkAttributedRange)
                viewModel.applyLinkOperation(.createLink(urlString: urlString, text: text))
            }
            actions.append(createWithTextAction(doubleTextAction: doubleTextAction))
            return AlertConfig(title: createLinkTitle, actions: actions)
        case let .edit(url):
            let editLinktitle = "Edit Link URL"
            actions.append(editTextAction(singleTextAction: singleTextAction, url: url))
            let removeAction = {
                viewModel.select(range: linkAttributedRange)
                viewModel.applyLinkOperation(.removeLinks)
            }
            actions.append(.destructive(title: "Remove", action: removeAction))
            return AlertConfig(title: editLinktitle, actions: actions)
        case .disabled, .none:
            return AlertConfig(title: "", actions: actions)
        }
    }
}

private extension ComposerActionToolbar {
    private func createAction(singleTextAction: @escaping ([String]) -> Void) -> AlertConfig.Action {
        .textAction(
            title: "Ok",
            textFieldsData: [
                .init(
                    accessibilityIdentifier: .linkUrlTextField,
                    placeholder: "URL",
                    defaultValue: nil
                ),
            ],
            action: singleTextAction
        )
    }

    private func createWithTextAction(doubleTextAction: @escaping ([String]) -> Void) -> AlertConfig.Action {
        .textAction(
            title: "Ok",
            textFieldsData: [
                .init(
                    accessibilityIdentifier: .linkUrlTextField,
                    placeholder: "URL",
                    defaultValue: nil
                ),
                .init(
                    accessibilityIdentifier: .linkTextTextField,
                    placeholder: "Text",
                    defaultValue: nil
                ),
            ],
            action: doubleTextAction
        )
    }

    private func editTextAction(singleTextAction: @escaping ([String]) -> Void, url: String) -> AlertConfig.Action {
        .textAction(
            title: "Ok",
            textFieldsData: [
                .init(
                    accessibilityIdentifier: .linkUrlTextField,
                    placeholder: "URL",
                    defaultValue: url
                ),
            ],
            action: singleTextAction
        )
    }
}
