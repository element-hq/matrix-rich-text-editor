/*
Copyright 2024 New Vector Ltd.
Copyright 2022 The Matrix.org Foundation C.I.C.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

import { SelectTuple, Tuple } from './types';

export function isSelectTuple(tuple: Tuple): tuple is SelectTuple {
    return tuple[0] === 'select';
}
