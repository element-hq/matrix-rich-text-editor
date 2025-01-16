/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.test.utils

import android.content.ContentValues
import android.content.Context
import android.graphics.Bitmap
import android.os.Environment
import android.provider.MediaStore
import android.view.View
import androidx.test.espresso.FailureHandler
import androidx.test.espresso.base.DefaultFailureHandler
import androidx.test.platform.app.InstrumentationRegistry.getInstrumentation
import org.hamcrest.Matcher
import java.io.IOException
import java.text.SimpleDateFormat
import java.util.Calendar

class ScreenshotFailureHandler(appContext: Context) : FailureHandler {
    private val defaultHandler: FailureHandler = DefaultFailureHandler(appContext)

    override fun handle(error: Throwable, viewMatcher: Matcher<View>) {
        getInstrumentation()
            .uiAutomation
            .takeScreenshot()
            .save()

        defaultHandler.handle(error, viewMatcher)
    }
}

private fun Bitmap.save() {
    val timestamp = timestamp()
    val contentResolver = getInstrumentation().targetContext.applicationContext.contentResolver
    try {
        val contentValues = ContentValues().apply {
            put(MediaStore.MediaColumns.DISPLAY_NAME, "$timestamp.jpeg")
            put(MediaStore.Images.Media.RELATIVE_PATH, "${Environment.DIRECTORY_PICTURES}/UiTest")
        }

        val uri = contentResolver
            .insert(MediaStore.Images.Media.EXTERNAL_CONTENT_URI, contentValues)
                ?: return

        contentResolver.openOutputStream(uri)?.use { outputStream ->
            compress(Bitmap.CompressFormat.PNG, 20, outputStream)
        }

        contentResolver.update(uri, contentValues, null, null)
    } catch (e: IOException) {
        e.printStackTrace()
    }
}

private fun timestamp(): String =
    SimpleDateFormat("yyyyMMdd_HHmmss").format(Calendar.getInstance().time)
