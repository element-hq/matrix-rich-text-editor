/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.compose

import android.content.Context
import io.element.android.wysiwyg.compose.internal.toStyleConfig
import io.element.android.wysiwyg.display.MentionDisplayHandler
import io.element.android.wysiwyg.utils.HtmlConverter
import timber.log.Timber

/**
 * An [HtmlConverter] that can be configured with a [RichTextEditorStyle].
 */
class StyledHtmlConverter(
    private val context: Context,
    private val mentionDisplayHandler: MentionDisplayHandler?,
    private val isEditor: Boolean,
    private val isMention: ((text: String, url: String) -> Boolean)?,
) : HtmlConverter {

    private var htmlConverter: HtmlConverter? = null

    fun configureWith(style: RichTextEditorStyle) {
        Timber.d("Configure with style: $style")
        htmlConverter = HtmlConverter.Factory.create(
            context = context,
            styleConfig = style.toStyleConfig(context),
            mentionDisplayHandler = mentionDisplayHandler,
            isEditor = isEditor,
            isMention = isMention,
        )
    }

    override fun fromHtmlToSpans(html: String): CharSequence {
        return htmlConverter?.fromHtmlToSpans(html) ?: errorNotConfigured()
    }

    private fun errorNotConfigured(): Nothing {
        error("ComposableHtmlConverter must be configured with a RichTextEditorStyle before use")
    }


}
