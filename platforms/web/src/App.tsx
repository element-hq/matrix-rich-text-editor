/*
Copyright 2026 New Vector Ltd.
Copyright 2022 The Matrix.org Foundation C.I.C.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

/**
 * Demo app.
 *
 * Drives `WysiwygViewModel` directly, the way an MVVM host would: the view
 * model is created for the lifetime of the component, the contenteditable is
 * bound to it with `useWysiwygEditor`, and everything rendered here comes
 * from `useViewModel(vm)`.
 */

import {
    type CSSProperties,
    type ReactElement,
    useEffect,
    useRef,
    useState,
} from 'react';
import {
    useCreateAutoDisposedViewModel,
    useViewModel,
} from '@element-hq/web-shared-components';

// Compound design tokens (CSS custom properties: --cpd-color-*, --cpd-space-*, etc.)
import '@vector-im/compound-design-tokens/assets/web/css/compound-design-tokens.css';
// Compound component styles
import '@vector-im/compound-web/dist/style.css';
// Inter font — the Element standard
import '@fontsource/inter/400.css';
import '@fontsource/inter/500.css';
import '@fontsource/inter/600.css';
import '@fontsource/inter/700.css';
import '@fontsource/inconsolata/400.css';

import './editor.css';
import { WysiwygViewModel } from '../lib/WysiwygViewModel.js';
import { useWysiwygEditor } from '../lib/useWysiwygEditor.js';
import {
    type ActionState,
    type ActionTypes,
    type WysiwygEvent,
} from '../lib/types.js';
import { type AllowedMentionAttributes } from '../lib/useListeners/types.js';
import { useTestCases } from '../lib/useTestCases/index.js';
import { refreshComposerView } from '../lib/dom.js';

const emojiSuggestions = new Map<string, string>([[':)', '🙂']]);

type ToolbarAction = Exclude<ActionTypes, 'link' | 'indent' | 'unindent'>;

const toolbarActions: Array<{ action: ToolbarAction; label: string }> = [
    { action: 'undo', label: 'undo' },
    { action: 'redo', label: 'redo' },
    { action: 'bold', label: 'bold' },
    { action: 'italic', label: 'italic' },
    { action: 'underline', label: 'underline' },
    { action: 'strikeThrough', label: 'strike through' },
    { action: 'unorderedList', label: 'list unordered' },
    { action: 'orderedList', label: 'list ordered' },
    { action: 'quote', label: 'quote' },
    { action: 'inlineCode', label: 'inline code' },
    { action: 'codeBlock', label: 'code block' },
];

type ButtonProps = {
    onClick: () => void;
    label: string;
    state?: ActionState;
};

function Button({ onClick, label, state }: ButtonProps): ReactElement {
    return (
        <button
            type="button"
            onClick={onClick}
            data-state={state}
            disabled={state === 'disabled'}
            style={{
                font: 'var(--cpd-font-body-sm-medium)',
                padding: 'var(--cpd-space-1x) var(--cpd-space-2x)',
                borderRadius: 'var(--cpd-space-1x)',
                border: '1px solid var(--cpd-color-border-interactive-secondary)',
                background:
                    state === 'reversed'
                        ? 'var(--cpd-color-bg-accent-rest)'
                        : 'var(--cpd-color-bg-canvas-default)',
                color:
                    state === 'reversed'
                        ? 'var(--cpd-color-text-on-solid-primary)'
                        : 'var(--cpd-color-text-primary)',
            }}
        >
            {label}
        </button>
    );
}

const mentionStyle: AllowedMentionAttributes = new Map([
    ['style', 'background-color:#d5f9d5'],
]);
const userMentionAttributes: AllowedMentionAttributes = new Map([
    ...mentionStyle,
    ['data-mention-type', 'user'],
]);

