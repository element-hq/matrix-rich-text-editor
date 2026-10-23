/*
Copyright 2026 Element Creations Ltd.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

import {
    type ComposerModel,
    type ComposerUpdate,
    type SuggestionPattern,
    new_composer_model,
    new_composer_model_from_html,
} from '@vector-im/matrix-wysiwyg-wasm';
import { BaseViewModel } from '@element-hq/web-shared-components';

import { initOnce } from './initOnce.js';
import { processInput } from './composer.js';
import { mapSuggestion } from './suggestion.js';
import {
    createDefaultActionStates,
    mapToAllActionStates,
} from './useListeners/utils.js';
import {
    extractActionStates,
    getInputFromKeyDown,
} from './useListeners/event.js';
import {
    type AllActionStates,
    type BlockType,
    type FormattingFunctions,
    type InputEventProcessor,
    type MappedSuggestion,
    type TraceAction,
    type WysiwygInputEvent,
} from './types.js';
import { type AllowedMentionAttributes } from './useListeners/types.js';
import { type BlockProjection } from './blockProjection.js';

/**
 * Minimal, framework-agnostic shape of a ref to the contenteditable element.
 * Structurally compatible with React's `RefObject<HTMLElement | null>`.
 */
export interface EditorRef {
    readonly current: HTMLElement | null;
}

/** A selection range in the model's UTF-16 code units. */
export interface EditorSelection {
    start: number;
    end: number;
}

/**
 * What the editor element should display after the model last changed.
 * Produced by the view model, applied to the DOM by {@link bindEditor}.
 */
export interface EditorRender extends EditorSelection {
    /**
     * The document as a flat list of blocks and inline runs, carrying the
     * model's own UTF-16 offsets. This is what the editor binding renders.
     */
    blocks: BlockProjection[];
    /** The same content as HTML, for consumers that still need it. */
    html: string;
}

export interface WysiwygViewSnapshot {
    /** True once the WASM module has loaded and the composer model exists. */
    isReady: boolean;
    /** Current HTML content of the composer, as last produced by the model. */
    content: string | null;
    /** HTML content formatted for use as a Matrix message body. */
    messageContent: string | null;
    /** State of each formatting action (enabled / reversed / disabled). */
    actionStates: AllActionStates;
    /** Active autocomplete suggestion, if any. */
    suggestion: MappedSuggestion | null;
    /**
     * The DOM content the model expects the editor to show. A new object is
     * published every time the model rewrites the editor; the editor binding
     * applies it. `null` until the model is ready.
     */
    render: EditorRender | null;
    /**
     * Where the model moved the caret without rewriting the content. A new
     * object is published for each such move; `null` otherwise.
     */
    selection: EditorSelection | null;
}

/**
 * Actions exposed to views. Identical to the `FormattingFunctions` surface
 * that `useWysiwyg` has always returned, so existing consumers keep working.
 */
export type WysiwygViewActions = FormattingFunctions;

export interface WysiwygViewModelProps {
    /** Initial HTML content to populate the composer with. */
    initialContent?: string;
    /** Optional hook for intercepting or overriding input events. */
    inputEventProcessor?: InputEventProcessor;
    /** Map of custom emoji shortcodes to their unicode characters. */
    emojiSuggestions?: Map<string, string>;
    /**
     * Ref to the contenteditable element. The view model never reads or
     * writes the element itself; it is only passed through to the public
     * `inputEventProcessor` callback, which receives it by contract.
     */
    editorRef?: EditorRef;
}

/**
 * Hooks used by the demo app's test-case recorder. Production consumers do
 * not need to provide these.
 */
export interface WysiwygTestHooks {
    traceAction: TraceAction;
    getSelectionAccordingToActions?: () => [number, number];
}

const identityTrace: TraceAction = (update) => update;
const defaultTestHooks: WysiwygTestHooks = { traceAction: identityTrace };

