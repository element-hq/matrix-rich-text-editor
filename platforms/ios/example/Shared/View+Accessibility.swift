//
// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import SwiftUI
import UIKit

/// Defines accessibility identifiers shared between the UIKit and the SwiftUI example apps.
public enum WysiwygSharedAccessibilityIdentifier: String {
    // Composer text view needs to be in sync with the value set for the text view in the library
    // Unfortunately trying to expose it from the lib results in undefined symbols error in a UI Test context.
    case composerTextView = "WysiwygComposer"
    case boldButton = "WysiwygBoldButton"
    case italicButton = "WysiwygItalicButton"
    case strikeThroughButton = "WysiwygStrikeThroughButton"
    case underlineButton = "WysiwygUnderlineButton"
    case inlineCodeButton = "WysiwygInlineCodeButton"
    case linkButton = "WysiwygLinkButton"
    case undoButton = "WysiwygUndoButton"
    case redoButton = "WysiwygRedoButton"
    case orderedListButton = "WysiwygOrderedListButton"
    case unorderedListButton = "WysiwygUnorderedListButton"
    case indentButton = "WysiwygIndentButton"
    case unindentButton = "WysiwygUnindentButton"
    case codeBlockButton = "WysiwygCodeBlockButton"
    case quoteButton = "WysiwygQuoteButton"
    case sendButton = "WysiwygSendButton"
    case minMaxButton = "WysiwygMinMaxButton"
    case plainRichButton = "WysiwygPlainRichButton"
    case contentText = "WysiwygContentText"
    case htmlContentText = "WysiwygHtmlContentText"
    case linkUrlTextField = "WysiwygLinkUrlTextField"
    case linkTextTextField = "WysiwygLinkTextTextField"
    case showTreeButton = "WysiwygShowTreeButton"
    case treeText = "WysiwygTreeText"
    case forceCrashButton = "WysiwygForceCrashButton"
    case setHtmlButton = "WysiwygSetHtmlButton"
    case setHtmlField = "WysiwygSetHtmlField"
    case autocorrectionIndicator = "WysiwygAutocorrectionIndicator"
    case toggleFocusButton = "WysiwygToggleFocusButton"

    // Mock buttons for menu
    case aliceButton = "WysiwygMenuAliceButton"
    case bobButton = "WysiwygMenuBobButton"
    case charlieButton = "WysiwygMenuCharlieButton"
    case room1Button = "WysiwygMenuRoom1Button"
    case room2Button = "WysiwygMenuRoom2Button"
    case room3Button = "WysiwygMenuRoom3Button"
    case joinCommandButton = "WysiwygMenuJoinButton"
    case inviteCommandButton = "WysiwygMenuInviteButton"
    case meCommandButton = "WysiwygMenuMeButton"
}

public extension View {
    /// Sets up an accessibility identifier to the view from the enum
    /// of expected accessibilityIdentifiers.
    ///
    /// - Parameters:
    ///   - identifier: the accessibility identifier to setup
    /// - Returns: modified view
    func accessibilityIdentifier(_ identifier: WysiwygSharedAccessibilityIdentifier)
        -> ModifiedContent<Self, AccessibilityAttachmentModifier> {
        accessibilityIdentifier(identifier.rawValue)
    }
}
