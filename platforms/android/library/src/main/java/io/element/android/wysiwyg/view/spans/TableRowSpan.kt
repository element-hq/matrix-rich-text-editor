/*
 * Copyright 2026 New Vector Ltd.
 *
 * SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
 * Please see LICENSE in the repository root for full details.
 */

package io.element.android.wysiwyg.view.spans

/**
 * Table row (<tr> in HTML) marker span, used purely to locate each row's line bounds so
 * [io.element.android.wysiwyg.view.inlinebg.TableRowDividerRenderer] can draw a divider line
 * between rows. Draws nothing itself.
 */
class TableRowSpan
