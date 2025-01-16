/*
Copyright 2024 New Vector Ltd.
Copyright 2022 The Matrix.org Foundation C.I.C.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

import { ComposerModel } from '@vector-im/matrix-wysiwyg-wasm';

import { processInput } from './composer';
import { FormattingFunctions } from './types';

// mocks and spies
const mockComposerModel = {
    replace_text: vi.fn(),
    code_block: vi.fn(),
    backspace_word: vi.fn(),
    delete_word: vi.fn(),
    delete: vi.fn(),
    inline_code: vi.fn(),
    ordered_list: vi.fn(),
    unordered_list: vi.fn(),
    quote: vi.fn(),
    enter: vi.fn(),
    indent: vi.fn(),
    unindent: vi.fn(),
    set_content_from_html: vi.fn(),
} as unknown as ComposerModel;

const mockAction = vi.fn();

const mockSuggestion = null;

const mockFormattingFunctions = {} as unknown as FormattingFunctions;

const consoleErrorSpy = vi.spyOn(console, 'error');

// regular test cases
type testCase = {
    eventType: string;
    eventArguments?: string[];
    composerMethod: keyof typeof ComposerModel.prototype;
    composerArguments?: unknown[];
    actionArguments?: unknown[];
};

const testCases: testCase[] = [
    {
        eventType: 'insertText',
        eventArguments: ['goo'],
        composerMethod: 'replace_text',
        composerArguments: ['goo'],
        actionArguments: ['goo'],
    },
    {
        eventType: 'insertCompositionText',
        eventArguments: ['gar'],
        composerMethod: 'replace_text',
        composerArguments: ['gar'],
        actionArguments: ['gar'],
    },
    {
        eventType: 'insertFromComposition',
        eventArguments: ['gaz'],
        composerMethod: 'replace_text',
        composerArguments: ['gaz'],
        actionArguments: ['gaz'],
    },
    {
        eventType: 'insertCodeBlock',
        composerMethod: 'code_block',
    },
    {
        eventType: 'deleteWordBackward',
        composerMethod: 'backspace_word',
    },
    {
        eventType: 'deleteWordForward',
        composerMethod: 'delete_word',
    },
    {
        eventType: 'deleteByCut',
        composerMethod: 'delete',
    },
    {
        eventType: 'formatInlineCode',
        composerMethod: 'inline_code',
    },
    {
        eventType: 'insertOrderedList',
        composerMethod: 'ordered_list',
    },
    {
        eventType: 'insertUnorderedList',
        composerMethod: 'unordered_list',
    },
    {
        eventType: 'insertLineBreak',
        composerMethod: 'enter',
    },
    {
        eventType: 'insertQuote',
        composerMethod: 'quote',
    },
    {
        eventType: 'formatIndent',
        composerMethod: 'indent',
    },
    {
        eventType: 'formatOutdent',
        composerMethod: 'unindent',
    },
    {
        eventType: 'insertReplacementText',
        composerMethod: 'set_content_from_html',
        composerArguments: ['content'],
        actionArguments: ['content'],
    },
];

const editor = document.createElement('div');
editor.innerHTML = 'content<br>';

describe('processInput', () => {
    beforeEach(() => {
        vi.resetAllMocks();
    });

    afterAll(() => {
        vi.restoreAllMocks();
    });

    it.each(testCases)(
        'handles $eventType with $composerMethod',
        ({
            eventType,
            eventArguments = [],
            composerMethod,
            composerArguments,
            actionArguments = [],
        }) => {
            // generate the event
            const e = inpEv(eventType, ...eventArguments);

            // process the input using the new event
            processInput(
                e,
                mockComposerModel,
                mockAction,
                mockFormattingFunctions,
                editor,
                mockSuggestion,
            );

            // check the calls to the composerModel and the action function
            const method = mockComposerModel[composerMethod];

            expect(method).toHaveBeenCalledTimes(1);
            if (composerArguments) {
                expect(method).toHaveBeenCalledWith(...composerArguments);
            }
            expect(mockAction).toHaveBeenCalledWith(
                undefined, // output from the jest mock in mockComposerModel
                composerMethod,
                ...actionArguments,
            );
        },
    );

    // special cases
    it('returns early if inputEventProcessor returns null', () => {
        const mockInputEventProcessor = vi.fn().mockReturnValue(null);

        const returnValue = processInput(
            new InputEvent('some event'),
            mockComposerModel,
            mockAction,
            mockFormattingFunctions,
            editor,
            mockSuggestion,
            mockInputEventProcessor,
        );

        expect(returnValue).not.toBeDefined();
        expect(mockAction).not.toHaveBeenCalled();
    });

    it('handles truthy and falsy data from clipboard with replace_text', () => {
        const sampleContent = ['clipboardData', null];

        sampleContent.forEach((clipboardContent) => {
            const e = new ClipboardEvent('some clipboard event');
            const mockGetter = vi.fn().mockReturnValue(clipboardContent);

            // We can't easily generate the correct type here, so disable ts
            // eslint-disable-next-line @typescript-eslint/ban-ts-comment
            // @ts-ignore
            e.clipboardData = { getData: mockGetter };

            processInput(
                e,
                mockComposerModel,
                mockAction,
                mockFormattingFunctions,
                editor,
                mockSuggestion,
            );

            expect(mockGetter).toHaveBeenCalledTimes(1);
            expect(mockComposerModel.replace_text).toHaveBeenCalledWith(
                // falsy values are defaulted to empty string
                clipboardContent || '',
            );
            expect(mockAction).toHaveBeenCalledWith(undefined, 'paste');
        });
    });

    it('handles insertFromPaste without calling action', () => {
        const e = inpEv('insertFromPaste');

        // When we process the input
        const returnValue = processInput(
            e,
            mockComposerModel,
            mockAction,
            mockFormattingFunctions,
            editor,
            mockSuggestion,
        );

        // Then mockAction have is not called
        expect(returnValue).not.toBeDefined();
        expect(mockAction).not.toHaveBeenCalled();
    });

    it('hits the break statement in insert text if input data is falsy', () => {
        const e = inpEv('insertText', '');

        // When we process the input
        const returnValue = processInput(
            e,
            mockComposerModel,
            mockAction,
            mockFormattingFunctions,
            editor,
            mockSuggestion,
        );

        // Then mockAction have is not called
        expect(returnValue).not.toBeDefined();
        expect(mockAction).not.toHaveBeenCalled();
    });

    it('returns null from a send message event', () => {
        const e = inpEv('sendMessage');

        // When we process the input
        const returnValue = processInput(
            e,
            mockComposerModel,
            mockAction,
            mockFormattingFunctions,
            editor,
            mockSuggestion,
        );

        // Then mockAction is not called and we get null back
        expect(mockAction).not.toHaveBeenCalled();
        expect(returnValue).toBe(null);
    });

    it('logs errors and returns null when unknown event is used', () => {
        const eventType = 'aNewEventType';
        const e = inpEv(eventType);

        // When we process the input
        const returnValue = processInput(
            e,
            mockComposerModel,
            mockAction,
            mockFormattingFunctions,
            editor,
            mockSuggestion,
        );

        // Then mockAction is not called, we get null back and there is an error
        // displayed in the console
        expect(mockAction).not.toHaveBeenCalled();
        expect(returnValue).toBe(null);
        expect(consoleErrorSpy).toHaveBeenCalledWith(
            `Unknown input type: ${eventType}`,
        );
    });
});

// util functions
function inpEv(inputType: string, data?: string): InputEvent {
    return new InputEvent('InputEvent', { data, inputType });
}
