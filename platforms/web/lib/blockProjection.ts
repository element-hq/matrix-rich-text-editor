/*
Copyright 2026 The Matrix.org Foundation C.I.C.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

/**
 * TypeScript type declarations for the BlockProjection API returned from the
 * Rust WASM binding's `ComposerModel.get_block_projections()`.
 *
 * These mirror the Rust types in `crates/wysiwyg/src/block_projection.rs` and
 * the serialisation in `bindings/wysiwyg-wasm/src/lib.rs`.
 */

export interface AttributeSet {
    bold: boolean;
    italic: boolean;
    strikeThrough: boolean;
    underline: boolean;
    inlineCode: boolean;
    linkUrl: string | null;
}

export type InlineRunKind =
    | { type: 'text'; text: string; attributes: AttributeSet }
    | { type: 'mention'; url: string; displayText: string }
    | { type: 'lineBreak' };

export interface InlineRun {
    nodeId: string;
    startUtf16: number;
    endUtf16: number;
    kind: InlineRunKind;
}

export type BlockKind =
    | { type: 'paragraph' }
    | { type: 'quote' }
    | { type: 'codeBlock' }
    | { type: 'listItemOrdered'; depth: number }
    | { type: 'listItemUnordered'; depth: number }
    | { type: 'generic' };

export interface BlockProjection {
    blockId: string;
    kind: BlockKind;
    inQuote: boolean;
    startUtf16: number;
    endUtf16: number;
    inlineRuns: InlineRun[];
}

// ─── ProjectionRenderer ──────────────────────────────────────────────────────

/**
 * Render a `BlockProjection[]` into a `contenteditable` div.
 *
 * This replaces `replaceEditor()` (which set `editor.innerHTML` from an HTML
 * string) with a structured DOM build that guarantees 1:1 UTF-16 offset
 * correspondence between `editor.textContent` and the Rust model's offsets.
 *
 * Returns `committedText`: the plain text of the rendered content, which is
 * used by `reconcileNative()` to compute prefix/suffix diffs.
 */
export function renderProjections(
    projections: BlockProjection[],
    editor: HTMLElement,
): string {
    // Build a document fragment off-screen so we only touch the live DOM once.
    const frag = document.createDocumentFragment();

    // Track consecutive list items so we can group them in the correct
    // <ul>/<ol> container.
    let currentListEl: HTMLUListElement | HTMLOListElement | null = null;
    let currentListDepth = 0;
    let currentListOrdered = false;

    const flushList = (): void => {
        if (currentListEl) {
            frag.appendChild(currentListEl);
            currentListEl = null;
        }
    };

    for (let i = 0; i < projections.length; i++) {
        const block = projections[i];
        const kind = block.kind;

        if (kind.type === 'listItemOrdered' || kind.type === 'listItemUnordered') {
            const ordered = kind.type === 'listItemOrdered';
            const depth = kind.depth;

            // Start a new list container if the type or depth changes.
            if (
                !currentListEl ||
                currentListOrdered !== ordered ||
                currentListDepth !== depth
            ) {
                flushList();
                currentListEl = document.createElement(ordered ? 'ol' : 'ul');
                currentListOrdered = ordered;
                currentListDepth = depth;
                // Indent nested lists via margin.
                if (depth > 1) {
                    (currentListEl as HTMLElement).style.paddingLeft = `${(depth - 1) * 24}px`;
                }
            }

            const li = document.createElement('li');
            appendInlineRuns(block.inlineRuns, li);
            currentListEl.appendChild(li);
        } else {
            // Non-list block: flush any pending list container first.
            flushList();

            const blockEl = buildBlockElement(block);
            frag.appendChild(blockEl);
        }
    }

    // Flush any trailing list.
    flushList();

    // Replace editor contents in a single DOM operation.
    editor.innerHTML = '';
    editor.appendChild(frag);

    // Always ensure a trailing <br> so the contenteditable remains focusable
    // when empty.
    if (!editor.childNodes.length || projections.length === 0) {
        editor.appendChild(document.createElement('br'));
    }

    return editor.textContent ?? '';
}

// ─── Private helpers ─────────────────────────────────────────────────────────

function buildBlockElement(block: BlockProjection): HTMLElement {
    const kind = block.kind;

    switch (kind.type) {
        case 'paragraph': {
            const p = document.createElement('p');
            appendInlineRuns(block.inlineRuns, p);
            // Empty paragraph needs an &nbsp; placeholder so UIKit-style
            // cursor works (matches existing behaviour in the old pipeline).
            if (block.inlineRuns.length === 0) {
                p.appendChild(document.createTextNode('\u00a0'));
            }
            return p;
        }

        case 'codeBlock': {
            const pre = document.createElement('pre');
            const code = document.createElement('code');
            appendInlineRuns(block.inlineRuns, code);
            pre.appendChild(code);
            return pre;
        }

        case 'quote': {
            const bq = document.createElement('blockquote');
            appendInlineRuns(block.inlineRuns, bq);
            return bq;
        }

        case 'generic': {
            // Root-level inline content not wrapped in a block element.
            const div = document.createElement('div');
            appendInlineRuns(block.inlineRuns, div);
            return div;
        }

        default:
            // Unreachable for list items (handled by the caller), but
            // TypeScript needs the exhaustive fallback.
            throw new Error(`Unhandled block kind: ${JSON.stringify(kind)}`);
    }
}

/**
 * Append inline run DOM nodes into `parent`.
 */
function appendInlineRuns(runs: InlineRun[], parent: HTMLElement): void {
    for (const run of runs) {
        parent.appendChild(buildInlineNode(run));
    }
}

/**
 * Build a single inline DOM node from an `InlineRun`.
 */
function buildInlineNode(run: InlineRun): Node {
    const k = run.kind;

    if (k.type === 'lineBreak') {
        return document.createElement('br');
    }

    if (k.type === 'mention') {
        // Preserve the data-mention-type attribute so the existing mention
        // pipeline (suggestion.ts, useListeners) continues to work.
        const span = document.createElement('span');
        span.setAttribute('data-mention-type', 'user');
        span.setAttribute('href', k.url);
        span.contentEditable = 'false';
        span.textContent = k.displayText;
        return span;
    }

    // k.type === 'text'
    const { text, attributes } = k;
    let node: Node = document.createTextNode(text);

    // Wrap innermost to outermost: link > inlineCode > bold/italic/underline/strikethrough
    if (attributes.inlineCode) {
        const code = document.createElement('code');
        code.appendChild(node);
        node = code;
    }
    if (attributes.bold) {
        const strong = document.createElement('strong');
        strong.appendChild(node);
        node = strong;
    }
    if (attributes.italic) {
        const em = document.createElement('em');
        em.appendChild(node);
        node = em;
    }
    if (attributes.underline) {
        const u = document.createElement('u');
        u.appendChild(node);
        node = u;
    }
    if (attributes.strikeThrough) {
        const del = document.createElement('del');
        del.appendChild(node);
        node = del;
    }
    if (attributes.linkUrl) {
        const a = document.createElement('a');
        a.href = attributes.linkUrl;
        a.appendChild(node);
        node = a;
    }

    return node;
}
