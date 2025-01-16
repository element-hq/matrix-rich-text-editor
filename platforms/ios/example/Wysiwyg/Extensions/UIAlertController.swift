//
// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import UIKit

extension UIAlertController {
    convenience init(alert: AlertConfig) {
        self.init(title: alert.title, message: nil, preferredStyle: .alert)
        var numberOfTextFields = 0
        for action in alert.actions {
            switch action {
            case let .cancel(title):
                addAction(UIAlertAction(title: title, style: .cancel) { _ in
                    alert.dismissAction?()
                })
            case let .destructive(title, action):
                addAction(UIAlertAction(title: title, style: .destructive) { _ in
                    alert.dismissAction?()
                    action()
                })
            case let .textAction(title: title, textFieldsData, action):
                for textFieldData in textFieldsData {
                    addTextField()
                    guard let textFields = textFields else { return }
                    textFields[numberOfTextFields].placeholder = textFieldData.placeholder
                    textFields[numberOfTextFields].text = textFieldData.defaultValue
                    textFields[numberOfTextFields].accessibilityIdentifier = textFieldData.accessibilityIdentifier.rawValue
                    numberOfTextFields += 1
                }
                guard let textFields = textFields else { return }
                addAction(UIAlertAction(title: title, style: .default) { _ in
                    alert.dismissAction?()
                    var strings: [String] = []
                    strings = textFields.compactMap { $0.text ?? "" }
                    action(strings)
                })
            }
        }
    }
}
