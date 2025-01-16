//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import OSLog
import UIKit

final class AppDelegate: NSObject, UIApplicationDelegate {
    func application(_ application: UIApplication,
                     didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]? = nil) -> Bool {
        Logger.wysywygLogLevel = .debug
        if #available(iOS 15.0, *) {
            NSTextAttachment.registerViewProviderClass(WysiwygAttachmentViewProvider.self,
                                                       forFileType: WysiwygAttachmentViewProvider.pillUTType)
        } else {
            // Fallback on earlier versions
        }
        return true
    }
}
