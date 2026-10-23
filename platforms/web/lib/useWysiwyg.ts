/*
Copyright 2024 New Vector Ltd.
Copyright 2022 The Matrix.org Foundation C.I.C.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

import { type RefObject, useEffect, useRef } from 'react';
import {
    useCreateAutoDisposedViewModel,
    useViewModel,
} from '@element-hq/web-shared-components';

import {
    type AllActionStates,
    type FormattingFunctions,
    type InputEventProcessor,
    type MappedSuggestion,
    type TraceAction,
} from './types.js';
import { WysiwygViewModel } from './WysiwygViewModel.js';
import { useWysiwygEditor } from './useWysiwygEditor.js';
import { useTestCases } from './useTestCases';
import { refreshComposerView } from './dom.js';

export { richToPlain, plainToRich } from './conversion';
export { initOnce } from './initOnce.js';

// MVVM API
export { WysiwygViewModel } from './WysiwygViewModel.js';
export type {
    EditorRef,
    EditorRender,
    WysiwygTestHooks,
    WysiwygViewActions,
    WysiwygViewModelProps,
    WysiwygViewSnapshot,
} from './WysiwygViewModel.js';
export { bindEditor } from './bindEditor.js';
export { useWysiwygEditor } from './useWysiwygEditor.js';

function useEditorFocus(
    editorRef: RefObject<HTMLElement | null>,
    isAutoFocusEnabled = false,
): void {
    useEffect(() => {
        if (isAutoFocusEnabled) {
            // TODO remove this workaround
            const id = setTimeout(() => editorRef.current?.focus(), 200);
            return (): void => clearTimeout(id);
        }
    }, [editorRef, isAutoFocusEnabled]);
}

export type WysiwygProps = {
    isAutoFocusEnabled?: boolean;
    inputEventProcessor?: InputEventProcessor;
    initialContent?: string;
    emojiSuggestions?: Map<string, string>;
};

export type UseWysiwyg = {
    ref: RefObject<HTMLDivElement | null>;
    isWysiwygReady: boolean;
    wysiwyg: FormattingFunctions;
    /** The view model behind this hook, for consumers migrating to MVVM. */
    viewModel: WysiwygViewModel;
    content: string | null;
    actionStates: AllActionStates;
    debug: {
        modelRef: RefObject<HTMLDivElement | null>;
        testRef: RefObject<HTMLDivElement | null>;
        resetTestCase: () => void | null;
        traceAction: TraceAction;
    };
    suggestion: MappedSuggestion | null;
    messageContent: string | null;
};

/**
 * React facade over {@link WysiwygViewModel}: creates the view model for the
 * lifetime of the component, binds the editor element behind `ref` to it,
 * and exposes the snapshot as plain values.
 *
 * `initialContent` is read once, when the view model is created. Later
 * changes to `inputEventProcessor` and `emojiSuggestions` are forwarded.
 */
export function useWysiwyg(wysiwygProps?: WysiwygProps): UseWysiwyg {
    const ref = useRef<HTMLDivElement | null>(null);
    const modelRef = useRef<HTMLDivElement>(null);

    const vm = useCreateAutoDisposedViewModel(
        () =>
            new WysiwygViewModel({
                initialContent: wysiwygProps?.initialContent,
                inputEventProcessor: wysiwygProps?.inputEventProcessor,
                emojiSuggestions: wysiwygProps?.emojiSuggestions,
                editorRef: ref,
            }),
    );

    const inputEventProcessor = wysiwygProps?.inputEventProcessor;
    useEffect(() => {
        vm.setInputEventProcessor(inputEventProcessor);
    }, [vm, inputEventProcessor]);

    const emojiSuggestions = wysiwygProps?.emojiSuggestions;
    useEffect(() => {
        vm.setEmojiSuggestions(emojiSuggestions);
    }, [vm, emojiSuggestions]);

    useWysiwygEditor(vm, ref);
    useEditorFocus(ref, wysiwygProps?.isAutoFocusEnabled);

    const snapshot = useViewModel(vm);

    // Debug tooling for the demo app: test-case recorder and model tree.
    const { testRef, utilities } = useTestCases(
        ref,
        snapshot.isReady ? vm.composerModel : null,
    );
    useEffect(() => {
        vm.setTestHooks(utilities);
    }, [vm, utilities]);
    useEffect(() => {
        if (snapshot.render) utilities.setEditorHtml(snapshot.render.html);
    }, [snapshot.render, utilities]);
    useEffect(() => {
        if (modelRef.current && vm.composerModel) {
            refreshComposerView(modelRef.current, vm.composerModel);
        }
    }, [snapshot, vm]);

    return {
        ref,
        isWysiwygReady: snapshot.isReady,
        wysiwyg: vm,
        viewModel: vm,
        content: snapshot.content,
        actionStates: snapshot.actionStates,
        debug: {
            modelRef,
            testRef,
            resetTestCase: utilities.onResetTestCase,
            traceAction: utilities.traceAction,
        },
        suggestion: snapshot.suggestion,
        messageContent: snapshot.messageContent,
    };
}
