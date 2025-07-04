/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.extensions

/**
 * Translates the Rust [UShort] list returned for strings into actual JVM Strings that we can use.
 */
internal fun List<UShort>.string() = with(StringBuffer()) {
    this@string.forEach {
        appendCodePoint(it.toInt())
    }
    toString()
}

/**
 * Translates a JVM String into a Rust [UShort] list.
 */
internal fun String.toUShortList(): List<UShort> = encodeToByteArray()
    .map(Byte::toUShort)
