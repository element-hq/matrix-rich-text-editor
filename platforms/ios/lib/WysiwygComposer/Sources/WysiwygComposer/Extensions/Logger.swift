//
// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import OSLog
import UIKit

// MARK: - Logger

extension Logger {
    // MARK: Internal

    /// Describes a log level for the library.
    public enum LogLevel: Int {
        /// Every log is reported
        case debug = 0
        /// Warning and errors are reported
        case warning
        /// Only errors are reported
        case error
        /// No logs are reported
        case none
    }

    /// Current log level reported to OSLog. Default: only errors are reported.
    public static var wysywygLogLevel: LogLevel = .error

    static var subsystem = "org.matrix.WysiwygComposer"

    /// Creates a customized log for debug.
    ///
    /// - Parameters:
    ///   - elements: Elements to log.
    ///   - functionName: Name from the function where it is called.
    func logDebug(_ elements: [String], functionName: String) {
        guard Logger.wysywygLogLevel == .debug else { return }
        debug("\(customLog(elements, functionName: functionName))")
    }

    /// Creates a customized error log.
    ///
    /// - Parameters:
    ///   - elements: Elements to log.
    ///   - functionName: Name from the function where it is called.
    func logError(_ elements: [String], functionName: String) {
        guard Logger.wysywygLogLevel.rawValue <= LogLevel.error.rawValue else { return }
        error("\(customLog(elements, functionName: functionName))")
    }

    /// Creates a customized warning log.
    ///
    /// - Parameters:
    ///   - elements: Elements to log.
    ///   - functionName: Name from the function where it is called.
    func logWarning(_ elements: [String], functionName: String) {
        guard Logger.wysywygLogLevel.rawValue <= LogLevel.warning.rawValue else { return }
        warning("\(customLog(elements, functionName: functionName))")
    }

    // MARK: Private

    private func customLog(_ elements: [String], functionName: String) -> String {
        var logMessage = elements.map { $0 + " | " }.joined()
        logMessage.append(contentsOf: functionName)
        return logMessage
    }
}

// MARK: - UITextView + Logger

extension UITextView {
    /// Returns a log ready description of the current selection.
    var logSelection: String {
        "Sel(att): \(selectedRange)"
    }

    /// Returns a log ready description of the current text..
    var logText: String {
        "Text: \"\(text ?? "")\""
    }
}

// MARK: - WysiwygComposerAttributedContent + Logger

extension WysiwygComposerAttributedContent {
    /// Returns a log ready description of the attributed selection.
    var logSelection: String {
        "Sel(att): \(selection)"
    }

    /// Returns a log ready description of the markdown text.
    var logText: String {
        "Text: \"\(text)\""
    }
}
