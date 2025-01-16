/*
Copyright 2024 New Vector Ltd.
Copyright 2022 The Matrix.org Foundation C.I.C.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

import { ACTION_TYPES } from '../constants';
import { ActionTypes, ActionState, AllActionStates } from '../types';

export function createDefaultActionStates(): AllActionStates {
    return ACTION_TYPES.reduce<AllActionStates>((acc, action) => {
        acc[action] = 'enabled';
        return acc;
    }, {} as AllActionStates);
}

/**
 * Convert a Map<string, string> containing title-case strings like:
 * "Bold": "Enabled"
 * to a AllActionStates record with entries like:
 * bold: enabled
 * @param {Map<string, string>} actionStatesMap Map to convert
 * @returns {AllActionStates}
 */
export function mapToAllActionStates(
    actionStatesMap: Map<string, string>,
): AllActionStates {
    const ret = {} as AllActionStates;
    for (const [key, value] of actionStatesMap) {
        ret[
            (key.substring(0, 1).toLowerCase() +
                key.substring(1)) as ActionTypes
        ] = value.toLowerCase() as ActionState;
    }
    return ret as AllActionStates;
}
