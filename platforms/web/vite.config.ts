/*
Copyright 2024 New Vector Ltd.
Copyright 2022 The Matrix.org Foundation C.I.C.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import { resolve } from 'node:path';
import dts from 'vite-plugin-dts';

// https://vitejs.dev/config/
export default defineConfig({
    define: {
        // counterpart (used by compound-web) reads process.env.NODE_ENV at runtime;
        // Vite doesn't provide a process polyfill in the browser, so define it here.
        'process.env.NODE_ENV': JSON.stringify(process.env.NODE_ENV ?? 'development'),
    },
    plugins: [
        react(),
        dts({
            include: [
                'lib/useWysiwyg.ts',
                'lib/useComposerModel.ts',
                'lib/conversion.ts',
                'lib/types.ts',
                'lib/constants.ts',
                'lib/useListeners/types.ts',
                'lib/useTestCases/types.ts',
                'lib/WysiwygViewModel.ts',
            ],
            rollupTypes: true,
            copyDtsFiles: true,
            beforeWriteFile: async (filePath, content) => {
                // Hack generated types to make all exported
                content = content.replace(/^declare/gm, 'export declare');
                return { filePath, content };
            },
        }),
    ],
    server: {
        fs: {
            // Allow serving files from the git root to access the wasm in bindings dir
            // Also allow the linked shared-components source outside this repo
            allow: ['../..', '../../../../element-web/packages/shared-components'],
        },
    },
    resolve: {
        alias: {
            '@element-hq/web-shared-components': resolve(
                __dirname,
                '../../../element-web/packages/shared-components/src/index.ts',
            ),
            // Ensure deps imported by the linked shared-components source resolve
            // from this repo's node_modules, not from the element-web directory.
            'react-resizable-panels': resolve(__dirname, 'node_modules/react-resizable-panels'),
            // Deduplicate React — shared-components source (loaded via alias from
            // element-web) must resolve the same React instance as the app.
            'react': resolve(__dirname, 'node_modules/react'),
            'react-dom': resolve(__dirname, 'node_modules/react-dom'),
            'react/jsx-runtime': resolve(__dirname, 'node_modules/react/jsx-runtime'),
            'react/jsx-dev-runtime': resolve(__dirname, 'node_modules/react/jsx-dev-runtime'),
        },
    },
    test: {
        globals: true,
        environment: 'jsdom',
        setupFiles: 'test.setup.ts',
        exclude: ['playwright/e2e/**', 'node_modules/**'],
        includeSource: ['lib/**/*.{ts,tsx}'],
        coverage: {
            provider: 'v8',
            all: true,
            include: ['lib/**/*.{ts,tsx}'],
            exclude: [
                'lib/testUtils/**/*.{ts,tsx}',
                'lib/**/*test.{ts,tsx}',
                'lib/**/*.d.ts',
                'lib/**/types.ts',
            ],
            reporter: ['text', 'lcov'],
        },
        reporters: ['default', 'vitest-sonar-reporter'],
        outputFile: 'coverage/sonar-report.xml',
        onConsoleLog: (log) => {
            if (log.includes('wasm')) return false;
        },
    },
    build: {
        lib: {
            entry: resolve(__dirname, 'lib/useWysiwyg.ts'),
            name: 'Matrix wysiwyg',
            // the proper extensions will be added
            fileName: 'matrix-wysiwyg',
        },
        rollupOptions: {
            // make sure to externalize deps that shouldn't be bundled
            // into your library
            external: ['react', 'react-dom', '@vector-im/matrix-wysiwyg-wasm'],
            output: {
                // Provide global variables to use in the UMD build
                // for externalized deps
                globals: {
                    'react': 'React',
                    'react-dom': 'ReactDOM',
                },
            },
        },
    },
});
