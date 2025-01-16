/*
Copyright 2024 New Vector Ltd.
Copyright 2022 The Matrix.org Foundation C.I.C.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

// rust generated bindings
import {
    ComposerModel,
    ComposerUpdate,
    // eslint-disable-next-line camelcase
    new_composer_model_from_html,
} from '@vector-im/matrix-wysiwyg-wasm';

import { getCurrentSelection } from '../dom';
import { TraceAction } from '../types';
import { isSelectTuple } from './assert';
import { Actions } from './types';

export function traceAction(
    testNode: HTMLElement | null,
    actions: Actions,
    composerModel: ComposerModel | null,
): TraceAction {
    return (
        update: ComposerUpdate | null,
        name: string,
        value1?: string | number,
        value2?: string | number,
    ) => {
        if (!testNode || !composerModel) {
            return update;
        }

        if (value2 !== undefined) {
            console.debug(`composer_model.${name}(${value1}, ${value2})`);
        } else if (value1 !== undefined) {
            console.debug(`composer_model.${name}(${value1})`);
        } else {
            console.debug(`composer_model.${name}()`);
        }

        actions.push([name, value1, value2]);

        updateTestCase(testNode, composerModel, update, actions);

        return update;
    };
}

export function getSelectionAccordingToActions(actions: Actions) {
    return (): [number, number] => {
        for (let i = actions.length - 1; i >= 0; i--) {
            const action = actions[i];
            if (isSelectTuple(action)) {
                return [action[1], action[2]];
            }
        }
        return [-1, -1];
    };
}

function updateTestCase(
    testNode: HTMLElement,
    composerModel: ComposerModel,
    update: ComposerUpdate | null,
    actions: Actions,
): void {
    // let html = editor.innerHTML;
    if (update) {
        // TODO: if (replacement_html !== html) SHOW AN ERROR?
        // TODO: handle other types of update (not just replace_all)
        update.text_update();
        //    html = update.text_update().replace_all?.replacement_html;
    }

    testNode.innerText = generateTestCase(
        actions,
        composerModel.to_example_format(),
    );

    testNode.scrollTo(0, testNode.scrollHeight - testNode.clientHeight);
}

export function generateTestCase(actions: Actions, html: string): string {
    let ret = '';

    function add(
        name: string,
        value1?: string | number,
        value2?: string | number,
    ): void {
        if (name === 'select') {
            ret +=
                'model.select(' +
                `Location::from(${value1}), ` +
                `Location::from(${value2}));\n`;
        } else if (value2 !== undefined) {
            ret += `model.${name}(${value1 ?? ''}, ${value2});\n`;
        } else if (name === 'replace_text') {
            ret += `model.${name}("${value1 ?? ''}");\n`;
        } else if (name === 'send') {
            ret += `// Send: ${value1 ?? ''}\n`;
        } else {
            ret += `model.${name}(${value1 ?? ''});\n`;
        }
    }

    function start(): void {
        const text = addSelection(collected, selection[0], selection[1]);
        ret += `let mut model = cm("${text}");\n`;
    }

    let lastName: string | null = null;
    let isCollectingMode = true;
    let collected = '';
    let selection = [0, 0];
    for (const action of actions) {
        const [name, value1, value2] = action;
        if (isCollectingMode) {
            if (name === 'replace_text') {
                collected += escapeHtml(value1);
            } else if (isSelectTuple(action)) {
                selection = [action[1], action[2]];
            } else {
                isCollectingMode = false;
                start();
                add(name, value1, value2);
            }
        } else if (lastName === 'select' && name === 'select') {
            const nl = ret.lastIndexOf('\n', ret.length - 2);
            if (nl > -1) {
                ret = ret.substring(0, nl) + '\n';
                add(name, value1, value2);
            }
        } else {
            add(name, value1, value2);
        }
        lastName = name;
    }

    if (isCollectingMode) {
        start();
    }

    ret += `assert_eq!(tx(&model), "${html}");\n`;

    return ret;
}

function addSelection(text: string, start: number, end: number): string {
    // In the original wysiwyg js, the function is called with one parameter but
    // the TS definition requires 3 params
    // new_composer_model_from_html(text)
    const model = new_composer_model_from_html(text, -1, -1);
    model.select(start, end);
    return model.to_example_format();
}

export function resetTestCase(
    editor: HTMLElement,
    testNode: HTMLElement,
    composerModel: ComposerModel,
    html: string,
): Actions {
    const [start, end] = getCurrentSelection(editor, document.getSelection());
    const actions: Actions = [
        ['replace_text', html],
        ['select', start, end],
    ];
    updateTestCase(testNode, composerModel, null, actions);
    return actions;
}

export function escapeHtml(html: string | number | undefined): string {
    if (!html) {
        return '';
    }
    const p = document.createElement('p');
    p.appendChild(document.createTextNode(html.toString()));
    return p.innerHTML;
}
