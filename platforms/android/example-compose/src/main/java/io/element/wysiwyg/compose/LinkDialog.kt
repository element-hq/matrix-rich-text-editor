/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.wysiwyg.compose

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import io.element.android.wysiwyg.view.models.LinkAction

@Composable
fun LinkDialog(
    linkAction: LinkAction,
    onRemoveLink: () -> Unit,
    onSetLink: (url: String) -> Unit,
    onInsertLink: (url: String, text: String) -> Unit,
    onDismissRequest: () -> Unit,
) {
    val currentUrl = (linkAction as? LinkAction.SetLink)?.currentUrl

    var newText by remember { mutableStateOf("") }
    var newLink by remember { mutableStateOf(currentUrl ?: "") }

    Dialog(onDismissRequest = onDismissRequest) {
        Surface(
            color = MaterialTheme.colorScheme.surface,
            shape = RoundedCornerShape(8.dp),
        ) {
            Column(
                modifier = Modifier.padding(16.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                if (linkAction is LinkAction.InsertLink) {
                    OutlinedTextField(
                        value = newText, onValueChange = { newText = it }, placeholder = {
                            Text(text = stringResource(R.string.link_text))
                        })
                }
                OutlinedTextField(value = newLink, onValueChange = { newLink = it }, placeholder = {
                    Text("Link")
                })
                Row(
                    modifier = Modifier.align(Alignment.End),
                ) {
                    if (currentUrl != null) {
                        TextButton(
                            onClick = {
                                onRemoveLink()
                                onDismissRequest()
                            }) {
                            Text(stringResource(id = R.string.link_remove))
                        }
                    }
                    TextButton(
                        onClick = {
                            when (linkAction) {
                                LinkAction.InsertLink -> onInsertLink(newLink, newText)
                                is LinkAction.SetLink -> onSetLink(newLink)
                            }
                            onDismissRequest()
                        }) {
                        Text(
                            stringResource(
                                id =
                                when (linkAction) {
                                    LinkAction.InsertLink -> R.string.link_insert
                                    is LinkAction.SetLink -> R.string.link_set
                                }
                            )
                        )
                    }
                }
            }
        }
    }
}

@Preview
@Composable
fun PreviewSetLinkDialog() {
    LinkDialog(
        linkAction = LinkAction.SetLink("https://element.io"),
        onRemoveLink = {},
        onSetLink = {},
        onInsertLink = { _, _ -> },
        onDismissRequest = {}
    )
}

@Preview
@Composable
fun PreviewInsertLinkDialog() {
    LinkDialog(
        linkAction = LinkAction.InsertLink,
        onRemoveLink = {},
        onSetLink = {},
        onInsertLink = { _, _ -> },
        onDismissRequest = {}
    )
}
