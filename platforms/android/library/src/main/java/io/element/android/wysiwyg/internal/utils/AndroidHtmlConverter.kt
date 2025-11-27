/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.internal.utils

import io.element.android.wysiwyg.utils.HtmlConverter
import io.element.android.wysiwyg.utils.HtmlToDomParser
import io.element.android.wysiwyg.utils.HtmlToSpansParser
import org.jsoup.nodes.Document

internal class AndroidHtmlConverter(
    private val provideHtmlToSpansParser: (dom: Document) -> HtmlToSpansParser
) : HtmlConverter {

    override fun fromHtmlToSpans(html: String): CharSequence {
        val dom = HtmlToDomParser.document(html)
        return fromDocumentToSpans(dom)
    }

    override fun fromDocumentToSpans(dom: Document): CharSequence {
        return provideHtmlToSpansParser(dom).convert()
    }
}