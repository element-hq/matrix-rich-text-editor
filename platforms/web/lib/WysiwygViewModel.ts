/*
Copyright 2024 New Vector Ltd.
Copyright 2022 The Matrix.org Foundation C.I.C.

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

import {
    BaseViewModel,
    type ComposerToolbarViewModel,
    type ComposerViewModel,
    type ComposerSuggestion,
} from '@element-hq/web-shared-components';
import { initOnce } from './useComposerModel.js';
import { replaceEditor } from './dom.js';
import { processInput } from './composer.js';
import { mapSuggestion } from './suggestion.js';
import { createDefaultActionStates, mapToAllActionStates } from './useListeners/utils.js';
import { isClipboardEvent, isInputEvent } from './useListeners/assert.js';
import { handleKeyDown, handleSelectionChange, extractActionStates } from './useListeners/event.js';
import {
    type AllActionStates,
    type InputEventProcessor,
    type TraceAction,
    type WysiwygInputEvent,
} from './types.js';
import { type AllowedMentionAttributes, type FormatBlockEvent } from './useListeners/types.js';
import { type TestUtilities } from './useTestCases/types.js';

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
    onResetTestCase: () => undefined,
    setEditorHtml: (_content: string) => undefined,
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
export class WysiwygViewModel extends BaseViewModel<WysiwygViewModelSnapshot, WysiwygViewModelOptions>
    implements ComposerViewModel, ComposerToolbarViewModel
{
    private _composerModel: ComposerModel | null = null;
    private _suggestion: SuggestionPattern | null = null;
    private _plainTextContent = '';
    private _editor: HTMLElement | null = null;
    private _cleanup: (() => void) | null = null;
    private _testUtilities: TestUtilities = noopTestUtilities;

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
        await initOnce();

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
                    replaceEditor(this._editor, html, 0, html.length);
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
            replaceEditor(this._editor, html, 0, html.length);
            this._syncSnapshotFromModel();
        }
    }

    /** Remove all DOM event listeners. Call before disposing. */
    public detach(): void {
        this._cleanup?.();
        this._cleanup = null;
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

    public bold = (): void => this._sendActionEvent('formatBold');
    public italic = (): void => this._sendActionEvent('formatItalic');
    public strikeThrough = (): void => this._sendActionEvent('formatStrikeThrough');
    public underline = (): void => this._sendActionEvent('formatUnderline');
    public inlineCode = (): void => this._sendActionEvent('formatInlineCode');
    public codeBlock = (): void => this._sendActionEvent('insertCodeBlock');
    public quote = (): void => this._sendActionEvent('insertQuote');
    public orderedList = (): void => this._sendActionEvent('insertOrderedList');
    public unorderedList = (): void => this._sendActionEvent('insertUnorderedList');
    public indent = (): void => this._sendActionEvent('formatIndent');
    public unindent = (): void => this._sendActionEvent('formatOutdent');
    public undo = (): void => this._sendActionEvent('historyUndo');
    public redo = (): void => this._sendActionEvent('historyRedo');
    public clear = (): void => this._sendActionEvent('clear');
    public removeLinks = (): void => this._sendActionEvent('removeLinks');

    public insertText = (text: string): void =>
        this._sendActionEvent('insertText', text);

    public link = (url: string, text?: string): void =>
        this._sendActionEvent('insertLink', { url, text });

    public mention = (
        url: string,
        text: string,
        attributes: AllowedMentionAttributes,
    ): void => this._sendActionEvent('insertSuggestion', { url, text, attributes });

    public mentionAtRoom = (attributes: AllowedMentionAttributes): void =>
        this._sendActionEvent('insertAtRoomSuggestion', { attributes });

    public command = (text: string): void =>
        this._sendActionEvent('insertCommand', text);

    public emoji = (text: string): void =>
        this._sendActionEvent('insertEmoji', text);

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
            actionStates: res.actionStates ?? this.snapshot.current.actionStates,
            suggestion: mapped ? { ...mapped, type: mapped.type as ComposerSuggestion['type'], isOpen: true } : null,
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
            this._mergeCore({ actionStates: extractActionStates(menuStateUpdate) });
        }
    };

    /**
     * Set the content from an HTML string, replacing whatever is currently in the composer.
     */
    public setContentFromHtml = (html: string): void => {
        if (!this._composerModel || !this._editor) return;
        this._composerModel.set_content_from_html(html);
        const newHtml = this._composerModel.get_content_as_html();
        replaceEditor(this._editor, newHtml, 0, newHtml.length);
        this._syncSnapshotFromModel();
    };

    /**
     * Set the content from a Markdown string.
     */
    public setContentFromMarkdown = (markdown: string): void => {
        if (!this._composerModel || !this._editor) return;
        this._composerModel.set_content_from_markdown(markdown);
        const newHtml = this._composerModel.get_content_as_html();
        replaceEditor(this._editor, newHtml, 0, newHtml.length);
        this._syncSnapshotFromModel();
    };

    /** Returns the URL of the currently-selected link, or empty string. */
    public getLink = (): string =>
        this._composerModel?.get_link_action()?.edit_link?.url ?? '';

    // -------------------------------------------------------------------------
    // Private helpers
    // -------------------------------------------------------------------------

    /**
     * Dispatch a `wysiwygInput` custom event to the editor element so it is
     * caught by the `wysiwygInput` event listener in `_bindListeners`, which
     * in turn calls `_handleInput`. This keeps all ComposerModel mutations on
     * a single code path.
     */
    private _sendActionEvent(
        blockType: string,
        data?: unknown,
    ): void {
        if (!this._editor) return;
        this._editor.dispatchEvent(
            new CustomEvent('wysiwygInput', { detail: { blockType, data } }),
        );
    }

    /**
     * Wire up all necessary DOM event listeners to `editor` and store a cleanup
     * function in `this._cleanup`.
     */
    private _bindListeners(editor: HTMLElement): void {
        const onInput = (e: Event): void => {
            if (isInputEvent(e) && !e.isComposing) {
                this._handleInput(e as WysiwygInputEvent, editor);
            }
        };

        const onPaste = (e: ClipboardEvent | InputEvent): void => {
            const isSpecialSafariCase =
                isInputEvent(e) &&
                (e as InputEvent).inputType === 'insertFromPaste' &&
                (e as InputEvent).dataTransfer !== null;

            if (isClipboardEvent(e) || isSpecialSafariCase) {
                e.preventDefault();
                e.stopPropagation();
                this._handleInput(e as WysiwygInputEvent, editor);
            }
        };

        const onWysiwygInput = ((e: FormatBlockEvent): void => {
            this._handleInput(
                {
                    inputType: e.detail.blockType,
                    data: e.detail.data,
                } as WysiwygInputEvent,
                editor,
            );
        }) as EventListener;

        const onKeyDown = (e: KeyboardEvent): void => {
            if (!this._composerModel) return;
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
            onPaste(e as ClipboardEvent | InputEvent);
        };

        const onCompositionEnd = (e: CompositionEvent): void => {
            const inputEvent = new InputEvent('input', {
                data: e.data,
                inputType: 'insertCompositionText',
            });
            onInput(inputEvent);
        };

        const onSelectionChange = (): void => {
            if (!this._composerModel) return;
            try {
                const actionStates = handleSelectionChange(
                    editor,
                    this._composerModel,
                    this._testUtilities,
                );
                if (actionStates) {
                    this._mergeCore({ actionStates });
                }
                this._plainTextContent =
                    this._composerModel.get_content_as_plain_text();
            } catch {
                // Selection errors are non-fatal — ignore
            }
        };

        editor.addEventListener('input', onInput);
        editor.addEventListener('paste', onPaste as EventListener);
        editor.addEventListener('wysiwygInput', onWysiwygInput);
        editor.addEventListener('keydown', onKeyDown);
        editor.addEventListener('beforeinput', onBeforeInput as EventListener);
        editor.addEventListener('compositionend', onCompositionEnd);
        document.addEventListener('selectionchange', onSelectionChange);

        this._cleanup = (): void => {
            editor.removeEventListener('input', onInput);
            editor.removeEventListener('paste', onPaste as EventListener);
            editor.removeEventListener('wysiwygInput', onWysiwygInput);
            editor.removeEventListener('keydown', onKeyDown);
            editor.removeEventListener('beforeinput', onBeforeInput as EventListener);
            editor.removeEventListener('compositionend', onCompositionEnd);
            document.removeEventListener('selectionchange', onSelectionChange);
        };
    }

    /**
     * Process a single WysiwygInputEvent, apply the resulting ComposerUpdate,
     * and emit a snapshot change.
     */
    private _handleInput(e: WysiwygInputEvent, editor: HTMLElement): void {
        if (!this._composerModel) return;
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

                this._mergeCore({
                    content,
                    messageContent:
                        this._composerModel.get_content_as_message_html(),
                    actionStates,
                    suggestion: mapped ? { ...mapped, type: mapped.type as ComposerSuggestion['type'], isOpen: true } : null,
                });
                this._plainTextContent =
                    this._composerModel.get_content_as_plain_text();
            }
        } catch {
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
    ): {
        content?: string;
        actionStates: AllActionStates | null;
        suggestion: SuggestionPattern | null;
    } | undefined {
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
            replaceEditor(
                editor,
                repl.replacement_html,
                repl.start_utf16_codeunit,
                repl.end_utf16_codeunit,
            );
            editor.focus();
        }

        const menuStateUpdate = update.menu_state().update();
        const menuActionUpdate =
            update.menu_action().suggestion()?.suggestion_pattern;

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
    private _coreFields(partial: Partial<Pick<WysiwygViewModelSnapshot,
        'content' | 'messageContent' | 'actionStates' | 'suggestion' | 'isReady'
    >>): Partial<WysiwygViewModelSnapshot> {
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
    private _setCore(core: Pick<WysiwygViewModelSnapshot,
        'content' | 'messageContent' | 'actionStates' | 'suggestion' | 'isReady'
    >): void {
        const cur = this.snapshot.current;
        this.snapshot.set({
            ...cur,
            ...this._coreFields(core),
        } as WysiwygViewModelSnapshot);
    }

    /**
     * Merge a partial core snapshot update and derive view fields.
     */
    private _mergeCore(partial: Partial<Pick<WysiwygViewModelSnapshot,
        'content' | 'messageContent' | 'actionStates' | 'suggestion' | 'isReady'
    >>): void {
        this.snapshot.merge(this._coreFields(partial));
    }

    /**
     * Build a minimal `FormattingFunctions` shim that routes calls back
     * through `_sendActionEvent`. Used when passing formattingFunctions to
     * `handleKeyDown` (which only needs the shim for inputEventProcessor calls)
     * and to `processInput` (which needs `formattingFunctions.getLink`).
     */
    private _makeFormattingFunctionShim() {
        const send = (blockType: string, data?: unknown): void =>
            this._sendActionEvent(blockType, data);
        return {
            bold: () => send('formatBold'),
            italic: () => send('formatItalic'),
            strikeThrough: () => send('formatStrikeThrough'),
            underline: () => send('formatUnderline'),
            undo: () => send('historyUndo'),
            redo: () => send('historyRedo'),
            orderedList: () => send('insertOrderedList'),
            unorderedList: () => send('insertUnorderedList'),
            inlineCode: () => send('formatInlineCode'),
            clear: () => send('clear'),
            insertText: (text: string) => send('insertText', text),
            link: (url: string, text?: string) =>
                send('insertLink', { url, text }),
            removeLinks: () => send('removeLinks'),
            getLink: () => this.getLink(),
            codeBlock: () => send('insertCodeBlock'),
            quote: () => send('insertQuote'),
            indent: () => send('formatIndent'),
            unindent: () => send('formatOutdent'),
            mention: (
                url: string,
                text: string,
                attributes: AllowedMentionAttributes,
            ) => send('insertSuggestion', { url, text, attributes }),
            command: (text: string) => send('insertCommand', text),
            emoji: (text: string) => send('insertEmoji', text),
            mentionAtRoom: (attributes: AllowedMentionAttributes) =>
                send('insertAtRoomSuggestion', { attributes }),
        };
    }
}
