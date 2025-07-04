//
// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import SwiftUI
import UIKit

public protocol WysiwygComposerViewModelProtocol: AnyObject {
    /// The textView that the model manages.
    var textView: WysiwygTextView { get set }

    /// Whether the current content of the composer is empty.
    var isContentEmpty: Bool { get }

    /// Update the composer compressed required height if it has changed.
    func updateCompressedHeightIfNeeded()

    /// Replace text in the model.
    ///
    /// - Parameters:
    ///   - range: Range to replace.
    ///   - replacementText: Replacement text to apply.
    /// - Returns: Whether the textView should continue with the insertion of the replacement text(within shouldChangeTextIn) or it should be left for a subquent model update to reflect the changes.
    func replaceText(range: NSRange, replacementText: String) -> Bool

    /// Select given range of text within the model.
    ///
    /// - Parameters:
    ///   - range: Range to select.
    func select(range: NSRange)

    /// Notify that the text view content has changed.
    func didUpdateText()

    /// Apply an enter/return key event.
    func enter()

    /// Get the ideal size for the composer's text view inside a SwiftUI context.
    ///
    /// - Parameter proposal: Proposed view size.
    /// - Returns: Ideal size for current context.
    @available(iOS 16.0, *)
    func getIdealSize(_ proposal: ProposedViewSize) -> CGSize
}
