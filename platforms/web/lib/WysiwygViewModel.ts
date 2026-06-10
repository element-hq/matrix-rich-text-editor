/*
Copyright 2026 Element Creations Ltd.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

// No React import — this is a framework-agnostic ViewModel
import {
    type ComposerModel,
    type ComposerUpdate,
    type SuggestionPattern,
    new_composer_model,
    new_composer_model_from_html,
} from '@vector-im/matrix-wysiwyg-wasm';

import { BaseViewModel } from '@element-hq/web-shared-components';

// Defined locally — will be re-exported from shared-components in a future
// version, but for now we keep the RTE standalone.
export interface ComposerSuggestion {
    type: 'mention' | 'command' | 'custom';
    keyChar: string;
    text: string;
    isOpen: boolean;
}
import { initOnce } from './useComposerModel.js';
import { getCurrentSelection, selectContent } from './dom.js';
import { processInput } from './composer.js';
import { mapSuggestion } from './suggestion.js';
import {
    createDefaultActionStates,
    mapToAllActionStates,
} from './useListeners/utils.js';
import { isClipboardEvent, isInputEvent } from './useListeners/assert.js';
import { handleKeyDown, extractActionStates } from './useListeners/event.js';
import {
    type AllActionStates,
    type FormattingFunctions,
    type InputEventProcessor,
    type TraceAction,
    type WysiwygInputEvent,
} from './types.js';
import { type AllowedMentionAttributes } from './useListeners/types.js';
import { type TestUtilities } from './useTestCases/types.js';

// ---------------------------------------------------------------------------
// Event Trace Logger
// ---------------------------------------------------------------------------

export interface TraceEntry {
    /** Monotonic high-resolution timestamp (ms) */
    t: number;
    /** Event category */
    cat: string;
    /** Human-readable detail */
    msg: string;
    /** Optional structured data */
    data?: Record<string, unknown>;
}

/**
 * Ring-buffer trace logger for debugging event ordering.
 * Exposed via `window.__RTE_TRACE` for Playwright inspection.
 */
export class TraceLog {
    private _entries: TraceEntry[] = [];
    private _maxEntries: number;
    private _t0: number;

    public constructor(maxEntries = 500) {
        this._maxEntries = maxEntries;
        this._t0 = typeof performance !== 'undefined' ? performance.now() : 0;
    }

    /** Record a trace event. */
    public log(cat: string, msg: string, data?: Record<string, unknown>): void {
        const t =
            typeof performance !== 'undefined'
                ? +(performance.now() - this._t0).toFixed(3)
                : 0;
        this._entries.push({ t, cat, msg, data });
        if (this._entries.length > this._maxEntries) {
            this._entries.shift();
        }
    }

    /** Return all entries (oldest first). */
    public entries(): TraceEntry[] {
        return this._entries;
    }

    /** Return entries as a formatted multi-line string for easy reading. */
    public dump(): string {
        return this._entries
            .map((e) => {
                const d = e.data ? ' ' + JSON.stringify(e.data) : '';
                return `[${e.t.toFixed(1)}ms] [${e.cat}] ${e.msg}${d}`;
            })
            .join('\n');
    }

    /** Clear all entries. */
    public clear(): void {
        this._entries = [];
    }
}

// ---------------------------------------------------------------------------
// Snapshot
// ---------------------------------------------------------------------------

export type WysiwygViewModelSnapshot = {
    /** Current HTML content of the editor */
    content: string | null;
    /** HTML content formatted for use as a Matrix message */
    messageContent: string | null;
    /** Formatting button states (bold/italic/etc.) — used by ComposerToolbarViewSnapshot */
    actionStates: AllActionStates;
    /** Current autocomplete suggestion (null = no suggestion active) */
    suggestion: ComposerSuggestion | null;
    /** True once the WASM module has loaded and the composer model is ready */
    isReady: boolean;

    // ── ComposerViewSnapshot fields ──────────────────────────────────────
    showToolbar: boolean;
    formattingStates: AllActionStates;
    canSend: boolean;
    placeholder: string;
    isEditing: boolean;
    isRichTextEnabled: boolean;
    encryptionState: 'encrypted' | 'notEncrypted' | 'onSignal';
    isDisabled: boolean;
    /** Whether all toolbar buttons should be disabled (ComposerToolbarViewSnapshot) */
    disabled: boolean;
};

