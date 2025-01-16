/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.compose.testutils

import android.view.View
import androidx.appcompat.widget.AppCompatEditText
import androidx.test.espresso.UiController
import androidx.test.espresso.ViewAction
import androidx.test.espresso.matcher.ViewMatchers.isDisplayed
import org.hamcrest.Matcher

object Editor {
    class SetSelection(
        private val start: Int,
        private val end: Int,
    ) : ViewAction {
        override fun getConstraints(): Matcher<View> = isDisplayed()

        override fun getDescription(): String = "Set selection to $start, $end"

        override fun perform(uiController: UiController?, view: View?) {
            val editor = view as? AppCompatEditText ?: return
            editor.setSelection(start, end)
        }
    }

}

object EditorActions {
    fun setSelection(start: Int, end: Int) = Editor.SetSelection(start, end)
}
