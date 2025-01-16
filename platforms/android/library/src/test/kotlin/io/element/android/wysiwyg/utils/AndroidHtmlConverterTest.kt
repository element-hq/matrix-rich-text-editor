/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.utils

import androidx.core.text.toSpanned
import io.element.android.wysiwyg.internal.utils.AndroidHtmlConverter
import io.mockk.every
import io.mockk.mockk
import org.hamcrest.MatcherAssert.assertThat
import org.hamcrest.Matchers.equalTo
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

@RunWith(RobolectricTestRunner::class)
class AndroidHtmlConverterTest {
    private val htmlToSpansParser = mockk<HtmlToSpansParser>()
    private val androidHtmlConverter = AndroidHtmlConverter(
        provideHtmlToSpansParser = { htmlToSpansParser }
    )

    @Test
    fun testToSpans() {
        val expectedParserOutput = "mock parser output".toSpanned()
        every { htmlToSpansParser.convert() } returns expectedParserOutput

        val result = androidHtmlConverter.fromHtmlToSpans("input")

        assertThat(result, equalTo(expectedParserOutput))
    }
}