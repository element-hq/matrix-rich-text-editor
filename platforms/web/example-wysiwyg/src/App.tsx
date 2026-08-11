/*
Copyright 2024 New Vector Ltd.
Copyright 2022 The Matrix.org Foundation C.I.C.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

import { useWysiwyg } from '@vector-im/matrix-wysiwyg';

import './App.css';

function App(): React.ReactElement {
    const { ref, isWysiwygReady, wysiwyg } = useWysiwyg({
        isAutoFocusEnabled: true,
    });

    const insertTable = (): void => {
        const rows = Number(window.prompt('Rows?', '2')) || 2;
        const columns = Number(window.prompt('Columns?', '2')) || 2;
        wysiwyg.insertTable(rows, columns);
    };

    return (
        <div>
            <button onClick={wysiwyg.undo}>undo</button>
            <button onClick={wysiwyg.redo}>redo</button>
            <button onClick={wysiwyg.bold}>bold</button>
            <button onClick={wysiwyg.italic}>italic</button>
            <button onClick={wysiwyg.underline}>underline</button>
            <button onClick={wysiwyg.strikeThrough}>strikeThrough</button>
            <button onClick={wysiwyg.orderedList}>orderedList</button>
            <button onClick={wysiwyg.unorderedList}>unorderedList</button>
            <button onClick={wysiwyg.inlineCode}>inlineCode</button>
            <button onClick={wysiwyg.clear}>clear</button>
            <button onClick={insertTable}>insertTable</button>
            <button onClick={wysiwyg.addTableRowAfter}>addTableRow</button>
            <button onClick={wysiwyg.removeTableRow}>removeTableRow</button>
            <button onClick={wysiwyg.addTableColumnAfter}>
                addTableColumn
            </button>
            <button onClick={wysiwyg.removeTableColumn}>
                removeTableColumn
            </button>
            <button onClick={wysiwyg.toggleTableHeader}>
                toggleTableHeader
            </button>
            <button onClick={wysiwyg.removeTable}>removeTable</button>
            <div ref={ref} contentEditable={isWysiwygReady} />
        </div>
    );
}

export default App;
