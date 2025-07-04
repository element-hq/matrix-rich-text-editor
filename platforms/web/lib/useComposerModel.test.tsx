/*
Copyright 2024 New Vector Ltd.
Copyright 2023 The Matrix.org Foundation C.I.C.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

import { act, RefObject } from 'react';
import { renderHook, waitFor } from '@testing-library/react';
import * as mockRustModel from '@vector-im/matrix-wysiwyg-wasm';

import { useComposerModel } from './useComposerModel';

describe('useComposerModel', () => {
    let mockComposer: HTMLDivElement;
    let mockNullRef: RefObject<null>;
    let mockComposerRef: RefObject<HTMLElement>;

    beforeEach(() => {
        mockComposer = document.createElement('div');
        mockNullRef = { current: null };
        mockComposerRef = {
            current: mockComposer,
        };
        vi.spyOn(mockRustModel, 'new_composer_model');
        vi.spyOn(mockRustModel, 'new_composer_model_from_html');
    });

    afterEach(() => {
        vi.clearAllMocks();
    });

    afterAll(() => {
        vi.restoreAllMocks();
    });

    it('Does not create a composerModel without a ref', () => {
        const { result } = renderHook(() => useComposerModel(mockNullRef));

        expect(result.current.composerModel).toBeNull();
    });

    it('Only calls `new_composer_model` if ref exists but no initial content exists', async () => {
        const { result } = renderHook(() => useComposerModel(mockComposerRef));

        // wait for the composerModel to be created
        await waitFor(() => {
            expect(result.current.composerModel).not.toBeNull();
        });

        // check only new_composer_model has been called
        expect(mockRustModel.new_composer_model).toHaveBeenCalledTimes(1);
        expect(
            mockRustModel.new_composer_model_from_html,
        ).not.toHaveBeenCalled();
    });

    it('Calls `new_composer_model_from_html` if ref and initial content exists', async () => {
        const { result } = renderHook(() =>
            useComposerModel(mockComposerRef, 'some content'),
        );

        // wait for the composerModel to be created
        await waitFor(() => {
            expect(result.current.composerModel).not.toBeNull();
        });

        // check only new_composer_model_from_html has been called
        expect(
            mockRustModel.new_composer_model_from_html,
        ).toHaveBeenCalledTimes(1);
        expect(mockRustModel.new_composer_model).not.toHaveBeenCalled();
    });

    it('Sets the ref inner html when initial content is valid html', async () => {
        const inputContent = `<a href="this is allowed" other="disallowedattribute">test link</a>`;

        // the rust model will strip "bad" attributes and the hook always adds a trailing <br>
        const expectedComposerInnerHtml = `<a href="this is allowed">test link</a><br>`;
        const { result } = renderHook(() =>
            useComposerModel(mockComposerRef, inputContent),
        );

        // wait for the composerModel to be created
        await waitFor(() => {
            expect(result.current.composerModel).not.toBeNull();
        });

        // check that the content of the div is the rust model output
        expect(mockComposer.innerHTML).toBe(expectedComposerInnerHtml);
    });

    it('Falls back to calling `new_composer_model` if there is a parsing error', async () => {
        // Use badly formed initial content to cause a html parsing error
        const { result } = renderHook(() =>
            useComposerModel(mockComposerRef, '<badly>formed content</>'),
        );

        // wait for the composerModel to be created
        await waitFor(() => {
            expect(result.current.composerModel).not.toBeNull();
        });

        // check that both functions have been called
        expect(
            mockRustModel.new_composer_model_from_html,
        ).toHaveBeenCalledTimes(1);
        expect(mockRustModel.new_composer_model).toHaveBeenCalledTimes(1);
    });

    it("Doesn't double intialize the model if customSuggestionPatterns are set", async () => {
        const useProps: {
            editorRef: RefObject<HTMLElement | null>;
            initialContent?: string;
            customSuggestionPatterns?: Array<string>;
        } = {
            editorRef: mockComposerRef,
            initialContent: '',
            customSuggestionPatterns: undefined,
        };

        const { result, rerender } = renderHook(
            (props: {
                editorRef: RefObject<HTMLElement | null>;
                initialContent?: string;
                customSuggestionPatterns?: Array<string>;
            }) =>
                useComposerModel(
                    props.editorRef,
                    props.initialContent,
                    props.customSuggestionPatterns,
                ),
            { initialProps: useProps },
        );

        // wait for the composerModel to be created
        await waitFor(() => {
            expect(result.current.composerModel).not.toBeNull();
        });

        await act(() => {
            useProps.customSuggestionPatterns = ['test'];
            rerender(useProps);
        });

        expect(mockRustModel.new_composer_model).toHaveBeenCalledTimes(1);
        expect(
            mockRustModel.new_composer_model_from_html,
        ).not.toHaveBeenCalled();
    });
});