// ---------------------------------------------------------------------------
// Options
// ---------------------------------------------------------------------------

export type WysiwygViewModelOptions = {
    /** Initial HTML content to populate the composer */
    initialContent?: string;
    /** Optional custom event processor for intercepting/overriding input events */
    inputEventProcessor?: InputEventProcessor;
    /** Map of custom emoji shortcodes to their unicode characters */
    emojiSuggestions?: Map<string, string>;
};

// ---------------------------------------------------------------------------
// Minimal no-op test utilities (test tracing not needed outside of test hooks)
// ---------------------------------------------------------------------------

const noopTrace: TraceAction = (update: ComposerUpdate | null) => update;

const noopTestUtilities = {
    traceAction: noopTrace,
    getSelectionAccordingToActions: (): [number, number] => [-1, -1],
    onResetTestCase: (): void => undefined,
    setEditorHtml: (_content: string): void => undefined,
};

// ---------------------------------------------------------------------------
// ViewModel
// ---------------------------------------------------------------------------

/**
 * Framework-agnostic ViewModel for the WYSIWYG composer.
 *
 * Conforms to the `ViewModel<T>` structural interface (getSnapshot + subscribe)
 * without importing anything from React. Consumers use `useViewModel(vm)` from
 * `@element-hq/web-shared-components` to drive React re-renders.
 *
 * Lifecycle:
 *   1. `new WysiwygViewModel(options)`
 *   2. `vm.attach(editorElement)` — called by the View once the contenteditable
 *      ref is available. Wires up all DOM event listeners.
 *   3. `await vm.init()` — loads WASM and initialises the ComposerModel.
 *   4. Use action methods (bold, italic, …) to mutate state.
 *   5. `vm.detach()` + `vm.dispose()` on unmount.
 */
export class WysiwygViewModel extends BaseViewModel<
    WysiwygViewModelSnapshot,
    WysiwygViewModelOptions
