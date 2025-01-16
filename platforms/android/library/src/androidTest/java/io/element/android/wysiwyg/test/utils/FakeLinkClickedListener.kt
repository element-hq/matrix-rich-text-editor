/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.test.utils

import org.junit.Assert

class FakeLinkClickedListener: (String) -> Unit {
    private val clickedLinks: MutableList<String> = mutableListOf()

    override fun invoke(link: String) {
        clickedLinks.add(link)
    }

    fun assertLinkClicked(url: String) {
        Assert.assertTrue(clickedLinks.size == 1)
        Assert.assertTrue(clickedLinks.contains(url))
    }
}