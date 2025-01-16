/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.internal.view.models

import io.element.android.wysiwyg.view.models.LinkAction
import uniffi.wysiwyg_composer.LinkAction as InternalLinkAction

internal fun InternalLinkAction.toApiModel(): LinkAction? =
    when (this) {
        is InternalLinkAction.Edit -> LinkAction.SetLink(currentUrl = url)
        is InternalLinkAction.Create -> LinkAction.SetLink(currentUrl = null)
        is InternalLinkAction.CreateWithText -> LinkAction.InsertLink
        is InternalLinkAction.Disabled -> null
    }
