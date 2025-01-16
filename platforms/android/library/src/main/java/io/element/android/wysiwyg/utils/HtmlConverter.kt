/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.utils

import android.app.Application
import android.content.Context
import io.element.android.wysiwyg.display.MentionDisplayHandler
import io.element.android.wysiwyg.internal.utils.AndroidHtmlConverter
import io.element.android.wysiwyg.view.StyleConfig

interface HtmlConverter {

    fun fromHtmlToSpans(html: String): CharSequence

    object Factory {
        fun create(
            context: Context,
            styleConfig: StyleConfig,
            mentionDisplayHandler: MentionDisplayHandler?,
            isEditor: Boolean,
            isMention: ((text: String, url: String) -> Boolean)? = null,
        ): HtmlConverter {
            val resourcesProvider = AndroidResourcesHelper(context)
            return AndroidHtmlConverter(provideHtmlToSpansParser = { html ->
                HtmlToSpansParser(
                    resourcesHelper = resourcesProvider,
                    html = html,
                    styleConfig = styleConfig,
                    mentionDisplayHandler = mentionDisplayHandler,
                    isEditor = isEditor,
                    isMention = isMention,
                )
            })
        }
    }


}
