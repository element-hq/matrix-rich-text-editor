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

/**
 * Dump the WASM model trace (.testCase div) to the test console for CI debugging.
 */
async function dumpModelTrace(page: Page, label: string): Promise<void> {
    const trace = await page
        .locator('.testCase')
        .textContent()
        .catch(() => null);
    // Use console.error so it shows in the test attachment even when the test passes
    console.error(`[${label}] WASM trace:\n${trace ?? '(not found)'}`);
}

/**
 * Paste plain text into the editor by dispatching a ClipboardEvent directly.
 * This avoids writing to the system clipboard and pressing Ctrl+V, which can
 * race with the WASM selectionchange handler in headless CI Chrome.
 */
async function pastePlainText(page: Page, text: string): Promise<void> {
    await page.locator(editorSelector).evaluate((el, t) => {
        const dt = new DataTransfer();
        dt.setData('text/plain', t);
        el.dispatchEvent(
            new ClipboardEvent('paste', {
                clipboardData: dt,
                bubbles: true,
                cancelable: true,
            }),
        );
    }, text);
}

/**
 * Paste rich text into the editor by dispatching a ClipboardEvent directly.
 * This avoids writing to the system clipboard and pressing Ctrl+V, which can
 * race with the WASM selectionchange handler in headless CI Chrome.
 */
async function pasteRichText(page: Page, html: string): Promise<void> {
    await page.locator(editorSelector).evaluate((el, h) => {
        const dt = new DataTransfer();
        dt.setData('text/html', h);
        dt.setData('text/plain', '');
        el.dispatchEvent(
            new ClipboardEvent('paste', {
                clipboardData: dt,
                bubbles: true,
                cancelable: true,
            }),
        );
    }, html);
}

test.describe('Clipboard', () => {
    test.beforeEach(async ({ page }) => {
        await page.goto('/');
        const editor = page.locator(editorSelector);
        await editor.waitFor();
        await editor.click();
        await expect(editor).toBeFocused();
        // Verify the WASM input pipeline is ready before running each test
        await page.keyboard.insertText('x');
        await expect(editor).toContainText('x');
        await page.keyboard.press('Backspace');
        await expect(editor).not.toContainText('x');
    });

    test('cut removes text and places it on clipboard', async ({ page }) => {
        const editor = page.locator(editorSelector);
        await page.keyboard.insertText('firstREMOVEME');
        await expect(editor).toContainText('firstREMOVEME');

        await selectRange(page, 5, 13);
        await page.evaluate(() => document.execCommand('cut'));

        const clipboardText = await page.evaluate(() =>
            navigator.clipboard.readText(),
        );
        expect(clipboardText).toBe('REMOVEME');

        await page.keyboard.insertText('last');
        await expect(editor).toContainText('last');
        await expect(editor).not.toContainText('REMOVEME');
    });

    test('paste displays pasted text after typing', async ({ page }) => {
        const editor = page.locator(editorSelector);
        await page.keyboard.insertText('BEFORE');
        await expect(editor).toContainText('BEFORE');

        await pastePlainText(page, 'pasted');
        await dumpModelTrace(page, 'after plain paste');
        await expect(editor).toContainText('BEFOREpasted');

        await page.keyboard.insertText('AFTER');
        await dumpModelTrace(page, 'after AFTER insert');
        await expect(editor).toContainText('BEFOREpastedAFTER');
    });

    test('paste displays pasted rich text after typing', async ({ page }) => {
        const editor = page.locator(editorSelector);
        await page.keyboard.insertText('BEFORE');
        await expect(editor).toContainText('BEFORE');

        await pasteRichText(page, "<a href='https://matrix.org'>link</a>");
        await dumpModelTrace(page, 'after rich paste');
        await expect(editor).toContainText('BEFORElink');

        await page.keyboard.insertText('AFTER');
        await dumpModelTrace(page, 'after AFTER insert (rich)');
        await expect(editor).toContainText('BEFORElinkAFTER');
    });
});
