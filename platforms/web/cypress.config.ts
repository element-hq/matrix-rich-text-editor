/*
Copyright 2024 New Vector Ltd.
Copyright 2022 The Matrix.org Foundation C.I.C.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

import { defineConfig } from 'cypress';

export default defineConfig({
    videoUploadOnPasses: false,
    //projectId: 'ppvnzg',
    experimentalInteractiveRunEvents: true,
    defaultCommandTimeout: 10000,
    chromeWebSecurity: false,
    e2e: {
        baseUrl: 'http://localhost:5173',
        specPattern: 'cypress/e2e/**/*.{ts,tsx}',
    },
});