function App(): ReactElement {
    const [enterToSend, setEnterToSend] = useState(true);
    const editorRef = useRef<HTMLDivElement | null>(null);
    const modelRef = useRef<HTMLDivElement | null>(null);

    const vm = useCreateAutoDisposedViewModel(
        () => new WysiwygViewModel({ emojiSuggestions, editorRef }),
    );
    useWysiwygEditor(vm, editorRef);
    const snapshot = useViewModel(vm);

    // Debug panels: Rust test-case recorder and live model tree.
    const { testRef, utilities } = useTestCases(
        editorRef,
        snapshot.isReady ? vm.composerModel : null,
    );
    useEffect(() => {
        vm.setTestHooks(utilities);
    }, [vm, utilities]);
    useEffect(() => {
        if (snapshot.render) utilities.setEditorHtml(snapshot.render.html);
    }, [snapshot.render, utilities]);
    useEffect(() => {
        if (modelRef.current && vm.composerModel) {
            refreshComposerView(modelRef.current, vm.composerModel);
        }
    }, [snapshot, vm]);

    // "Send" on Enter (or Ctrl+Enter) by clearing the composer.
    useEffect(() => {
        vm.setInputEventProcessor(
            (e: WysiwygEvent, wysiwyg): WysiwygEvent | null => {
                if (e instanceof ClipboardEvent) return e;
                if (
                    !(e instanceof KeyboardEvent) &&
                    ((enterToSend && e.inputType === 'insertParagraph') ||
                        e.inputType === 'sendMessage')
                ) {
                    if (testRef.current) {
                        utilities.traceAction(null, 'send', wysiwyg.content());
                    }
                    console.log(
                        `SENDING MESSAGE HTML: ${wysiwyg.messageContent()}`,
                    );
                    wysiwyg.actions.clear();
                    return null;
                }
                return e;
            },
        );
    }, [vm, enterToSend, testRef, utilities]);

    const { actionStates, suggestion } = snapshot;
    const isInList =
        actionStates.unorderedList === 'reversed' ||
        actionStates.orderedList === 'reversed';
    const commandExists = suggestion?.type === 'command';
    const mentionExists = suggestion?.type === 'mention';
    const shouldDisplayAtMention = mentionExists && suggestion.keyChar === '@';
    const shouldDisplayHashMention =
        mentionExists && suggestion.keyChar === '#';

    return (
        <div style={appStyle}>
            {/* Debug area — takes all available space above the composer */}
            <div style={debugAreaStyle}>
                <div style={debugPanelStyle}>
                    <h2 style={debugHeadingStyle}>Model:</h2>
                    <div className="dom" ref={modelRef} style={scrollStyle} />
                </div>

                <div style={debugPanelStyle}>
                    <h2 style={debugHeadingStyle}>
                        Test case:{' '}
                        <button
                            type="button"
                            onClick={utilities.onResetTestCase}
                        >
                            Start from here
                        </button>
                    </h2>
                    <div
                        className="testCase"
                        ref={testRef}
                        style={scrollStyle}
                    />
                </div>

                <div style={{ flexShrink: 0 }}>
                    <h2 style={debugHeadingStyle}>
                        Message Content (Matrix HTML):
                    </h2>
                    <pre style={messageContentStyle}>
                        {snapshot.messageContent ?? '(empty)'}
                    </pre>
                </div>
            </div>

            {/* Composer area — pinned to the bottom */}
            <div style={composerAreaStyle}>
                <div style={toolbarStyle}>
                    {toolbarActions.map(({ action, label }) => (
                        <Button
                            key={action}
                            onClick={vm[action]}
                            label={label}
                            state={actionStates[action]}
                        />
                    ))}
                    {isInList && (
                        <Button
                            onClick={vm.indent}
                            label="indent"
                            state={actionStates.indent}
                        />
                    )}
                    {isInList && (
                        <Button
                            onClick={vm.unindent}
                            label="unindent"
                            state={actionStates.unindent}
                        />
                    )}
                    <Button onClick={vm.clear} label="clear" />
                    {shouldDisplayAtMention && (
                        <>
                            <Button
                                onClick={(): void =>
                                    vm.mention(
                                        'https://matrix.to/#/@alice_user:element.io',
                                        'Alice',
                                        new Map(userMentionAttributes),
                                    )
                                }
                                label="Add User mention"
                            />
                            <Button
                                onClick={(): void =>
                                    vm.mentionAtRoom(new Map(mentionStyle))
                                }
                                label="Add at-room mention"
                            />
                        </>
                    )}
                    {shouldDisplayHashMention && (
                        <Button
                            onClick={(): void =>
                                vm.mention(
                                    'https://matrix.to/#/#my_room:element.io',
                                    'My room',
                                    new Map(mentionStyle),
                                )
                            }
                            label="Add Room mention"
                        />
                    )}
                    {commandExists && (
                        <Button
                            onClick={(): void => vm.command('/spoiler')}
                            label="Add /spoiler command"
                        />
                    )}
                </div>

                <div
                    ref={editorRef}
                    contentEditable={snapshot.isReady}
                    role="textbox"
                    aria-label="Message"
                    aria-multiline="true"
                    className="editor"
                    style={editorStyle}
                />

                <div style={optionsStyle}>
                    <input
                        type="checkbox"
                        id="enterToSend"
                        checked={enterToSend}
                        onChange={(): void => setEnterToSend((v) => !v)}
                    />
                    <label htmlFor="enterToSend">
                        Enter to "send" (if unchecked, use Ctrl+Enter)
                    </label>
                </div>
            </div>
        </div>
    );
}

