//
// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import Foundation

typealias UTF16Removals = [NSRange]
typealias UTF16Insertions = [(range: NSRange, text: String)]

extension CollectionDifference<Character> {
    /// Transforms the removals in the `CollectionDifference` into an array of ranges.
    /// Output ranges are defined with UTF16 locations and lengths.
    ///
    /// - Parameter string: String in which the characters have been removed.
    /// - Returns: Array of removed ranges.
    func utf16Removals(in string: String) -> UTF16Removals {
        removals.reduce([]) { partialResult, change in
            var partialRes = partialResult
            switch change {
            case .remove(offset: let offset, element: let element, associatedWith: _):
                let index = string.index(string.startIndex, offsetBy: offset)
                let utf16Offset = string[..<index].utf16.count
                guard let lastRange = partialRes.popLast() else {
                    return [NSRange(location: utf16Offset, length: element.utf16.count)]
                }
                if lastRange.upperBound == utf16Offset {
                    partialRes.append(NSRange(location: lastRange.location, length: lastRange.length + element.utf16.count))
                } else {
                    partialRes.append(lastRange)
                    partialRes.append(NSRange(location: utf16Offset, length: element.utf16.count))
                }
                return partialRes
            default:
                return []
            }
        }
    }
    
    /// Transforms the insertions in the `CollectionDifference` into an array of ranges and associated text.
    /// Output ranges are defined with UTF16 locations and lengths.
    ///
    /// - Parameter string: String in which the characters have been inserted.
    /// - Returns: Array of inserted ranges and text.
    func utf16Insertions(in string: String) -> UTF16Insertions {
        insertions.reduce([]) { partialResult, change in
            var partialRes = partialResult
            switch change {
            case .insert(offset: let offset, element: let element, associatedWith: _):
                let index = string.index(string.startIndex, offsetBy: offset)
                let utf16Offset = string[..<index].utf16.count
                guard let lastRange = partialRes.popLast() else {
                    return [(NSRange(location: utf16Offset, length: element.utf16.count), String(element))]
                }
                let (range, text) = lastRange
                if range.upperBound == utf16Offset {
                    partialRes.append(
                        (NSRange(location: range.location, length: range.length + element.utf16.count), text + String(element))
                    )
                } else {
                    partialRes.append(lastRange)
                    partialRes.append((NSRange(location: utf16Offset, length: element.utf16.count), String(element)))
                }
                return partialRes
            default:
                return []
            }
        }
    }
}
