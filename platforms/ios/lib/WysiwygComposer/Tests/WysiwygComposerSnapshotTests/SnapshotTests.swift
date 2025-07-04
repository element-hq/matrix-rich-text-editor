//
// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import SnapshotTesting
import SwiftUI
@testable import WysiwygComposer
import XCTest

class SnapshotTests: XCTestCase {
    let isRecord = false
    
    var viewModel = WysiwygComposerViewModel()
    var hostingController: UIViewController!
    
    override func setUpWithError() throws {
        try super.setUpWithError()
        let composerView = WysiwygComposerView(placeholder: "Placeholder",
                                               viewModel: viewModel,
                                               itemProviderHelper: nil,
                                               keyCommands: nil,
                                               pasteHandler: nil)
        hostingController = UIHostingController(rootView: VStack {
            // Set the composer's text view at the top of the controller.
            composerView
            Spacer()
        })
    }
    
    override func tearDownWithError() throws {
        try super.tearDownWithError()
        viewModel.clearContent()
        hostingController = nil
    }
}
