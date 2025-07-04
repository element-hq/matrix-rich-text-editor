//
// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import SwiftUI
import WysiwygComposer

extension ComposerAction: CaseIterable, Identifiable {
    public static var allCases: [ComposerAction] = [
        .bold, .italic, .strikeThrough, .underline, .inlineCode,
        .link, .undo, .redo, .orderedList, .unorderedList, .indent, .unindent, .codeBlock, .quote,
    ]

    public var id: String {
        accessibilityIdentifier.rawValue
    }

    /// Compute color for action button.
    ///
    /// - Parameter viewModel: Composer's view model.
    /// - Returns: Tint color that the button should use.
    public func color(_ viewModel: WysiwygComposerViewModel) -> Color {
        switch viewModel.actionStates[self] {
        case .enabled:
            return Color.primary
        case .reversed:
            return Color.accentColor
        default:
            return Color.primary.opacity(0.3)
        }
    }

    /// Compute disabled status for action.
    ///
    /// - Parameter viewModel: Composer's view model.
    /// - Returns: True if the action is disabled, false otherwise.
    public func isDisabled(_ viewModel: WysiwygComposerViewModel) -> Bool {
        viewModel.actionStates[self] == ActionState.disabled
    }

    /// Compute visibility status for action.
    ///
    /// - Parameter viewModel: Composer's view model.
    /// - Returns: True if the action is visible, false otherwise.
    public func isVisible(_ viewModel: WysiwygComposerViewModel) -> Bool {
        switch self {
        case .indent, .unindent:
            return viewModel.isInList
        default:
            return true
        }
    }

    var accessibilityIdentifier: WysiwygSharedAccessibilityIdentifier {
        switch self {
        case .bold:
            return .boldButton
        case .italic:
            return .italicButton
        case .strikeThrough:
            return .strikeThroughButton
        case .underline:
            return .underlineButton
        case .inlineCode:
            return .inlineCodeButton
        case .link:
            return .linkButton
        case .undo:
            return .undoButton
        case .redo:
            return .redoButton
        case .orderedList:
            return .orderedListButton
        case .unorderedList:
            return .unorderedListButton
        case .indent:
            return .indentButton
        case .unindent:
            return .unindentButton
        case .codeBlock:
            return .codeBlockButton
        case .quote:
            return .quoteButton
        }
    }

    /// Returns the name of the system icon that should be used for button display.
    var iconName: String {
        switch self {
        case .bold:
            return "bold"
        case .italic:
            return "italic"
        case .strikeThrough:
            return "strikethrough"
        case .underline:
            return "underline"
        case .inlineCode:
            return "chevron.left.forwardslash.chevron.right"
        case .link:
            return "link"
        case .undo:
            return "arrow.uturn.backward"
        case .redo:
            return "arrow.uturn.forward"
        case .orderedList:
            return "list.number"
        case .unorderedList:
            return "list.bullet"
        case .indent:
            return "increase.indent"
        case .unindent:
            return "decrease.indent"
        case .codeBlock:
            return "note.text"
        case .quote:
            return "text.quote"
        }
    }
}

private extension WysiwygComposerViewModel {
    /// Returns true if we are currently inside a list.
    var isInList: Bool {
        actionStates[.orderedList] == .reversed || actionStates[.unorderedList] == .reversed
    }
}
