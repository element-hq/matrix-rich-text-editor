/*
 * Copyright 2026 New Vector Ltd.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.compose.next.internal

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class TextDiffTest {
    @Test
    fun `no changes returns null`() {
        assertNull(minimalEdit("hello", "hello"))
    }

    @Test
    fun `space and nbsp are considered equal`() {
        assertNull(minimalEdit("hello world ", "hello world "))
    }

    @Test
    fun `insertion at the end`() {
        assertEquals(TextEdit(5, 5, "!"), minimalEdit("hello", "hello!"))
    }

    @Test
    fun `deletion in the middle`() {
        assertEquals(TextEdit(2, 3, ""), minimalEdit("abcde", "abde"))
    }

    @Test
    fun `replacement of a composing word`() {
        assertEquals(TextEdit(3, 4, "lo "), minimalEdit("helo", "hello "))
    }

    @Test
    fun `only looks at the given window`() {
        // Typing 'a' between 2 other 'a's must be an insertion at 1, not at the end
        assertEquals(
            TextEdit(1, 1, "a"),
            minimalEdit("aa", "aaa", oldStart = 1, oldEnd = 1, newStart = 1, newEnd = 2),
        )
    }

    @Test
    fun `does not split surrogate pairs`() {
        // Replacing one emoji with another sharing the high surrogate
        val edit = minimalEdit("a😀b", "a😁b")
        assertEquals(TextEdit(1, 3, "😁"), edit)
    }
}
