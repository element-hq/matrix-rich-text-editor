/*
Copyright 2026 Element Creations Ltd.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
    testDir: './playwright/e2e',
    outputDir: './playwright/test-results',
    fullyParallel: true,
    forbidOnly: !!process.env.CI,
    retries: process.env.CI ? 2 : 0,
    // The WASM-backed contenteditable drops key events under heavy concurrent load,
    // and there are only 3 tests, so serial execution is the right trade-off.
    workers: 1,
    reporter: process.env.CI
        ? [['html', { outputFolder: 'playwright/html-report', open: 'never' }], ['github']]
        : [['html', { outputFolder: 'playwright/html-report' }]],
    use: {
        baseURL: 'http://localhost:5173',
        video: 'retain-on-failure',
        trace: 'on-first-retry',
    },
    projects: [
        {
            name: 'chromium',
            use: {
                ...devices['Desktop Chrome'],
                permissions: ['clipboard-read', 'clipboard-write'],
            },
        },
    ],
    webServer: {
        command: 'yarn dev',
        url: 'http://localhost:5173',
        reuseExistingServer: !process.env.CI,
    },
});