> {
    private _composerModel: ComposerModel | null = null;
    private _suggestion: SuggestionPattern | null = null;
    private _plainTextContent = '';
    private _editor: HTMLElement | null = null;
    private _cleanup: (() => void) | null = null;
    private _testUtilities: TestUtilities = noopTestUtilities;

    // ── Render guard ──
    // True while the ViewModel is mutating the DOM (innerHTML + selection).
    // Prevents selectionchange from feeding stale DOM state back into the model.
    private _isRendering = false;
    // After a render, the expected DOM selection. The first selectionchange
    // that matches this after the guard clears is an echo, not a user action.
    private _expectedSelection: [number, number] | null = null;
    // Monotonic counter incremented each time init() is called.
    // Used to discard stale async init completions (e.g. React StrictMode
    // double-invokes effects, causing two concurrent init() calls).
    private _initVersion = 0;

    // ── Event trace log ──
    public readonly trace = new TraceLog();

    /** Expose the underlying WASM model for debug tooling (demo app, tests). */
    public get composerModel(): ComposerModel | null {
        return this._composerModel;
    }

    /** Allow external test utilities (e.g. useTestCases) to hook into input tracing. */
    public setTestUtilities(utils: TestUtilities): void {
        this._testUtilities = utils;
    }

    public constructor(options: WysiwygViewModelOptions = {}) {
        const defaultActions = createDefaultActionStates();
        super(options, {
            content: null,
            messageContent: null,
            actionStates: defaultActions,
            suggestion: null,
            isReady: false,
            // ComposerViewSnapshot defaults
            showToolbar: true,
            formattingStates: defaultActions,
            canSend: false,
            placeholder: 'Send a message\u2026',
            isEditing: false,
            isRichTextEnabled: true,
            encryptionState: 'encrypted',
            isDisabled: true,
            // ComposerToolbarViewSnapshot default
            disabled: true,
        });
    }

    // -------------------------------------------------------------------------
    // Lifecycle
    // -------------------------------------------------------------------------

    /**
     * Attach the ViewModel to a contenteditable DOM element.
     * Call this before `init()` so that DOM updates during init are applied.
     */
    public attach(editor: HTMLElement): void {
        this._editor = editor;
        // Ensure the editor has at least one child so cursor can be placed
        if (!editor.childElementCount) {
            editor.appendChild(document.createElement('br'));
        }
        this._bindListeners(editor);
    }

    /**
     * Initialise the WASM module and create the ComposerModel.
     * Safe to call multiple times (idempotent via `initOnce`).
     */
    public async init(): Promise<void> {
        const myVersion = ++this._initVersion;
        this.trace.log('init', `starting init() v${myVersion}`);
        await initOnce();

        // After the async boundary, check whether a newer init() was started
        // (e.g. React StrictMode double-invokes effects).  If so, this one
        // is stale — let the newer call set up the model.
        if (myVersion !== this._initVersion) {
            this.trace.log(
                'init',
                `stale init v${myVersion} (current v${this._initVersion}) — skipped`,
            );
            return;
        }

        const { initialContent, emojiSuggestions } = this.props;
        let model: ComposerModel;

        if (initialContent) {
            try {
                model = new_composer_model_from_html(
                    initialContent,
                    0,
                    initialContent.length,
                );
                if (this._editor) {
                    const html = model.get_content_as_html();
                    this._renderToDOM(this._editor, html, 0, html.length);
                }
            } catch {
                // HTML parse failure — fall back to empty composer
                model = new_composer_model();
            }
        } else {
            model = new_composer_model();
        }

        if (emojiSuggestions) {
            model.set_custom_suggestion_patterns(
                Array.from(emojiSuggestions.keys()),
            );
        }

        this._composerModel = model;
        this._plainTextContent = model.get_content_as_plain_text();
        this.trace.log('init', 'model created', {
            html: model.get_content_as_html(),
            sel: [model.selection_start(), model.selection_end()],
        });

        this._setCore({
            content: model.get_content_as_html(),
            messageContent: model.get_content_as_message_html(),
            actionStates: mapToAllActionStates(model.action_states()),
            suggestion: null,
            isReady: true,
        });
    }

    /**
     * Re-initialise the model, optionally restoring plain-text content.
     * Called internally on WASM panic recovery.
     */
    public async reinit(plainTextContent?: string): Promise<void> {
        this.trace.log(
            'reinit',
            `plainTextContent=${plainTextContent?.slice(0, 50)}`,
        );
        this.detach();
        // Do NOT set this._composerModel = null here — TypeScript would narrow it to null
        // and lose the type after the async init() call below.
        this._suggestion = null;
        if (this._editor) {
            this.attach(this._editor);
        }
        await this.init();
        if (plainTextContent && this._composerModel && this._editor) {
            this._composerModel.replace_text(plainTextContent);
            const html = this._composerModel.get_content_as_html();
            this._renderToDOM(this._editor, html, 0, html.length);
            this._syncSnapshotFromModel();
        }
    }

    /** Remove all DOM event listeners. Call before disposing. */
    public detach(): void {
        this._cleanup?.();
        this._cleanup = null;
        this._editor = null;
    }

    /** Release all resources. The ViewModel should not be used after this. */
    public override dispose(): void {
        this.detach();
        this._editor = null;
        super.dispose();
    }

    // -------------------------------------------------------------------------
    // ComposerViewActions — consumed by ComposerView
    // -------------------------------------------------------------------------

    public onSend = (): void => {
        console.log(`SENDING: ${this.snapshot.current.messageContent}`);
        this.clear();
    };
    public onSaveEdit = (): void => undefined;
    public onCancelEdit = (): void => this.clear();
    public onToggleToolbar = (): void => undefined;

    // -------------------------------------------------------------------------
    // ComposerToolbarViewActions + ComposerViewActions formatting
    // -------------------------------------------------------------------------

    public onBold = (): void => this.bold();
    public onItalic = (): void => this.italic();
    public onUnderline = (): void => this.underline();
    public onStrikeThrough = (): void => this.strikeThrough();
    public onUnorderedList = (): void => this.unorderedList();
    public onOrderedList = (): void => this.orderedList();
    public onIndent = (): void => this.indent();
    public onUnindent = (): void => this.unindent();
    public onQuote = (): void => this.quote();
    public onInlineCode = (): void => this.inlineCode();
    public onCodeBlock = (): void => this.codeBlock();
    public onLink = (isEditing: boolean): void => {
        if (isEditing) {
            this.removeLinks();
        } else {
            const url = window.prompt('Enter URL:');
            if (url) this.link(url);
        }
    };

    // -------------------------------------------------------------------------
    // Formatting actions
    // -------------------------------------------------------------------------

    public bold = (): void => this._handleAction('formatBold');
    public italic = (): void => this._handleAction('formatItalic');
    public strikeThrough = (): void =>
        this._handleAction('formatStrikeThrough');
    public underline = (): void => this._handleAction('formatUnderline');
    public inlineCode = (): void => this._handleAction('formatInlineCode');
    public codeBlock = (): void => this._handleAction('insertCodeBlock');
    public quote = (): void => this._handleAction('insertQuote');
    public orderedList = (): void => this._handleAction('insertOrderedList');
    public unorderedList = (): void =>
        this._handleAction('insertUnorderedList');
    public indent = (): void => this._handleAction('formatIndent');
    public unindent = (): void => this._handleAction('formatOutdent');
    public undo = (): void => this._handleAction('historyUndo');
    public redo = (): void => this._handleAction('historyRedo');
    public clear = (): void => {
        this.trace.log('clear', 'clear() called');
        this._handleAction('clear');
    };
    public removeLinks = (): void => this._handleAction('removeLinks');

    public insertText = (text: string): void =>
        this._handleAction('insertText', text);

    public link = (url: string, text?: string): void =>
        this._handleAction('insertLink', { url, text });

    public mention = (
        url: string,
        text: string,
        attributes: AllowedMentionAttributes,
    ): void =>
        this._handleAction('insertSuggestion', { url, text, attributes });

    public mentionAtRoom = (attributes: AllowedMentionAttributes): void =>
        this._handleAction('insertAtRoomSuggestion', { attributes });

    public command = (text: string): void =>
        this._handleAction('insertCommand', text);

    public emoji = (text: string): void =>
        this._handleAction('insertEmoji', text);

    /**
     * Replace the entire composer content with `text` (plain text replacement).
     */
    public replaceText = (text: string): void => {
        if (!this._composerModel || !this._editor) return;
        const update = this._composerModel.replace_text(text);
        const res = this._applyUpdate(update, this._editor);
        this._suggestion = res.suggestion;
        const mapped = mapSuggestion(res.suggestion);
        this._mergeCore({
            content: res.content,
            messageContent: this._composerModel.get_content_as_message_html(),
            actionStates:
                res.actionStates ?? this.snapshot.current.actionStates,
            suggestion: mapped
                ? {
                      ...mapped,
                      type: mapped.type as ComposerSuggestion['type'],
                      isOpen: true,
                  }
                : null,
        });
    };

    /**
     * Set the selection range in the model (updates action states).
     */
    public select = (start: number, end: number): void => {
        if (!this._composerModel || !this._editor) return;
        const update = this._composerModel.select(start, end);
        const menuStateUpdate = update.menu_state().update();
        if (menuStateUpdate) {
            this._mergeCore({
                actionStates: extractActionStates(menuStateUpdate),
            });
        }
    };

    /**
     * Set the content from an HTML string, replacing whatever is currently in the composer.
     */
    public setContentFromHtml = (html: string): void => {
        if (!this._composerModel || !this._editor) return;
        this._composerModel.set_content_from_html(html);
        const newHtml = this._composerModel.get_content_as_html();
        this._renderToDOM(this._editor, newHtml, 0, newHtml.length);
        this._syncSnapshotFromModel();
    };

    /**
     * Set the content from a Markdown string.
     */
    public setContentFromMarkdown = (markdown: string): void => {
        if (!this._composerModel || !this._editor) return;
        this._composerModel.set_content_from_markdown(markdown);
        const newHtml = this._composerModel.get_content_as_html();
        this._renderToDOM(this._editor, newHtml, 0, newHtml.length);
        this._syncSnapshotFromModel();
    };

    /** Returns the URL of the currently-selected link, or empty string. */
    public getLink = (): string =>
        this._composerModel?.get_link_action()?.edit_link?.url ?? '';

    // -------------------------------------------------------------------------
    // Private helpers
    // -------------------------------------------------------------------------

    /**
     * Execute a formatting/editing action by constructing a synthetic
     * WysiwygInputEvent and running it through the standard _handleInput
     * pipeline. Called directly from public action methods — no DOM event
     * round-trip.
     */
    private _handleAction(blockType: string, data?: unknown): void {
        if (!this._editor) return;
        this._handleInput(
            { inputType: blockType, data } as WysiwygInputEvent,
            this._editor,
        );
    }

    /**
     * Wire up all necessary DOM event listeners to `editor` and store a cleanup
     * function in `this._cleanup`.
     */
    private _bindListeners(editor: HTMLElement): void {
        const onInput = (e: Event): void => {
            if (isInputEvent(e) && !e.isComposing) {
                const ie = e as InputEvent;
                this.trace.log('input', `inputType=${ie.inputType}`, {
                    data: ie.data,
                    isRendering: this._isRendering,
                });
                this._handleInput(ie as WysiwygInputEvent, editor);
            }
        };

        const onPaste = (e: ClipboardEvent | InputEvent): void => {
            const isSpecialSafariCase =
                isInputEvent(e) &&
                (e as InputEvent).inputType === 'insertFromPaste' &&
                (e as InputEvent).dataTransfer !== null;

            if (isClipboardEvent(e) || isSpecialSafariCase) {
                const cd = (e as ClipboardEvent).clipboardData;
                this.trace.log('paste', `type=${e.type}`, {
                    hasHtml: !!cd?.getData('text/html'),
                    plainText: cd?.getData('text/plain')?.slice(0, 50),
                    htmlText: cd?.getData('text/html')?.slice(0, 100),
                    isRendering: this._isRendering,
                    modelSel: this._composerModel
                        ? [
                              this._composerModel.selection_start(),
                              this._composerModel.selection_end(),
                          ]
                        : null,
                    modelHtml: this._composerModel?.get_content_as_html(),
                });
                e.preventDefault();
                e.stopPropagation();
                this._handleInput(e as WysiwygInputEvent, editor);
            }
        };

        const onKeyDown = (e: KeyboardEvent): void => {
            if (!this._composerModel) return;
            this.trace.log('keydown', `key=${e.key}`, {
                ctrl: e.ctrlKey,
                meta: e.metaKey,
                shift: e.shiftKey,
                isRendering: this._isRendering,
                modelHtml: this._composerModel.get_content_as_html(),
                modelSel: [
                    this._composerModel.selection_start(),
                    this._composerModel.selection_end(),
                ],
            });
            handleKeyDown(
                e,
                editor,
                this._composerModel,
                this._makeFormattingFunctionShim(),
                this.props.inputEventProcessor,
            );
        };

        const onBeforeInput = (e: InputEvent | ClipboardEvent): void => {
            if (isInputEvent(e) && (e as InputEvent).isComposing) return;
            this.trace.log(
                'beforeinput',
                `inputType=${isInputEvent(e) ? (e as InputEvent).inputType : 'clipboard'}`,
                {
                    isRendering: this._isRendering,
                },
            );
            onPaste(e as ClipboardEvent | InputEvent);
        };

        const onCompositionEnd = (e: CompositionEvent): void => {
            this.trace.log('compositionend', `data=${e.data}`);
            const inputEvent = new InputEvent('input', {
                data: e.data,
                inputType: 'insertCompositionText',
            });
            onInput(inputEvent);
        };

        const onSelectionChange = (): void => {
            // Always log, even if we're going to skip it
            const sel = document.getSelection();
            let domSel: [number, number] | null = null;
            try {
                if (sel && editor.contains(sel.anchorNode)) {
                    domSel = getCurrentSelection(editor, sel) as [
                        number,
                        number,
                    ];
                }
            } catch {
                /* ignore */
            }
            this.trace.log(
                'selectionchange',
                this._isRendering ? 'BLOCKED(rendering)' : 'processing',
                {
                    domSel,
                    isRendering: this._isRendering,
                    expectedSel: this._expectedSelection,
                    modelSel: this._composerModel
                        ? [
                              this._composerModel.selection_start(),
                              this._composerModel.selection_end(),
                          ]
                        : null,
                    editorHtml: editor.innerHTML?.slice(0, 200),
                    activeElement: document.activeElement?.tagName,
                    selInEditor: sel ? editor.contains(sel.anchorNode) : false,
                },
            );
            if (this._isRendering) return;
            this._handleUserSelectionChange();
        };

        editor.addEventListener('input', onInput);
        editor.addEventListener('paste', onPaste as EventListener);
        editor.addEventListener('keydown', onKeyDown);
        editor.addEventListener('beforeinput', onBeforeInput as EventListener);
        editor.addEventListener('compositionend', onCompositionEnd);
        document.addEventListener('selectionchange', onSelectionChange);

        // Debug-only listeners for full event visibility
        const onFocus = (): void => {
            this.trace.log('focus', 'editor focused', {
                modelHtml: this._composerModel?.get_content_as_html(),
                modelSel: this._composerModel
                    ? [
                          this._composerModel.selection_start(),
                          this._composerModel.selection_end(),
                      ]
                    : null,
            });
        };
        const onBlur = (): void => {
            this.trace.log('blur', 'editor blurred', {
                modelHtml: this._composerModel?.get_content_as_html(),
                modelSel: this._composerModel
                    ? [
                          this._composerModel.selection_start(),
                          this._composerModel.selection_end(),
                      ]
                    : null,
            });
        };
        const onClick = (): void => {
            this.trace.log('click', 'editor clicked', {
                modelHtml: this._composerModel?.get_content_as_html(),
                modelSel: this._composerModel
                    ? [
                          this._composerModel.selection_start(),
                          this._composerModel.selection_end(),
                      ]
                    : null,
                editorHtml: editor.innerHTML?.slice(0, 200),
            });
        };
        editor.addEventListener('focus', onFocus);
        editor.addEventListener('blur', onBlur);
        editor.addEventListener('click', onClick);

        this._cleanup = (): void => {
            editor.removeEventListener('input', onInput);
            editor.removeEventListener('paste', onPaste as EventListener);
            editor.removeEventListener('keydown', onKeyDown);
            editor.removeEventListener(
                'beforeinput',
                onBeforeInput as EventListener,
            );
            editor.removeEventListener('compositionend', onCompositionEnd);
            document.removeEventListener('selectionchange', onSelectionChange);
            editor.removeEventListener('focus', onFocus);
            editor.removeEventListener('blur', onBlur);
            editor.removeEventListener('click', onClick);
        };
    }

    /**
     * Process a single WysiwygInputEvent, apply the resulting ComposerUpdate,
     * and emit a snapshot change.
     */
    private _handleInput(e: WysiwygInputEvent, editor: HTMLElement): void {
        if (!this._composerModel) return;
        const inputType = (e as { inputType?: string }).inputType ?? 'unknown';
        this.trace.log('handleInput', `inputType=${inputType}`, {
            modelSelBefore: [
                this._composerModel.selection_start(),
                this._composerModel.selection_end(),
            ],
            modelHtmlBefore: this._composerModel.get_content_as_html(),
        });
        try {
            const res = this._processWysiwygInput(e, editor);
            if (res) {
                const content =
                    res.content !== undefined
                        ? res.content
                        : this.snapshot.current.content;
                const actionStates =
                    res.actionStates ?? this.snapshot.current.actionStates;
                this._suggestion = res.suggestion;
                const mapped = mapSuggestion(this._suggestion);

                this.trace.log('handleInput.result', `inputType=${inputType}`, {
                    newContent: content?.slice(0, 200),
                    modelSelAfter: [
                        this._composerModel.selection_start(),
                        this._composerModel.selection_end(),
                    ],
                    editorHtmlAfter: editor.innerHTML?.slice(0, 200),
                });

                this._mergeCore({
                    content,
                    messageContent:
                        this._composerModel.get_content_as_message_html(),
                    actionStates,
                    suggestion: mapped
                        ? {
                              ...mapped,
                              type: mapped.type as ComposerSuggestion['type'],
                              isOpen: true,
                          }
                        : null,
                });
                this._plainTextContent =
                    this._composerModel.get_content_as_plain_text();
            } else {
                this.trace.log(
                    'handleInput.noop',
                    `inputType=${inputType} returned undefined`,
                );
            }
        } catch (err) {
            this.trace.log(
                'handleInput.error',
                `inputType=${inputType} ERROR: ${err}`,
            );
            // Attempt recovery with last known plain text
            void this.reinit(this._plainTextContent);
        }
    }

    /**
     * Delegate to the existing `processInput` function from `composer.ts`,
     * then apply any DOM mutations required by the ComposerUpdate.
     */
    private _processWysiwygInput(
        e: WysiwygInputEvent,
        editor: HTMLElement,
    ):
        | {
              content?: string;
              actionStates: AllActionStates | null;
              suggestion: SuggestionPattern | null;
          }
        | undefined {
        const update = processInput(
            e,
            this._composerModel!,
            this._testUtilities.traceAction,
            this._makeFormattingFunctionShim(),
            editor,
            this._suggestion,
            this.props.inputEventProcessor,
            this.props.emojiSuggestions,
        );

        if (!update) return undefined;
        return this._applyUpdate(update, editor);
    }

    /**
     * Apply a ComposerUpdate to the DOM (replace editor content if needed) and
     * return the extracted state changes without modifying `this._snapshot`.
     */
    private _applyUpdate(
        update: ComposerUpdate,
        editor: HTMLElement,
    ): {
        content?: string;
        actionStates: AllActionStates | null;
        suggestion: SuggestionPattern | null;
    } {
        const repl = update.text_update().replace_all;
        if (repl) {
            this.trace.log(
                'applyUpdate',
                `replace_all sel=[${repl.start_utf16_codeunit},${repl.end_utf16_codeunit}]`,
                {
                    html: repl.replacement_html.slice(0, 200),
                },
            );
            this._renderToDOM(
                editor,
                repl.replacement_html,
                repl.start_utf16_codeunit,
                repl.end_utf16_codeunit,
            );
        } else {
            this.trace.log('applyUpdate', 'no replace_all (keep DOM)');
        }

        const menuStateUpdate = update.menu_state().update();
        const menuActionUpdate = update
            .menu_action()
            .suggestion()?.suggestion_pattern;

        const actionStates = menuStateUpdate
            ? extractActionStates(menuStateUpdate)
            : null;
        const suggestion = menuActionUpdate ?? null;

        return {
            content: repl?.replacement_html,
            actionStates,
            suggestion,
        };
    }

    /**
     * Render model state to the DOM inside a guarded window.
     * All selectionchange events during this window are ignored.
     */
    private _renderToDOM(
        editor: HTMLElement,
        html: string,
        start: number,
        end: number,
    ): void {
        this.trace.log('renderToDOM.enter', `sel=[${start},${end}]`, {
            html: html.slice(0, 200),
            wasRendering: this._isRendering,
            prevExpected: this._expectedSelection,
        });
        this._isRendering = true;
        this._expectedSelection = [start, end];

        editor.innerHTML = html + '<br />';
        selectContent(editor, start, end);

        const needsFocus = document.activeElement !== editor;
        if (needsFocus) {
            editor.focus();
        }

        this.trace.log(
            'renderToDOM.done',
            `sel=[${start},${end}] needsFocus=${needsFocus}`,
            {
                editorHtml: editor.innerHTML.slice(0, 200),
            },
        );

        // Clear the guard after the browser has flushed pending selection events.
        // rAF fires after microtasks but before the next frame's event processing,
        // covering all observed selectionchange delivery timing in Chrome.
        requestAnimationFrame(() => {
            this.trace.log('renderToDOM.rAF', 'clearing _isRendering', {
                expectedSel: this._expectedSelection,
            });
            this._isRendering = false;
        });
    }

    /**
     * Handle a selectionchange event that passed the render guard.
     * Applies echo detection and dedup before forwarding to the model.
     */
    private _handleUserSelectionChange(): void {
        if (!this._composerModel || !this._editor) return;
        try {
            const [start, end] = getCurrentSelection(
                this._editor,
                document.getSelection(),
            );

            // Echo detection: first selectionchange after a render that matches
            // what we set is not a user action.
            if (this._expectedSelection) {
                const [expStart, expEnd] = this._expectedSelection;
                if (start === expStart && end === expEnd) {
                    this.trace.log(
                        'selChange.echoHit',
                        `sel=[${start},${end}] matches expected — skipped`,
                    );
                    this._expectedSelection = null;
                    return;
                }
                this.trace.log(
                    'selChange.echoMiss',
                    `sel=[${start},${end}] expected=[${expStart},${expEnd}] — NOT an echo`,
                );
                this._expectedSelection = null;
            }

            // Dedup: ignore if model already has this selection
            const prevStart = this._composerModel.selection_start();
            const prevEnd = this._composerModel.selection_end();
            if (start === prevStart && end === prevEnd) {
                this.trace.log(
                    'selChange.dedup',
                    `sel=[${start},${end}] matches model — skipped`,
                );
                return;
            }

            // Also ignore reversed duplicates (backwards selections)
            if (start === prevEnd && end === prevStart) {
                this.trace.log(
                    'selChange.dedupRev',
                    `sel=[${start},${end}] reverse matches model — skipped`,
                );
                return;
            }

            this.trace.log(
                'selChange.SELECT',
                `sel=[${start},${end}] model was=[${prevStart},${prevEnd}]`,
                {
                    editorHtml: this._editor.innerHTML?.slice(0, 200),
                },
            );
            const update = this._composerModel.select(start, end);
            this._testUtilities.traceAction(null, 'select', start, end);

            const menuStateUpdate = update.menu_state().update();
            if (menuStateUpdate) {
                this._mergeCore({
                    actionStates: extractActionStates(menuStateUpdate),
                });
            }
            this._plainTextContent =
                this._composerModel.get_content_as_plain_text();
        } catch {
            // Selection errors are non-fatal — ignore
        }
    }

    /**
     * Synchronise the snapshot from the composer model state without DOM
     * selection changes (used after setContentFromHtml / setContentFromMarkdown).
     */
    private _syncSnapshotFromModel(): void {
        if (!this._composerModel) return;
        this._mergeCore({
            content: this._composerModel.get_content_as_html(),
            messageContent: this._composerModel.get_content_as_message_html(),
            actionStates: mapToAllActionStates(
                this._composerModel.action_states(),
            ),
        });
        this._plainTextContent =
            this._composerModel.get_content_as_plain_text();
    }

    /**
     * Core snapshot fields (the WASM-centric fields). This type is the
     * subset that the rest of the class mutates directly.
     */
    private _coreFields(
        partial: Partial<
            Pick<
                WysiwygViewModelSnapshot,
                | 'content'
                | 'messageContent'
                | 'actionStates'
                | 'suggestion'
                | 'isReady'
            >
        >,
    ): Partial<WysiwygViewModelSnapshot> {
        // After merging the core partial into the current snapshot, derive
        // the ComposerView / ComposerToolbar fields from the result.
        const cur = this.snapshot.current;
        const merged = { ...cur, ...partial };
        return {
            ...partial,
            // ComposerViewSnapshot derived fields
            formattingStates: merged.actionStates,
            canSend: merged.isReady && !!merged.messageContent,
            isDisabled: !merged.isReady,
            suggestion: merged.suggestion,
            // ComposerToolbarViewSnapshot derived fields
            disabled: !merged.isReady,
        };
    }

    /**
     * Replace the entire core snapshot and derive view fields.
     */
    private _setCore(
        core: Pick<
            WysiwygViewModelSnapshot,
            | 'content'
            | 'messageContent'
            | 'actionStates'
            | 'suggestion'
            | 'isReady'
        >,
    ): void {
        const cur = this.snapshot.current;
        this.snapshot.set({
            ...cur,
            ...this._coreFields(core),
        } as WysiwygViewModelSnapshot);
    }

    /**
     * Merge a partial core snapshot update and derive view fields.
     */
    private _mergeCore(
        partial: Partial<
            Pick<
                WysiwygViewModelSnapshot,
                | 'content'
                | 'messageContent'
                | 'actionStates'
                | 'suggestion'
                | 'isReady'
            >
        >,
    ): void {
        this.snapshot.merge(this._coreFields(partial));
    }

    /**
     * Build a `FormattingFunctions` shim that delegates to the ViewModel's
     * public action methods. Used when passing formattingFunctions to
     * `handleKeyDown` (which needs it for the `inputEventProcessor` callback)
     * and to `processInput` (which needs `formattingFunctions.getLink`).
     */
    private _makeFormattingFunctionShim(): FormattingFunctions {
        return {
            bold: this.bold,
            italic: this.italic,
            strikeThrough: this.strikeThrough,
            underline: this.underline,
            undo: this.undo,
            redo: this.redo,
            orderedList: this.orderedList,
            unorderedList: this.unorderedList,
            inlineCode: this.inlineCode,
            clear: this.clear,
            insertText: this.insertText,
            link: this.link,
            removeLinks: this.removeLinks,
            getLink: this.getLink,
            codeBlock: this.codeBlock,
            quote: this.quote,
            indent: this.indent,
            unindent: this.unindent,
            mention: this.mention,
            command: this.command,
            emoji: this.emoji,
            mentionAtRoom: this.mentionAtRoom,
        };
    }
}
