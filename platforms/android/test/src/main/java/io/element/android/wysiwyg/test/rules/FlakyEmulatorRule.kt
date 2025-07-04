/*
 * Copyright 2024 New Vector Ltd.
 * Copyright 2024 The Matrix.org Foundation C.I.C.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.test.rules

import org.junit.rules.RuleChain
import org.junit.rules.TestRule

/**
 * Creates a rule that helps to reduce emulator related flakiness.
 */
fun createFlakyEmulatorRule(retry: Boolean = true): TestRule = if (retry) {
    RuleChain
        .outerRule(RetryOnFailureRule())
        .around(DismissAnrRule())
} else {
    RuleChain
        .outerRule(DismissAnrRule())
}
