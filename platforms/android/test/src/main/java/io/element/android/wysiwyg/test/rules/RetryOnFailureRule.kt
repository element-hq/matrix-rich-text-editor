/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.test.rules

import org.junit.rules.TestRule
import org.junit.runner.Description
import org.junit.runners.model.Statement

internal class RetryOnFailureRule : TestRule {
    override fun apply(
        base: Statement,
        description: Description
    ): Statement =
        RetryStatement(base)
}

private class RetryStatement(private val base: Statement) : Statement() {
    override fun evaluate() {
        try {
            base.evaluate()
            return
        } catch (t: Throwable) {
            base.evaluate()
        }
    }
}