/**
 * View model for the WYSIWYG composer.
 *
 * Owns the WASM `ComposerModel` and publishes an immutable snapshot of its
 * state. It does not touch the DOM: translating browser events into calls on
 * this class, and applying `snapshot.render` back to the contenteditable, is
 * the job of {@link bindEditor} (or the `useWysiwygEditor` hook).
 *
 * The WASM module is loaded and the model created as soon as the view model is
 * constructed; `snapshot.isReady` flips to true when that completes. Whoever
 * creates the view model must call `dispose()`.
 */
export class WysiwygViewModel
    extends BaseViewModel<WysiwygViewSnapshot, WysiwygViewModelProps>
    implements WysiwygViewActions
{
    private model: ComposerModel | null = null;
    private suggestionPattern: SuggestionPattern | null = null;
    private plainText = '';
    private initGeneration = 0;
    private testHooks: WysiwygTestHooks = defaultTestHooks;

    /** Resolves once the model has been created (or creation has failed). */
    public readonly ready: Promise<void>;

    public constructor(props: WysiwygViewModelProps = {}) {
        super(props, {
            isReady: false,
            content: null,
            messageContent: null,
            actionStates: createDefaultActionStates(),
            suggestion: null,
            render: null,
            selection: null,
        });
        this.disposables.track(() => this.freeModel());
        this.ready = this.initialise().catch((e: unknown) => {
            console.error('Failed to initialise the composer model', e);
        });
    }

    // -------------------------------------------------------------------------
    // Model access (debug tooling only)
    // -------------------------------------------------------------------------

    /** The underlying WASM model, for debug tooling. `null` until ready. */
    public get composerModel(): ComposerModel | null {
        return this.model;
    }

    // -------------------------------------------------------------------------
    // Prop updates
    // -------------------------------------------------------------------------

    public setInputEventProcessor(processor?: InputEventProcessor): void {
        this.props.inputEventProcessor = processor;
    }

    public setEmojiSuggestions(emojiSuggestions?: Map<string, string>): void {
        this.props.emojiSuggestions = emojiSuggestions;
        this.model?.set_custom_suggestion_patterns(
            Array.from(emojiSuggestions?.keys() ?? []),
        );
    }

    /** Wire in the demo app's test-case recorder. */
    public setTestHooks(hooks: WysiwygTestHooks): void {
        this.testHooks = hooks;
    }

    // -------------------------------------------------------------------------
    // Intents from the editor binding
    // -------------------------------------------------------------------------

    /**
     * Process an input event (native, synthetic or clipboard) against the
     * model and publish the resulting state.
     *
     * @returns whether the model handled the event. When it did not (an
     * input type the model has no handler for, or one the
     * `inputEventProcessor` suppressed), the browser's own edit is still in
     * the DOM and the editor binding reconciles it via {@link replaceTextIn}.
     */
    public handleInput(e: WysiwygInputEvent): boolean {
        if (!this.model) return false;
        try {
            const update = processInput(
                e,
                this.model,
                this.testHooks.traceAction,
                this,
                this.editor,
                this.suggestionPattern,
                this.props.inputEventProcessor,
                this.props.emojiSuggestions,
            );
            if (!update) return false;
            this.applyUpdate(update);
            return true;
        } catch {
            // The WASM model panicked; rebuild it with the last good content.
            void this.initialise(this.plainText);
            return true;
        }
    }

    /**
     * Replace the model text in `[start, end)` with `text`, in UTF-16 code
     * units. Used by the editor binding to push an edit the browser already
     * applied to the DOM (one the model did not intercept) into the model.
     * The full content is re-published so the DOM is rebuilt from the model.
     */
    public replaceTextIn(text: string, start: number, end: number): void {
        if (!this.model) return;
        try {
            this.testHooks.traceAction(
                this.model.replace_text_in(text, start, end),
                'replace_text_in',
                text,
                start,
            );
            this.syncFromModel();
        } catch {
            void this.initialise(this.plainText);
        }
    }

    /**
     * Resolve keyboard shortcuts to formatting actions, and give the
     * `inputEventProcessor` a chance to see other key presses.
     */
    public handleKeyDown(e: KeyboardEvent): void {
        if (!this.model) return;
        const blockType = getInputFromKeyDown(
            e,
            this.model,
            this,
            this.editor,
            this.props.inputEventProcessor,
        );
        if (blockType) {
            e.preventDefault();
            e.stopPropagation();
            this.dispatch(blockType);
        }
    }

    /**
     * Inform the model that the user moved the selection. Selections that
     * match the model's current selection are ignored.
     */
    public select(start: number, end: number): void {
        if (!this.model) return;
        const prevStart = this.model.selection_start();
        const prevEnd = this.model.selection_end();

        if (
            start === prevStart &&
            end === prevEnd &&
            this.matchesRecordedSelection(start, end)
        ) {
            return;
        }
        // Backwards selections are reported reversed by the browser.
        if (
            start === prevEnd &&
            end === prevStart &&
            this.matchesRecordedSelection(end, start)
        ) {
            return;
        }

        try {
            const update = this.model.select(start, end);
            this.testHooks.traceAction(null, 'select', start, end);
            const menuState = update.menu_state().update();
            if (menuState) {
                this.snapshot.merge({
                    actionStates: extractActionStates(menuState),
                });
            }
            this.plainText = this.model.get_content_as_plain_text();
        } catch {
            void this.initialise(this.plainText);
        }
    }

    // -------------------------------------------------------------------------
    // WysiwygViewActions
    // -------------------------------------------------------------------------

    public bold = (): void => this.dispatch('formatBold');
    public italic = (): void => this.dispatch('formatItalic');
    public strikeThrough = (): void => this.dispatch('formatStrikeThrough');
    public underline = (): void => this.dispatch('formatUnderline');
    public inlineCode = (): void => this.dispatch('formatInlineCode');
    public codeBlock = (): void => this.dispatch('insertCodeBlock');
    public quote = (): void => this.dispatch('insertQuote');
    public orderedList = (): void => this.dispatch('insertOrderedList');
    public unorderedList = (): void => this.dispatch('insertUnorderedList');
    public indent = (): void => this.dispatch('formatIndent');
    public unindent = (): void => this.dispatch('formatOutdent');
    public undo = (): void => this.dispatch('historyUndo');
    public redo = (): void => this.dispatch('historyRedo');
    public clear = (): void => this.dispatch('clear');
    public removeLinks = (): void => this.dispatch('removeLinks');

    public insertText = (text: string): void =>
        this.dispatch('insertText', text);

    public link = (url: string, text?: string): void =>
        this.dispatch('insertLink', { url, text });

    public mention = (
        url: string,
        text: string,
        attributes: AllowedMentionAttributes,
    ): void => this.dispatch('insertSuggestion', { url, text, attributes });

    public mentionAtRoom = (attributes: AllowedMentionAttributes): void =>
        this.dispatch('insertAtRoomSuggestion', { attributes });

    public command = (text: string): void =>
        this.dispatch('insertCommand', text);

    public emoji = (text: string): void => this.dispatch('insertEmoji', text);

    public getLink = (): string =>
        this.model?.get_link_action()?.edit_link?.url ?? '';

    // -------------------------------------------------------------------------
    // Content replacement
    // -------------------------------------------------------------------------

    /** Replace the whole composer content with the given HTML. */
    public setContentFromHtml = (html: string): void => {
        if (!this.model) return;
        this.testHooks.traceAction(
            this.model.set_content_from_html(html),
            'set_content_from_html',
            html,
        );
        this.syncFromModel();
    };

    /** Replace the whole composer content with the given Markdown. */
    public setContentFromMarkdown = (markdown: string): void => {
        if (!this.model) return;
        this.testHooks.traceAction(
            this.model.set_content_from_markdown(markdown),
            'set_content_from_markdown',
            markdown,
        );
        this.syncFromModel();
    };

    // -------------------------------------------------------------------------
    // Private
    // -------------------------------------------------------------------------

    private get editor(): HTMLElement | null {
        return this.props.editorRef?.current ?? null;
    }

    private dispatch(inputType: BlockType, data?: unknown): void {
        this.handleInput({ inputType, data } as WysiwygInputEvent);
    }

    /**
     * The demo app records a test case as a list of actions. When it does,
     * a selection only counts as unchanged if the recording agrees too.
     */
    private matchesRecordedSelection(start: number, end: number): boolean {
        const recorded = this.testHooks.getSelectionAccordingToActions?.();
        if (!recorded) return true;
        return recorded[0] === start && recorded[1] === end;
    }

    /**
     * Load the WASM module and create the model. Safe to call again to
     * recover from a WASM panic; a later call supersedes an earlier one that
     * is still awaiting the module.
     */
    private async initialise(plainText?: string): Promise<void> {
        const generation = ++this.initGeneration;
        await initOnce();
        if (this.isDisposed || generation !== this.initGeneration) return;

        this.freeModel();
        const { initialContent, emojiSuggestions } = this.props;
        let model: ComposerModel;
        if (initialContent) {
            try {
                model = new_composer_model_from_html(
                    initialContent,
                    0,
                    initialContent.length,
                );
            } catch {
                // Unparseable HTML: fall back to an empty composer.
                model = new_composer_model();
            }
        } else {
            model = new_composer_model();
        }
        if (plainText) model.replace_text(plainText);
        if (emojiSuggestions) {
            model.set_custom_suggestion_patterns(
                Array.from(emojiSuggestions.keys()),
            );
        }

        this.model = model;
        this.suggestionPattern = null;
        this.plainText = model.get_content_as_plain_text();
        this.snapshot.set({
            isReady: true,
            content: model.get_content_as_html(),
            messageContent: model.get_content_as_message_html(),
            actionStates: mapToAllActionStates(model.action_states()),
            suggestion: null,
            render: this.renderFromModel(),
            selection: null,
        });
    }

    /** A render instruction for the model's whole current content. */
    private renderFromModel(): EditorRender {
        const model = this.model!;
        return {
            html: model.get_content_as_html(),
            blocks: model.get_block_projections() as BlockProjection[],
            start: model.selection_start(),
            end: model.selection_end(),
        };
    }

    private freeModel(): void {
        try {
            this.model?.free();
        } catch {
            // A panicked model may refuse to free; nothing more to do.
        }
        this.model = null;
    }

    /** Publish the outcome of a `ComposerUpdate`. */
    private applyUpdate(update: ComposerUpdate): void {
        if (!this.model) return;
        const textUpdate = update.text_update();
        const replaceAll = textUpdate.replace_all;
        const select = textUpdate.select;
        const menuState = update.menu_state().update();
        const pattern =
            update.menu_action().suggestion()?.suggestion_pattern ?? null;

        const partial: Partial<WysiwygViewSnapshot> = {
            messageContent: this.model.get_content_as_message_html(),
        };
        if (pattern !== this.suggestionPattern) {
            this.suggestionPattern = pattern;
            partial.suggestion = mapSuggestion(pattern);
        }
        if (replaceAll) {
            partial.content = replaceAll.replacement_html;
            partial.render = {
                html: replaceAll.replacement_html,
                blocks: this.model.get_block_projections() as BlockProjection[],
                start: replaceAll.start_utf16_codeunit,
                end: replaceAll.end_utf16_codeunit,
            };
        } else if (select) {
            partial.selection = {
                start: select.start_utf16_codeunit,
                end: select.end_utf16_codeunit,
            };
        }
        if (menuState) {
            partial.actionStates = extractActionStates(menuState);
        }
        this.plainText = this.model.get_content_as_plain_text();
        this.snapshot.merge(partial);
    }

    /** Publish the model's full state after a wholesale content change. */
    private syncFromModel(): void {
        if (!this.model) return;
        this.suggestionPattern = null;
        this.plainText = this.model.get_content_as_plain_text();
        const render = this.renderFromModel();
        this.snapshot.merge({
            content: render.html,
            messageContent: this.model.get_content_as_message_html(),
            actionStates: mapToAllActionStates(this.model.action_states()),
            suggestion: null,
            render,
        });
    }
}