const appStyle: CSSProperties = {
    display: 'flex',
    flexDirection: 'column',
    height: '100%',
    background: 'var(--cpd-color-bg-subtle-secondary)',
};

const debugAreaStyle: CSSProperties = {
    flex: '1 1 0',
    minHeight: 0,
    padding: 'var(--cpd-space-4x)',
    display: 'flex',
    flexDirection: 'column',
    gap: 'var(--cpd-space-3x)',
    color: 'var(--cpd-color-text-primary)',
};

const debugPanelStyle: CSSProperties = {
    flex: '1 1 0',
    minHeight: 0,
    display: 'flex',
    flexDirection: 'column',
};

const scrollStyle: CSSProperties = {
    flex: '1 1 0',
    minHeight: 0,
    overflow: 'auto',
};

const debugHeadingStyle: CSSProperties = {
    margin: '0',
    font: 'var(--cpd-font-body-md-semibold)',
    color: 'var(--cpd-color-text-secondary)',
};

const messageContentStyle: CSSProperties = {
    margin: 0,
    maxHeight: '80px',
    overflow: 'auto',
    whiteSpace: 'pre-wrap',
    wordBreak: 'break-word',
    font: 'var(--cpd-font-body-sm-regular)',
};

const composerAreaStyle: CSSProperties = {
    position: 'relative',
    flexShrink: 0,
    background: 'var(--cpd-color-bg-canvas-default)',
    borderTop: '1px solid var(--cpd-color-border-interactive-secondary)',
    padding: 'var(--cpd-space-3x)',
    display: 'flex',
    flexDirection: 'column',
    gap: 'var(--cpd-space-2x)',
};

const toolbarStyle: CSSProperties = {
    display: 'flex',
    flexWrap: 'wrap',
    gap: 'var(--cpd-space-1x)',
};

const editorStyle: CSSProperties = {
    outline: 'none',
    width: '100%',
    minHeight: '1.5em',
    font: 'var(--cpd-font-body-md-regular)',
    border: '1px solid var(--cpd-color-border-interactive-secondary)',
    borderRadius: 'var(--cpd-space-3x)',
    padding: 'var(--cpd-space-2x) var(--cpd-space-3x)',
};

const optionsStyle: CSSProperties = {
    display: 'flex',
    alignItems: 'center',
    gap: 'var(--cpd-space-2x)',
    font: 'var(--cpd-font-body-sm-regular)',
    color: 'var(--cpd-color-text-secondary)',
    padding: '0 var(--cpd-space-2x)',
};

export default App;
