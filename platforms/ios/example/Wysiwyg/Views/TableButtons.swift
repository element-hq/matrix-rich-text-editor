//
// Copyright 2026 New Vector Ltd.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

import SwiftUI
import WysiwygComposer

/// A simple table editing toolbar for the example app: insert a 2x2 table, add/remove the
/// current row/column, toggle the current row as a header, or remove the table entirely.
struct TableButtons: View {
    @EnvironmentObject private var viewModel: WysiwygComposerViewModel

    var body: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: 4) {
                Button("Table") { viewModel.insertTable(rows: 2, columns: 2) }
                    .accessibilityIdentifier(.insertTableButton)
                Button("+Row") { viewModel.addTableRowAfter() }
                    .accessibilityIdentifier(.addTableRowButton)
                Button("-Row") { viewModel.removeTableRow() }
                    .accessibilityIdentifier(.removeTableRowButton)
                Button("+Col") { viewModel.addTableColumnAfter() }
                    .accessibilityIdentifier(.addTableColumnButton)
                Button("-Col") { viewModel.removeTableColumn() }
                    .accessibilityIdentifier(.removeTableColumnButton)
                Button("Header") { viewModel.toggleTableHeader() }
                    .accessibilityIdentifier(.toggleTableHeaderButton)
                Button("Remove") { viewModel.removeTable() }
                    .accessibilityIdentifier(.removeTableButton)
            }
        }
    }
}

struct TableButtons_Previews: PreviewProvider {
    static let viewModel = WysiwygComposerViewModel()
    static var previews: some View {
        TableButtons()
            .environmentObject(viewModel)
    }
}
