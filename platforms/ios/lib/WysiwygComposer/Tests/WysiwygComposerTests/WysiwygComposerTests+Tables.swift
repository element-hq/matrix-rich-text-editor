//
// Copyright 2026 New Vector Ltd.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import Testing
@testable import WysiwygComposer

extension WysiwygComposerTests {
    @Test func insertTable() {
        ComposerModelWrapper()
            .action { $0.insertTable(rows: 2, columns: 2) }
            .assertHtml(
                "<table><tbody><tr><td></td><td></td></tr><tr><td></td><td></td></tr></tbody></table>"
            )
    }

    @Test func addAndRemoveTableRow() {
        ComposerModelWrapper()
            .action { $0.insertTable(rows: 1, columns: 2) }
            .action { $0.addTableRowAfter() }
            .assertHtml(
                "<table><tbody><tr><td></td><td></td></tr><tr><td></td><td></td></tr></tbody></table>"
            )
            .action { $0.removeTableRow() }
            .assertHtml("<table><tbody><tr><td></td><td></td></tr></tbody></table>")
    }

    @Test func addAndRemoveTableColumn() {
        ComposerModelWrapper()
            .action { $0.insertTable(rows: 2, columns: 1) }
            .action { $0.addTableColumnAfter() }
            .assertHtml(
                "<table><tbody><tr><td></td><td></td></tr><tr><td></td><td></td></tr></tbody></table>"
            )
            .action { $0.removeTableColumn() }
            .assertHtml("<table><tbody><tr><td></td></tr><tr><td></td></tr></tbody></table>")
    }

    @Test func toggleTableHeader() {
        ComposerModelWrapper()
            .action { $0.insertTable(rows: 1, columns: 2) }
            .action { $0.toggleTableHeader() }
            .assertHtml("<table><thead><tr><th></th><th></th></tr></thead></table>")
            .action { $0.toggleTableHeader() }
            .assertHtml("<table><tbody><tr><td></td><td></td></tr></tbody></table>")
    }

    @Test func removeTable() {
        ComposerModelWrapper()
            .action { $0.insertTable(rows: 1, columns: 1) }
            .action { $0.removeTable() }
            .assertHtml("")
    }

    @Test func removingLastRowRemovesTable() {
        ComposerModelWrapper()
            .action { $0.insertTable(rows: 1, columns: 1) }
            .action { $0.removeTableRow() }
            .assertHtml("")
    }

    @Test func removingLastColumnRemovesTable() {
        ComposerModelWrapper()
            .action { $0.insertTable(rows: 1, columns: 1) }
            .action { $0.removeTableColumn() }
            .assertHtml("")
    }
}
