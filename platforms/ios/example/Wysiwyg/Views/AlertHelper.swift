//
// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import SwiftUI
import UIKit

struct AlertHelper<Content: View>: UIViewControllerRepresentable {
    @Binding var isPresented: Bool
    let alert: AlertConfig
    let content: Content

    func makeUIViewController(context _: UIViewControllerRepresentableContext<AlertHelper>) -> UIHostingController<Content> {
        UIHostingController(rootView: content)
    }

    final class Coordinator {
        var alertController: UIAlertController?
        init(_ controller: UIAlertController? = nil) {
            alertController = controller
        }
    }

    func makeCoordinator() -> Coordinator {
        Coordinator()
    }

    func updateUIViewController(_ uiViewController: UIHostingController<Content>,
                                context: UIViewControllerRepresentableContext<AlertHelper>) {
        uiViewController.rootView = content
        var alert = alert
        alert.dismissAction = {
            isPresented = false
        }
        if isPresented, uiViewController.presentedViewController == nil {
            context.coordinator.alertController = UIAlertController(alert: alert)
            guard let controller = context.coordinator.alertController else { return }
            uiViewController.present(controller, animated: true)
        }
        if !isPresented, uiViewController.presentedViewController == context.coordinator.alertController {
            uiViewController.dismiss(animated: true)
        }
    }
}

public struct AlertConfig {
    public struct TextFieldData {
        let accessibilityIdentifier: WysiwygSharedAccessibilityIdentifier
        let placeholder: String
        let defaultValue: String?
    }
    
    public enum Action {
        case cancel(title: String)
        case destructive(title: String, action: () -> Void)
        case textAction(title: String, textFieldsData: [TextFieldData], action: ([String]) -> Void)
    }
    
    public var title: String
    public var actions: [Action]
    public var dismissAction: (() -> Void)?
}
