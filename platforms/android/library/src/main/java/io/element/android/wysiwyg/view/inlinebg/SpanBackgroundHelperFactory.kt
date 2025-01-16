/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.view.inlinebg

import io.element.android.wysiwyg.view.spans.CodeBlockSpan
import io.element.android.wysiwyg.view.spans.InlineCodeSpan
import io.element.android.wysiwyg.view.CodeBlockStyleConfig
import io.element.android.wysiwyg.view.InlineCodeStyleConfig

object SpanBackgroundHelperFactory {
    fun createInlineCodeBackgroundHelper(styleConfig: InlineCodeStyleConfig): SpanBackgroundHelper {
        return SpanBackgroundHelper(
            spanType = InlineCodeSpan::class.java,
            horizontalPadding = styleConfig.horizontalPadding,
            verticalPadding = styleConfig.verticalPadding,
            drawable = styleConfig.singleLineBg,
            drawableLeft = styleConfig.multiLineBgLeft,
            drawableMid = styleConfig.multiLineBgMid,
            drawableRight = styleConfig.multiLineBgRight,
        )
    }

    fun createCodeBlockBackgroundHelper(styleConfig: CodeBlockStyleConfig): SpanBackgroundHelper {
        return SpanBackgroundHelper(
            spanType = CodeBlockSpan::class.java,
            horizontalPadding = 0,
            verticalPadding = 0,
            drawable = styleConfig.backgroundDrawable,
        )
    }
}
