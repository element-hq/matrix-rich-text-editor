//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import Foundation
import UIKit

/// Provides flushing of views created by NSTextAttachmentViewProvider.
/// This is needed because of an issue with iOS not removing properly views
/// that are created by `NSTextAttachmentViewProvider`.
final class WysiwygPillsFlusher {
    private var pillViews = [UIView]()

    /// Register a view to be flushed on attributed text updates.
    /// Should be called when creating a view from NSTextAttachmentViewProvider.
    ///
    /// - Parameter pillView: View to register.
    func registerPillView(_ pillView: UIView) {
        pillViews.append(pillView)
    }

    /// Flush all the registered view, should be called before setting a new attributed string.
    func flush() {
        for view in pillViews {
            view.alpha = 0.0
            view.removeFromSuperview()
        }
        pillViews.removeAll()
    }
}
