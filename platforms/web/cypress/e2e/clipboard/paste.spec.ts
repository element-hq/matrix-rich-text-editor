/*
Copyright 2024 New Vector Ltd.
Copyright 2022 The Matrix.org Foundation C.I.C.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

/// <reference types="cypress" />

describe('Paste', () => {
    const editor = '.editor:not([disabled])[contenteditable="true"]';

    it(
        'should display pasted text after we type',
        { browser: 'electron' },
        () => {
            cy.visit('/');
            cy.get(editor).wait(500);
            cy.get(editor).type('BEFORE');
            cy.contains(editor, 'BEFORE'); // Wait for the typing to finish

            cy.window()
                .its('navigator.clipboard')
                .invoke('writeText', 'pasted');
            cy.get(editor).focus();
            cy.document().invoke('execCommand', 'paste');
            cy.contains(editor, 'BEFOREpasted');

            cy.get(editor).type('AFTER');
            cy.contains(editor, /^BEFOREpastedAFTER/);
        },
    );

    // Note: we used to test it 'should convert pasted newlines into BRs' but
    // the test was flakey, sometimes correctly showing text containing br tags,
    // and sometimes mysteriously showing converted into two divs.
});
