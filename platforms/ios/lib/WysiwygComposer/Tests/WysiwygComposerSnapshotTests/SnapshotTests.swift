//
// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import SwiftUI
import UIKit
@testable import WysiwygComposer

/// Shared scaffolding for the snapshot test suites.
///
/// Swift Testing suites are structs (no inheritance), so the common composer setup that the
/// XCTest base class used to provide is built here and composed into each suite via `init()`.
@MainActor
enum SnapshotScene {
    /// Whether snapshots should be (re)recorded rather than compared.
    static let isRecord = false

    /// Builds a fresh view model hosted in a controller, mirroring the previous `setUpWithError`.
    static func make() -> (viewModel: WysiwygComposerViewModel, hostingController: UIViewController) {
        let viewModel = WysiwygComposerViewModel()
        let composerView = WysiwygComposerView(placeholder: "Placeholder",
                                               viewModel: viewModel,
                                               itemProviderHelper: nil,
                                               keyCommands: nil,
                                               pasteHandler: nil)
        let hostingController = UIHostingController(rootView: VStack {
            // Set the composer's text view at the top of the controller.
            composerView
            Spacer()
        })
        return (viewModel, hostingController)
    }
}
