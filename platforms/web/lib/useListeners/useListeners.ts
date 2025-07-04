/*
Copyright 2024 New Vector Ltd.
Copyright 2022 The Matrix.org Foundation C.I.C.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

import { RefObject, useEffect, useRef, useState } from 'react';
import {
    ComposerModel,
    SuggestionPattern,
} from '@vector-im/matrix-wysiwyg-wasm';

import { isClipboardEvent, isInputEvent } from './assert';
import { handleInput, handleKeyDown, handleSelectionChange } from './event';
import {
    FormattingFunctions,
    AllActionStates,
    InputEventProcessor,
    WysiwygInputEvent,
} from '../types';
import { TestUtilities } from '../useTestCases/types';
import { FormatBlockEvent } from './types';
import { createDefaultActionStates, mapToAllActionStates } from './utils';

type State = {
    content: string | null;
    actionStates: AllActionStates;
    suggestion: SuggestionPattern | null;
};

export function useListeners(
    editorRef: RefObject<HTMLElement | null>,
    modelRef: RefObject<HTMLElement | null>,
    composerModel: ComposerModel | null,
    testUtilities: TestUtilities,
    formattingFunctions: FormattingFunctions,
    onError: (content?: string) => void,
    inputEventProcessor?: InputEventProcessor,
    emojiSuggestions?: Map<string, string>,
): {
    areListenersReady: boolean;
    content: string | null;
    actionStates: AllActionStates;
    suggestion: SuggestionPattern | null;
} {
    const [state, setState] = useState<State>({
        content: null,
        actionStates: createDefaultActionStates(),
        suggestion: null,
    });

    const plainTextContentRef = useRef<string>(undefined);

    const [areListenersReady, setAreListenersReady] = useState(false);

    useEffect(() => {
        if (composerModel) {
            setState({
                content: composerModel.get_content_as_html(),
                actionStates: mapToAllActionStates(
                    composerModel.action_states(),
                ),
                suggestion: null,
            });
            plainTextContentRef.current =
                composerModel.get_content_as_plain_text();
        }
    }, [composerModel]);

    useEffect(() => {
        const editorNode = editorRef.current;
        if (!composerModel || !editorNode) {
            return;
        }

        const _handleInput = (e: WysiwygInputEvent): void => {
            try {
                const res = handleInput(
                    e,
                    editorNode,
                    composerModel,
                    modelRef.current,
                    testUtilities,
                    formattingFunctions,
                    state.suggestion,
                    inputEventProcessor,
                    emojiSuggestions,
                );

                if (res) {
                    setState((prevState) => {
                        // the state here is different for each piece of state
                        // state.content: update it if not undefined
                        const content =
                            res.content !== undefined
                                ? res.content
                                : prevState.content;

                        // state.actionStates: update if they are non-null
                        const actionStates =
                            res.actionStates || prevState.actionStates;

                        // state.suggestion: update even if null
                        const suggestion = res.suggestion;

                        return {
                            content,
                            actionStates,
                            suggestion,
                        };
                    });
                    plainTextContentRef.current =
                        composerModel.get_content_as_plain_text();
                }
            } catch {
                onError(plainTextContentRef.current);
            }
        };

        // React uses SyntheticEvent (https://reactjs.org/docs/events.html) and
        // doesn't catch manually fired event (myNode.dispatchEvent)

        // Also skip this if we are composing IME such as inputting accents or CJK
        const onInput = (e: Event): void => {
            if (isInputEvent(e) && !e.isComposing) {
                _handleInput(e);
            }
        };

        editorNode.addEventListener('input', onInput);

        // Can be called by onPaste or onBeforeInput
        const onPaste = (e: ClipboardEvent | InputEvent): void => {
            // this is required to handle edge case image pasting in Safari, see
            // https://github.com/vector-im/element-web/issues/25327
            const isSpecialCaseInputEvent =
                isInputEvent(e) &&
                e.inputType === 'insertFromPaste' &&
                e.dataTransfer !== null;

            const isEventToHandle =
                isClipboardEvent(e) || isSpecialCaseInputEvent;

            if (isEventToHandle) {
                e.preventDefault();
                e.stopPropagation();

                _handleInput(e);
            }
        };
        editorNode.addEventListener('paste', onPaste);

        const onWysiwygInput = ((e: FormatBlockEvent) => {
            _handleInput({
                inputType: e.detail.blockType,
                data: e.detail.data,
            } as WysiwygInputEvent);
        }) as EventListener;
        editorNode.addEventListener('wysiwygInput', onWysiwygInput);

        const onKeyDown = (e: KeyboardEvent): void => {
            handleKeyDown(
                e,
                editorNode,
                composerModel,
                formattingFunctions,
                inputEventProcessor,
            );
        };
        editorNode.addEventListener('keydown', onKeyDown);

        const onSelectionChange = (): void => {
            try {
                const actionStates = handleSelectionChange(
                    editorNode,
                    composerModel,
                    testUtilities,
                );

                if (actionStates) {
                    setState(({ content, suggestion }) => ({
                        content,
                        actionStates,
                        suggestion,
                    }));
                }
                plainTextContentRef.current =
                    composerModel.get_content_as_plain_text();
            } catch {
                onError(plainTextContentRef.current);
            }
        };
        document.addEventListener('selectionchange', onSelectionChange);

        // this is required to handle edge case image pasting in Safari, see
        // https://github.com/vector-im/element-web/issues/25327
        const onBeforeInput = (e: ClipboardEvent | InputEvent): void => {
            if (isInputEvent(e) && e.isComposing) return;
            return onPaste(e);
        };
        editorNode.addEventListener('beforeinput', onBeforeInput);

        const onCompositionEnd = (e: CompositionEvent): void => {
            // create a new inputEvent for us to process
            const inputEvent = new InputEvent('input', {
                data: e.data,
                inputType: 'insertCompositionText',
            });

            // now process that new event
            onInput(inputEvent);
        };
        editorNode.addEventListener('compositionend', onCompositionEnd);

        setAreListenersReady(true);

        return (): void => {
            setAreListenersReady(false);
            editorNode.removeEventListener('input', onInput);
            editorNode.removeEventListener('paste', onPaste);
            editorNode.removeEventListener('wysiwygInput', onWysiwygInput);
            editorNode.removeEventListener('keydown', onKeyDown);
            editorNode.removeEventListener('beforeinput', onBeforeInput);
            editorNode.removeEventListener('compositionend', onCompositionEnd);
            document.removeEventListener('selectionchange', onSelectionChange);
        };
    }, [
        editorRef,
        emojiSuggestions,
        composerModel,
        formattingFunctions,
        modelRef,
        testUtilities,
        inputEventProcessor,
        onError,
        plainTextContentRef,
        state.suggestion,
    ]);

    return { areListenersReady, ...state };
}
