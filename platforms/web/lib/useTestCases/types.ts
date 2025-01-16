/*
Copyright 2024 New Vector Ltd.
Copyright 2022 The Matrix.org Foundation C.I.C.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

import { useTestCases } from './useTestCases';

export type TestUtilities = ReturnType<typeof useTestCases>['utilities'];
export type SelectTuple = ['select', number, number];
export type Tuple =
    | SelectTuple
    | [string, (string | number)?, (string | number)?];
export type Actions = Array<Tuple>;
