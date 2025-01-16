/*
Copyright 2024 New Vector Ltd.
Copyright 2022 The Matrix.org Foundation C.I.C.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

import { selectContent } from '../dom';

export function select(
    editor: HTMLDivElement,
    startIndex: number,
    endIndex: number,
): void {
    selectContent(editor, startIndex, endIndex);

    // the event is not automatically fired in jest
    document.dispatchEvent(new CustomEvent('selectionchange'));
}

export function deleteRange(
    editor: HTMLDivElement,
    start: number,
    end: number,
): void {
    select(editor, start, end);
    const sel = document.getSelection();
    sel?.deleteFromDocument();
}
