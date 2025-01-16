/*
Copyright 2024 New Vector Ltd.
Copyright 2022 The Matrix.org Foundation C.I.C.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

import {
    fireEvent,
    render,
    screen,
    waitFor,
    act,
} from '@testing-library/react';
import userEvent from '@testing-library/user-event';

import { Editor } from './testUtils/Editor';

describe('Undo redo', () => {
    let undo: HTMLButtonElement;
    let redo: HTMLButtonElement;
    let textbox: HTMLDivElement;

    beforeEach(async () => {
        render(<Editor />);
        textbox = screen.getByRole('textbox');
        await waitFor(() =>
            expect(textbox).toHaveAttribute('contentEditable', 'true'),
        );
        undo = screen.getByRole('button', { name: 'undo' });
        redo = screen.getByRole('button', { name: 'redo' });
    });

    test('Should be disabled by default', async () => {
        // Then
        await waitFor(() => {
            expect(undo).toHaveAttribute('data-state', 'disabled');
            expect(redo).toHaveAttribute('data-state', 'disabled');
        });
    });

    test('Should undo and redo content', async () => {
        // When
        fireEvent.input(textbox, {
            data: 'foo bar',
            inputType: 'insertText',
        });

        // Then
        await waitFor(() => {
            expect(undo).toHaveAttribute('data-state', 'enabled');
            expect(redo).toHaveAttribute('data-state', 'disabled');
        });

        // When
        await act(() => userEvent.click(undo));

        // Then
        await waitFor(() => {
            expect(textbox).toHaveTextContent(/^$/);
            expect(undo).toHaveAttribute('data-state', 'disabled');
            expect(redo).toHaveAttribute('data-state', 'enabled');
        });

        // When
        await act(() => userEvent.click(redo));

        // Then
        await waitFor(() => {
            expect(textbox).toHaveTextContent(/^foo bar$/);
            expect(undo).toHaveAttribute('data-state', 'enabled');
            expect(redo).toHaveAttribute('data-state', 'disabled');
        });
    });
});
