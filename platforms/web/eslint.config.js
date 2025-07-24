import matrixOrg from 'eslint-plugin-matrix-org';
import prettier from 'eslint-plugin-prettier';
import react from 'eslint-plugin-react';
import typescriptEslint from '@typescript-eslint/eslint-plugin';
import typescriptParser from '@typescript-eslint/parser';
import stylistic from '@stylistic/eslint-plugin';
import reactHooks from 'eslint-plugin-react-hooks';
import deprecate from 'eslint-plugin-deprecate';
import ts from 'typescript';

export default [
    {
        files: ['**/*.{js,jsx,ts,tsx}'],
        ignores: [
            'src/images/**',
            'generated/**',
            'dist/**',
            'dist-demo/**',
            'cypress.config.ts',
            'vite-env.d.ts',
            'vite.config.ts',
            'vite.demo.config.ts',
            'scripts/**',
            'cypress/**',
            'example-wysiwyg/**',
            'coverage/**',
            'eslint.config.js',
        ],
        plugins: {
            'matrix-org': matrixOrg,
            'prettier': prettier,
            'react': react,
            '@typescript-eslint': typescriptEslint,
            '@stylistic': stylistic,
            'react-hooks': reactHooks,
            'deprecate': deprecate,
        },
        languageOptions: {
            parser: typescriptParser,
            parserOptions: {
                projectService: true,
                tsconfigRootDir: import.meta.dirname,
            },
            globals: {
                // Browser globals
                window: 'readonly',
                document: 'readonly',
                console: 'readonly',
                // Node globals
                process: 'readonly',
                Buffer: 'readonly',
                __dirname: 'readonly',
                __filename: 'readonly',
            },
        },
        settings: {
            react: {
                version: 'detect',
            },
        },
        rules: {
            // Matrix.org plugin rules
            ...matrixOrg.configs.typescript.rules,
            ...matrixOrg.configs.react.rules,
            ...matrixOrg.configs.a11y.rules,

            // Custom overrides
            'react/jsx-curly-spacing': 'off',
            'new-cap': 'off',
            '@typescript-eslint/naming-convention': [
                'error',
                {
                    selector: ['variable', 'function'],
                    modifiers: ['private'],
                    format: ['camelCase'],
                    leadingUnderscore: 'allow',
                },
            ],
            'max-len': ['error', { code: 120, ignoreUrls: true }],
            'matrix-org/require-copyright-header': 'error',
            'prettier/prettier': 'error',
        },
    },
];
