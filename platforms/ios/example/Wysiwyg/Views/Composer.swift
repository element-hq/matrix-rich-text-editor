//
// Copyright 2022-2024 New Vector Ltd.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import SwiftUI
import WysiwygComposer

/// Example SwiftUI that embeds the WysiwygComposerView in a bordered container
/// that grows to a max height.
struct Composer: View {
    @ObservedObject var viewModel: WysiwygComposerViewModel
    let itemProviderHelper: WysiwygItemProviderHelper?
    let keyCommands: [WysiwygKeyCommand]?
    let pasteHandler: PasteHandler?
    let minTextViewHeight: CGFloat = 20
    let borderHeight: CGFloat = 40
    @FocusState var focused: Bool
    var verticalPadding: CGFloat {
        (borderHeight - minTextViewHeight) / 2
    }
    
    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            let rect = RoundedRectangle(cornerRadius: borderHeight / 2)
            HStack {
                WysiwygComposerView(
                    placeholder: "Placeholder",
                    viewModel: viewModel,
                    itemProviderHelper: itemProviderHelper,
                    keyCommands: keyCommands,
                    pasteHandler: pasteHandler
                )
                .frame(height: viewModel.idealHeight)
                .padding(.horizontal, 12)
                .onAppear {
                    viewModel.setup()
                }
                .focused($focused, equals: true)
            }
            .padding(.vertical, verticalPadding)
            .clipShape(rect)
            .overlay(rect.stroke(Color.gray, lineWidth: 1))
            .padding(.horizontal, 12)
            .onTapGesture {
                focused = true
            }
            if let suggestion = viewModel.suggestionPattern {
                WysiwygSuggestionList(suggestion: suggestion)
                    .environmentObject(viewModel)
            }
            if !viewModel.plainTextMode {
                ComposerActionToolbar { action in
                    viewModel.apply(action)
                }
                .environmentObject(viewModel)
                .padding(.horizontal, 16)
            }
        }
    }
}

struct Composer_Previews: PreviewProvider {
    static let viewModel = WysiwygComposerViewModel()
    static var previews: some View {
        Composer(viewModel: viewModel,
                 itemProviderHelper: nil,
                 keyCommands: nil,
                 pasteHandler: nil)
    }
}
