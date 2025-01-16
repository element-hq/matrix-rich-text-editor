/*
Copyright 2024 New Vector Ltd.
Copyright 2022 The Matrix.org Foundation C.I.C.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

import { fireEvent, render, screen, waitFor } from '@testing-library/react';

import { Editor } from './testUtils/Editor';

describe('inputEventProcessor', () => {
    const inputEventProcessor = vitest.fn();
    let textbox: HTMLElement;

    beforeEach(async () => {
        render(<Editor inputEventProcessor={inputEventProcessor} />);
        textbox = screen.getByRole('textbox');
        await waitFor(() =>
            expect(textbox).toHaveAttribute('contentEditable', 'true'),
        );
        inputEventProcessor.mockReset();
    });

    it('Should call inputEventProcess on keydown', async () => {
        // When
        fireEvent.keyDown(textbox, { key: 'A', code: 'KeyA' });

        await waitFor(() => {
            expect(inputEventProcessor).toBeCalledTimes(1);
            expect(inputEventProcessor).toBeCalledWith(
                new KeyboardEvent('keyDown', {
                    key: 'A',
                    code: 'KeyA',
                }),
                expect.anything(),
                textbox,
            );
        });
    });

    it('Should call inputEventProcessor', async () => {
        // When
        const inputEvent = new InputEvent('input', {
            data: 'foo',
            inputType: 'insertText',
        });
        inputEventProcessor.mockReturnValue(null);
        fireEvent(textbox, inputEvent);

        // Then
        // As we're returning null from the inputEventProcessor, do not expect
        // anything to be displayed
        expect(textbox).toHaveTextContent('');
        expect(textbox).toHaveAttribute('data-content', '');
        expect(inputEventProcessor).toBeCalledTimes(1);
        expect(inputEventProcessor).toBeCalledWith(
            inputEvent,
            expect.anything(),
            expect.anything(),
        );
    });
});
