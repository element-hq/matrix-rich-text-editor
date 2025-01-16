/*
Copyright 2024 New Vector Ltd.
Copyright 2022 The Matrix.org Foundation C.I.C.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';

import { Editor } from './testUtils/Editor';
import { select } from './testUtils/selection';

describe('delete content', () => {
    let clearButton: HTMLButtonElement;
    let textbox: HTMLDivElement;

    beforeEach(async () => {
        render(<Editor />);
        textbox = screen.getByRole('textbox');
        await waitFor(() =>
            expect(textbox).toHaveAttribute('contentEditable', 'true'),
        );
        clearButton = screen.getByRole('button', { name: 'clear' });
    });

    async function fillContent(): Promise<void> {
        fireEvent.input(textbox, {
            data: 'foo',
            inputType: 'insertText',
        });
        await userEvent.type(textbox, '{enter}');
        fireEvent.input(textbox, {
            data: 'bar',
            inputType: 'insertText',
        });
    }

    it('Should delete the content when using clear button', async () => {
        // When
        await fillContent();
        await userEvent.click(clearButton);

        // Then
        await waitFor(() => expect(textbox).toHaveTextContent(/^$/));
    });

    it('Should delete one character when using backspace', async () => {
        // When
        await fillContent();

        select(textbox, 2, 2);
        await userEvent.type(textbox, '{backspace}');

        // Then
        await waitFor(() => {
            expect(textbox).toContainHTML('<p>fo</p><p>bar</p>');
            expect(textbox).toHaveAttribute(
                'data-content',
                '<p>fo</p><p>bar</p>',
            );
        });
    });

    it('Should delete the selection when using backspace', async () => {
        // When
        await fillContent();

        select(textbox, 2, 5);
        await userEvent.type(textbox, '{backspace}');

        // Then
        await waitFor(() => {
            expect(textbox).toContainHTML('<p>foar</p>');
            expect(textbox).toHaveAttribute('data-content', '<p>foar</p>');
        });
    });

    it('Should delete one character when using delete', async () => {
        // When
        fireEvent.input(textbox, {
            data: 'foobar',
            inputType: 'insertText',
        });
        select(textbox, 3, 3);
        fireEvent.input(textbox, { inputType: 'deleteContentForward' });

        // Then
        await waitFor(() => {
            expect(textbox).toContainHTML('fooar');
            expect(textbox).toHaveAttribute('data-content', 'fooar');
        });
    });

    // eslint-disable-next-line max-len
    it('Should call Selection.modify with the correct arguments when using meta + backspace', async () => {
        // this test is the best we can do given the limitation that jsdom does
        // not implement Selection.modify
        const mockModify = vi.fn();
        Object.assign(Selection.prototype, { modify: mockModify });

        fireEvent.input(textbox, {
            data: 'foobar',
            inputType: 'insertText',
        });
        select(textbox, 6, 6);
        fireEvent.keyDown(textbox, { key: 'Backspace', metaKey: true });

        expect(mockModify).toHaveBeenCalledWith(
            'extend',
            'backward',
            'lineboundary',
        );

        mockModify.mockRestore();
    });
});
