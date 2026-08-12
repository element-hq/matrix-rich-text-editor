/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.wysiwyg.compose.ui.components

import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.rememberScrollState
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp

/**
 * A simple table editing toolbar for the example app: insert a 2x2 table, add/remove the
 * current row/column, toggle the current row as a header, or remove the table entirely.
 */
@Composable
fun TableButtons(
    modifier: Modifier = Modifier,
    onInsertTable: () -> Unit = {},
    onAddRow: () -> Unit = {},
    onRemoveRow: () -> Unit = {},
    onAddColumn: () -> Unit = {},
    onRemoveColumn: () -> Unit = {},
    onToggleHeader: () -> Unit = {},
    onRemoveTable: () -> Unit = {},
) {
    val scrollState = rememberScrollState()
    Row(
        modifier = modifier.horizontalScroll(scrollState),
        horizontalArrangement = Arrangement.spacedBy(4.dp),
    ) {
        TextButton(onClick = onInsertTable) { Text("Table") }
        TextButton(onClick = onAddRow) { Text("+Row") }
        TextButton(onClick = onRemoveRow) { Text("-Row") }
        TextButton(onClick = onAddColumn) { Text("+Col") }
        TextButton(onClick = onRemoveColumn) { Text("-Col") }
        TextButton(onClick = onToggleHeader) { Text("Header") }
        TextButton(onClick = onRemoveTable) { Text("Remove") }
    }
}

@Preview
@Composable
private fun TableButtonsPreview() = TableButtons()
