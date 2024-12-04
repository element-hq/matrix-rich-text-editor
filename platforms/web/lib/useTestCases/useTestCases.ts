/*
Copyright 2024 New Vector Ltd.
Copyright 2022 The Matrix.org Foundation C.I.C.

SPDX-License-Identifier: AGPL-3.0-only
Please see LICENSE in the repository root for full details.
*/

import { RefObject, useCallback, useMemo, useRef, useState } from 'react';
import { ComposerModel } from '@vector-im/matrix-wysiwyg-wasm';

import { Actions } from './types';
import {
    getSelectionAccordingToActions,
    resetTestCase,
    traceAction,
} from './utils';
import { TraceAction } from '../types';

export type UseTestCases = {
    testRef: RefObject<HTMLDivElement>;
    utilities: {
        traceAction: TraceAction;
        getSelectionAccordingToActions: () => [number, number];
        onResetTestCase: () => void | null;
        setEditorHtml: (content: string) => void;
    };
};

export function useTestCases(
    editorRef: RefObject<HTMLElement | null>,
    composerModel: ComposerModel | null,
): UseTestCases {
    const testRef = useRef<HTMLDivElement>(null);
    const [actions, setActions] = useState<Actions>([]);

    const [editorHtml, setEditorHtml] = useState<string>('');

    const setEditor = useCallback(
        (content: string) => {
            if (testRef.current) {
                setEditorHtml(content);
            }
        },
        [testRef],
    );

    const memorizedTraceAction = useMemo(
        () => traceAction(testRef.current, actions, composerModel),
        [testRef, actions, composerModel],
    );

    const memorizedGetSelection = useMemo(
        () => getSelectionAccordingToActions(actions),
        [actions],
    );

    const onResetTestCase = useCallback(
        () =>
            editorRef.current &&
            testRef.current &&
            composerModel &&
            setActions(
                resetTestCase(
                    editorRef.current,
                    testRef.current,
                    composerModel,
                    editorHtml,
                ),
            ),
        [editorRef, testRef, composerModel, editorHtml],
    );

    const utilities = useMemo(
        () => ({
            traceAction: memorizedTraceAction,
            getSelectionAccordingToActions: memorizedGetSelection,
            onResetTestCase,
            setEditorHtml: setEditor,
        }),
        [
            memorizedTraceAction,
            memorizedGetSelection,
            onResetTestCase,
            setEditor,
        ],
    );

    return {
        testRef,
        utilities,
    };
}
