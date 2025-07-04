/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.mocks

import io.mockk.every
import io.mockk.mockk
import uniffi.wysiwyg_composer.ComposerUpdate
import uniffi.wysiwyg_composer.LinkActionUpdate
import uniffi.wysiwyg_composer.MenuAction
import uniffi.wysiwyg_composer.MenuState
import uniffi.wysiwyg_composer.TextUpdate

object MockComposerUpdateFactory {
    fun create(
        menuAction: MenuAction = MenuAction.Keep,
        menuState: MenuState = MenuState.Keep,
        textUpdate: TextUpdate = TextUpdate.Keep,
        linkAction: LinkActionUpdate = LinkActionUpdate.Keep,
    ): ComposerUpdate = mockk {
        every { menuAction() } returns menuAction
        every { menuState() } returns menuState
        every { textUpdate() } returns textUpdate
        every { linkAction() } returns linkAction
    }
}
