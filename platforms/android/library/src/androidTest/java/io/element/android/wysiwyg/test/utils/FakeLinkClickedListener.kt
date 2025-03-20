/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.test.utils

import io.element.android.wysiwyg.link.Link
import org.junit.Assert

class FakeLinkClickedListener: (Link) -> Unit {
    private val clickedLinks: MutableList<Link> = mutableListOf()

    override fun invoke(link: Link) {
        clickedLinks.add(link)
    }

    fun assertLinkClicked(link: Link) {
        Assert.assertTrue(clickedLinks.size == 1)
        Assert.assertTrue(clickedLinks.contains(link))
    }
}