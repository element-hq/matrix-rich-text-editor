/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.test.rules

import androidx.test.platform.app.InstrumentationRegistry.getInstrumentation
import androidx.test.uiautomator.UiDevice
import androidx.test.uiautomator.UiObject
import androidx.test.uiautomator.UiSelector
import org.junit.rules.TestWatcher
import org.junit.runner.Description

internal class DismissAnrRule : TestWatcher() {
    override fun starting(description: Description) {
        dismissAnr()
    }
}

private fun dismissAnr() {
    val device = UiDevice.getInstance(getInstrumentation())
    val dialog = device.findAnrDialog()
    if (dialog.exists()) {
        device.findWaitButton().click()
    }
}

private fun UiDevice.findAnrDialog(): UiObject =
    findObject(UiSelector().textContains("isn't responding"))

private fun UiDevice.findWaitButton(): UiObject =
    findObject(UiSelector().text("Wait").enabled(true))
        .apply { waitForExists(5000) }