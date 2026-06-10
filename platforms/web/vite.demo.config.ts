/*
Copyright 2024 New Vector Ltd.
Copyright 2022 The Matrix.org Foundation C.I.C.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';
import { resolve } from 'node:path';

const __dirname = new URL('.', import.meta.url).pathname;

export default defineConfig({
    plugins: [react()],
    base: '',
    define: {
        'process.env.NODE_ENV': JSON.stringify(process.env.NODE_ENV ?? 'development'),
    },
    server: {
        fs: {
            // Allow serving files from the workspace root, node_modules, and
            // the linked element-web shared-components source
            allow: [
                resolve(__dirname, '../..'),              // matrix-rich-text-editor root
                resolve(__dirname, '../../../element-web'), // linked shared-components source
            ],
        },
    },
    resolve: {
        alias: {
            '@element-hq/web-shared-components': resolve(
                __dirname,
                '../../../element-web/packages/shared-components/src/index.ts',
            ),
            'react-resizable-panels': resolve(__dirname, 'node_modules/react-resizable-panels'),
            'react': resolve(__dirname, 'node_modules/react'),
            'react-dom': resolve(__dirname, 'node_modules/react-dom'),
            'react/jsx-runtime': resolve(__dirname, 'node_modules/react/jsx-runtime'),
            'react/jsx-dev-runtime': resolve(__dirname, 'node_modules/react/jsx-dev-runtime'),
        },
    },
    build: {
        rollupOptions: {
            output: {
                dir: 'dist-demo',
            },
        },
    },
});
