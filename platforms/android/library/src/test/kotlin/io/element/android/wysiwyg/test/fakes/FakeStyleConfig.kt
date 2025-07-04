/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.test.fakes

import android.graphics.drawable.ColorDrawable
import io.element.android.wysiwyg.test.R
import io.element.android.wysiwyg.view.BulletListStyleConfig
import io.element.android.wysiwyg.view.CodeBlockStyleConfig
import io.element.android.wysiwyg.view.InlineCodeStyleConfig
import io.element.android.wysiwyg.view.PillStyleConfig
import io.element.android.wysiwyg.view.StyleConfig

private val fakeDrawable = ColorDrawable()

internal fun createFakeStyleConfig() = StyleConfig(
    bulletList = BulletListStyleConfig(
        bulletGapWidth = 1f,
        bulletRadius = 1f,
    ),
    inlineCode = InlineCodeStyleConfig(
        horizontalPadding = 2,
        verticalPadding = 2,
        relativeTextSize = 1f,
        singleLineBg = fakeDrawable,
        multiLineBgLeft = fakeDrawable,
        multiLineBgMid = fakeDrawable,
        multiLineBgRight = fakeDrawable,
    ),
    codeBlock = CodeBlockStyleConfig(
        leadingMargin = 0,
        verticalPadding = 0,
        relativeTextSize = 1f,
        backgroundDrawable = fakeDrawable,
    ),
    pill = PillStyleConfig(
        android.R.color.white
    )
)
