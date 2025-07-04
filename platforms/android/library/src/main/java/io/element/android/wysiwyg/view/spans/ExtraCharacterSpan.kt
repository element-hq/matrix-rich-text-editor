/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.view.spans

import android.text.NoCopySpan
import uniffi.wysiwyg_composer.ComposerModel

/**
 * Special 'span' used as a marker to know there's a difference between the indexes in the Rust code
 * and the ones in the UI components.
 *
 * This is done because in the Dom, just using a `<li>` tag will create a new line break in a list,
 * but for Android we have to add an extra `\n` line break character for list items to be rendered
 * properly. To fix this mismatch, we use this [ExtraCharacterSpan] to understand when a line break
 * character should be left as is, because it exists in the [ComposerModel] or if we should handle
 * it in a special way because it's not present in the HTML output.
 */
class ExtraCharacterSpan: NoCopySpan
