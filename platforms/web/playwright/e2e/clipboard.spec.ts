/*
Copyright 2026 Element Creations Ltd.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

import { test, expect, type Page } from '@playwright/test';

const editorSelector = '.editor:not([disabled])[contenteditable="true"]';

/**
 * Select a range of text within the editor by walking its text nodes.
 */
async function selectRange(
    page: Page,
    start: number,
    end: number,
): Promise<void> {
    await page.locator(editorSelector).evaluate(
        (el, { start, end }) => {
            const walker = document.createTreeWalker(el, NodeFilter.SHOW_TEXT);
            let offset = 0;
            let startNode: Node | null = null;
            let startOffset = 0;
            let endNode: Node | null = null;
            let endOffset = 0;

            while (walker.nextNode()) {
                const node = walker.currentNode;
                const len = node.textContent?.length ?? 0;
                if (!startNode && offset + len > start) {
                    startNode = node;
                    startOffset = start - offset;
                }
                if (!endNode && offset + len >= end) {
                    endNode = node;
                    endOffset = end - offset;
                    break;
                }
                offset += len;
            }

            if (startNode && endNode) {
                const sel = document.getSelection()!;
                sel.setBaseAndExtent(
                    startNode,
                    startOffset,
                    endNode,
                    endOffset,
                );
            }
        },
        { start, end },
    );
}

test.describe('Clipboard', () => {
    test.beforeEach(async ({ page }) => {
        await page.goto('/');
        const editor = page.locator(editorSelector);
        await editor.waitFor();
        await editor.click();
        await expect(editor).toBeFocused();
        // Warm up the WASM input pipeline — wait for the model to process each event
        await page.keyboard.type('x');
        await expect(editor).toContainText('x');
        await page.keyboard.press('Backspace');
        await expect(editor).not.toContainText('x');
    });

    test('cut removes text and places it on clipboard', async ({ page }) => {
        const editor = page.locator(editorSelector);
        await page.keyboard.type('firstREMOVEME');
        await expect(editor).toContainText('firstREMOVEME');

        await selectRange(page, 5, 13);
        await page.evaluate(() => document.execCommand('cut'));

        const clipboardText = await page.evaluate(() =>
            navigator.clipboard.readText(),
        );
        expect(clipboardText).toBe('REMOVEME');

        await page.keyboard.type('last');
        await expect(editor).toContainText('last');
        await expect(editor).not.toContainText('REMOVEME');
    });

    test('paste displays pasted text after typing', async ({ page }) => {
        const editor = page.locator(editorSelector);
        await page.keyboard.type('BEFORE');
        await expect(editor).toContainText('BEFORE');

        await page.evaluate(() => navigator.clipboard.writeText('pasted'));
        await page.keyboard.press('ControlOrMeta+v');
        await expect(editor).toContainText('BEFOREpasted');

        await page.keyboard.press('End');
        await page.keyboard.type('AFTER');
        await expect(editor).toContainText('BEFOREpastedAFTER');
    });

    test('paste displays pasted rich text after typing', async ({ page }) => {
        const editor = page.locator(editorSelector);
        await page.keyboard.type('BEFORE');
        await expect(editor).toContainText('BEFORE');

        await page.evaluate(async () => {
            const blob = new Blob(["<a href='https://matrix.org'>link</a>"], {
                type: 'text/html',
            });
            const item = new ClipboardItem({ 'text/html': blob });
            await navigator.clipboard.write([item]);
        });
        await page.keyboard.press('ControlOrMeta+v');
        await expect(editor).toContainText('BEFORElink');

        await page.keyboard.press('End');
        await page.keyboard.type('AFTER');
        await expect(editor).toContainText('BEFORElinkAFTER');
    });
});
