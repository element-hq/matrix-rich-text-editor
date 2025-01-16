/*
Copyright 2024 New Vector Ltd.
Copyright 2022 The Matrix.org Foundation C.I.C.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

/// <reference types="cypress" />

describe('Cut', () => {
    const editor = '.editor:not([disabled])[contenteditable="true"]';

    it('remove text that is cut to clipboard', { browser: 'electron' }, () => {
        cy.visit('/');
        cy.get(editor).wait(500);
        cy.get(editor).type('firstREMOVEME');
        cy.contains(editor, 'firstREMOVEME');

        cy.setSelection(editor, 5, 13);
        cy.document().invoke('execCommand', 'cut');

        // The clipboard should contain the text we cut
        cy.getSelection().should('equal', 'REMOVEME');

        // Clear the selection, so we really check that the cut is working,
        // instead of typing over the selected text
        cy.setSelection(editor, 5, 5);

        // Type something, because only when we do that do we reveal what was
        // really in the Rust model.
        cy.get(editor).type('last');
        cy.contains(editor, 'last');
        cy.get(editor).should('not.contain', 'REMOVEME');
    });
});
