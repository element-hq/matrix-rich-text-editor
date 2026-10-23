/*
Copyright 2026 Element Creations Ltd.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

import { type RefObject, useEffect } from 'react';

import { bindEditor } from './bindEditor.js';
import { type WysiwygViewModel } from './WysiwygViewModel.js';

/**
 * Bind the contenteditable element behind `editorRef` to `vm` for as long as
 * the calling component is mounted. The element must be rendered on the
 * first commit.
 */
export function useWysiwygEditor(
    vm: WysiwygViewModel,
    editorRef: RefObject<HTMLElement | null>,
): void {
    useEffect(() => {
        const editor = editorRef.current;
        if (!editor) return;
        return bindEditor(vm, editor);
    }, [vm, editorRef]);
}
