/*
Copyright 2024 New Vector Ltd.
Copyright 2022 The Matrix.org Foundation C.I.C.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

/// <reference types="cypress" />

import Chainable = Cypress.Chainable;

declare global {
    // eslint-disable-next-line @typescript-eslint/no-namespace
    namespace Cypress {
        interface Chainable {
            setSelection: typeof setSelection;
            getSelection: typeof getSelection;
        }
    }
}

const getSelection = (): Chainable<string> => {
    // eslint-disable-next-line max-len
    // From https://github.com/cypress-io/cypress/issues/2752#issuecomment-759746305
    return cy.window().then((win) => win.navigator.clipboard.readText());
};

const setSelection = (
    selector: string,
    start: number,
    end: number,
): Chainable<string> => {
    // eslint-disable-next-line max-len
    // From https://github.com/cypress-io/cypress/issues/2839#issuecomment-447012818
    cy.get(selector)
        .trigger('mousedown')
        .then(($el) => {
            const editorEl = $el[0];
            const document = editorEl.ownerDocument;
            const textNode = editorEl.firstChild;
            document
                .getSelection()
                .setBaseAndExtent(textNode, start, textNode, end);
        })
        .trigger('mouseup');
    return cy.document().trigger('selectionchange');
};

Cypress.Commands.add('setSelection', setSelection);
Cypress.Commands.add('getSelection', getSelection);

// Make this file a module
export {};
