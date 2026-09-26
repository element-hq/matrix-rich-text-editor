/*
 * Copyright 2026 New Vector Ltd.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.compose.next.internal

import io.element.android.wysiwyg.compose.next.internal.document.NBSP

/**
 * Replace the [start]..[end] range of some old text with [replacement].
 */
internal data class TextEdit(
    val start: Int,
    val end: Int,
    val replacement: String,
)

/**
 * Computes the smallest [TextEdit] that transforms [old] into [new], only looking for differences
 * between [old] in [oldStart]..[oldEnd] and [new] in [newStart]..[newEnd]. The rest of both texts
 * is assumed to be equal.
 *
 * Surrogate pairs are never split and spaces and non-breaking spaces are considered equal,
 * see [isLooselyEqualTo].
 *
 * @return the edit to apply to [old], or null if there are no differences.
 */
internal fun minimalEdit(
    old: CharSequence,
    new: CharSequence,
    oldStart: Int = 0,
    oldEnd: Int = old.length,
    newStart: Int = 0,
    newEnd: Int = new.length,
): TextEdit? {
    var prefix = 0
    val maxPrefix = minOf(oldEnd - oldStart, newEnd - newStart)
    while (prefix < maxPrefix && old[oldStart + prefix].isLooselyEqualTo(new[newStart + prefix])) {
        prefix++
    }
    // Don't split a surrogate pair
    if (prefix > 0 && old[oldStart + prefix - 1].isHighSurrogate()) prefix--

    var suffix = 0
    val maxSuffix = maxPrefix - prefix
    while (suffix < maxSuffix && old[oldEnd - 1 - suffix].isLooselyEqualTo(new[newEnd - 1 - suffix])) {
        suffix++
    }
    if (suffix > 0 && old[oldEnd - suffix].isLowSurrogate()) suffix--

    val editStart = oldStart + prefix
    val editEnd = oldEnd - suffix
    val replacement = new.subSequence(newStart + prefix, newEnd - suffix).toString()
    if (editStart == editEnd && replacement.isEmpty()) return null
    return TextEdit(editStart, editEnd, replacement)
}

/**
 * Rust uses non-breaking spaces in its HTML output to preserve consecutive and trailing
 * whitespaces, but the user actually typed a plain space, so we treat them as equal to avoid
 * modifying the text field contents (and resetting the IME composition) because of them.
 */
internal fun Char.isLooselyEqualTo(other: Char): Boolean =
    this == other || (this.isSpaceLike() && other.isSpaceLike())

private fun Char.isSpaceLike() = this == ' ' || this == NBSP

internal fun CharSequence.isLooselyEqualTo(other: CharSequence): Boolean {
    if (length != other.length) return false
    for (i in indices) {
        if (!this[i].isLooselyEqualTo(other[i])) return false
    }
    return true
}
