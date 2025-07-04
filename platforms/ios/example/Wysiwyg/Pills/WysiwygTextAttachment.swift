//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import UIKit

/// Text attachment for pills display.
@available(iOS 15.0, *)
@objcMembers
class WysiwygTextAttachment: NSTextAttachment {
    // MARK: - Properties

    /// Return `WysiwygTextAttachmentData` contained in the text attachment.
    var data: WysiwygTextAttachmentData? {
        get {
            guard let contents = contents else { return nil }
            return try? Self.serializationService.deserialize(contents)
        }
        set {
            guard let newValue = newValue else {
                contents = nil
                return
            }
            contents = try? Self.serializationService.serialize(newValue)
            updateBounds()
        }
    }

    private static let serializationService = SerializationService()

    // MARK: - Init

    override init(data contentData: Data?, ofType uti: String?) {
        super.init(data: contentData, ofType: uti)

        updateBounds()
    }

    /// Create a Mention Pill text attachment for given display name.
    ///
    /// - Parameters:
    ///   - displayName: the display name for the pill
    ///   - url: The absolute URL for the item.
    ///   - font: the text font
    convenience init?(displayName: String,
                      url: String,
                      font: UIFont) {
        let data = WysiwygTextAttachmentData(displayName: displayName,
                                             url: url,
                                             font: font)

        guard let encodedData = try? Self.serializationService.serialize(data) else {
            return nil
        }
        self.init(data: encodedData, ofType: WysiwygAttachmentViewProvider.pillUTType)
    }

    required init?(coder: NSCoder) {
        super.init(coder: coder)

        updateBounds()
    }
}

// MARK: - Private

@available(iOS 15.0, *)
private extension WysiwygTextAttachment {
    func updateBounds() {
        guard let data = data else { return }
        let pillSize = WysiwygAttachmentViewProvider.size(forDisplayText: data.displayName, andFont: data.font)
        // Offset to align pill centerY with text centerY.
        let offset = data.font.descender + (data.font.lineHeight - pillSize.height) / 2.0
        bounds = CGRect(origin: CGPoint(x: 0.0, y: offset), size: pillSize)
    }
}
