/*
Copyright 2024 New Vector Ltd.
Copyright 2022 The Matrix.org Foundation C.I.C.

SPDX-License-Identifier: AGPL-3.0-only
Please see LICENSE in the repository root for full details.
*/

import { RefObject, useEffect, useMemo, useRef, useState } from 'react';

import {
    AllActionStates,
    FormattingFunctions,
    InputEventProcessor,
    MappedSuggestion,
    TraceAction,
} from './types.js';
import { useFormattingFunctions } from './useFormattingFunctions';
import { useComposerModel } from './useComposerModel';
import { useListeners } from './useListeners';
import { useTestCases } from './useTestCases';
import { mapSuggestion } from './suggestion.js';

export { richToPlain, plainToRich } from './conversion';

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

function useEditor(): React.MutableRefObject<HTMLDivElement | null> {
    const ref = useRef<HTMLDivElement | null>(null);

    useEffect(() => {
        if (!ref.current?.childElementCount) {
            ref.current?.appendChild(document.createElement('br'));
        }
    }, [ref]);

    return ref;
}

export type WysiwygProps = {
    isAutoFocusEnabled?: boolean;
    inputEventProcessor?: InputEventProcessor;
    initialContent?: string;
    emojiSuggestions?: Map<string, string>;
};

export type UseWysiwyg = {
    ref: React.MutableRefObject<HTMLDivElement | null>;
    isWysiwygReady: boolean;
    wysiwyg: FormattingFunctions;
    content: string | null;
    actionStates: AllActionStates;
    debug: {
        modelRef: RefObject<HTMLDivElement>;
        testRef: RefObject<HTMLDivElement>;
        resetTestCase: () => void | null;
        traceAction: TraceAction;
    };
    suggestion: MappedSuggestion | null;
    messageContent: string | null;
};

function getEmojiKeys(emojiSuggestions?: Map<string, string>): string[] {
    const keys = emojiSuggestions?.keys();
    return keys ? Array.from(keys) : [];
}

export function useWysiwyg(wysiwygProps?: WysiwygProps): UseWysiwyg {
    const ref = useEditor();
    const modelRef = useRef<HTMLDivElement>(null);
    const [emojiKeys, setEmojiKeys] = useState(
        getEmojiKeys(wysiwygProps?.emojiSuggestions),
    );
    useEffect(() => {
        setEmojiKeys(getEmojiKeys(wysiwygProps?.emojiSuggestions));
    }, [wysiwygProps?.emojiSuggestions]);

    const { composerModel, onError } = useComposerModel(
        ref,
        wysiwygProps?.initialContent,
        emojiKeys,
    );
    const { testRef, utilities: testUtilities } = useTestCases(
        ref,
        composerModel,
    );

    const formattingFunctions = useFormattingFunctions(ref, composerModel);

    const { content, actionStates, areListenersReady, suggestion } =
        useListeners(
            ref,
            modelRef,
            composerModel,
            testUtilities,
            formattingFunctions,
            onError,
            wysiwygProps?.inputEventProcessor,
            wysiwygProps?.emojiSuggestions,
        );

    useEditorFocus(ref, wysiwygProps?.isAutoFocusEnabled);

    const memoisedMappedSuggestion = useMemo(
        () => mapSuggestion(suggestion),
        [suggestion],
    );

    return {
        ref,
        isWysiwygReady: areListenersReady,
        wysiwyg: formattingFunctions,
        content,
        actionStates,
        debug: {
            modelRef,
            testRef,
            resetTestCase: testUtilities.onResetTestCase,
            traceAction: testUtilities.traceAction,
        },
        suggestion: memoisedMappedSuggestion,
        messageContent: composerModel?.get_content_as_message_html() ?? null,
    };
}

export { initOnce } from './useComposerModel.js';
