/*
Copyright 2026 New Vector Ltd.
Copyright 2022 The Matrix.org Foundation C.I.C.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

/**
 * Demo app — Phase 2 integration.
 *
 * Uses `WysiwygViewModel` (Phase 1) and wires it directly to the shared
 * `ComposerView` + `ComposerToolbarView` from `@element-hq/web-shared-components`.
 *
 * The contenteditable div is still owned by this component and passed as
 * `children` to `ComposerView` — the WASM model is attached via `vm.attach()`.
 */

import { type ReactElement, useState, useMemo, useRef, useEffect } from 'react';
import { ComposerView, ComposerToolbarView, useViewModel } from '@element-hq/web-shared-components';
import { TooltipProvider } from '@vector-im/compound-web';

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
import { type WysiwygEvent } from '../lib/types.js';
import { useTestCases } from '../lib/useTestCases/index.js';
import { refreshComposerView } from '../lib/dom.js';

const emojiSuggestions = new Map<string, string>([[':)', '🙂']]);

function App(): ReactElement {
    const [enterToSend, setEnterToSend] = useState(true);

    const vm = useMemo(() => {
        return new WysiwygViewModel({
            emojiSuggestions,
            inputEventProcessor: (e: WysiwygEvent, wysiwyg, _editor) => {
                if (e instanceof ClipboardEvent) return e;
                if (
                    !(e instanceof KeyboardEvent) &&
                    ((enterToSend && e.inputType === 'insertParagraph') ||
                        e.inputType === 'sendMessage')
                ) {
                    console.log(`SENDING MESSAGE HTML: ${wysiwyg.messageContent()}`);
                    wysiwyg.actions.clear();
                    return null;
                }
                return e;
            },
        });
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, []);

    const editorRef = useRef<HTMLDivElement | null>(null);
    const modelRef = useRef<HTMLDivElement | null>(null);

    useEffect(() => {
        const el = editorRef.current;
        if (!el) return;
        vm.attach(el);
        vm.init().catch(console.error);
        return () => {
            vm.detach();
        };
    }, [vm]);

    // Dispose the ViewModel on unmount
    useEffect(() => () => vm.dispose(), [vm]);

    // Wire the useTestCases hook into the ViewModel for test case tracing
    const { testRef, utilities: testUtilities } = useTestCases(editorRef, vm.composerModel);
    useEffect(() => {
        vm.setTestUtilities(testUtilities);
    }, [vm, testUtilities]);

    const onEnterToSendChanged = (): void => {
        setEnterToSend((prev) => !prev);
    };

    // Subscribe to the VM snapshot for the debug panel
    const snapshot = useViewModel(vm);

    // Update the Model DOM tree whenever the snapshot changes
    useEffect(() => {
        if (modelRef.current && vm.composerModel) {
            refreshComposerView(modelRef.current, vm.composerModel);
        }
    });

    return (
        <TooltipProvider>
        <div style={{
            display: 'flex',
            flexDirection: 'column',
            height: '100%',
            background: 'var(--cpd-color-bg-subtle-secondary)',
        }}>
            {/* Debug area — takes all available space above the composer */}
            <div style={{
                flex: '1 1 0',
                minHeight: 0,
                padding: 'var(--cpd-space-4x)',
                display: 'flex',
                flexDirection: 'column',
                gap: 'var(--cpd-space-3x)',
                color: 'var(--cpd-color-text-primary)',
            }}>
                {/* Model and Test case share space equally */}
                <div style={{ flex: '1 1 0', minHeight: 0, display: 'flex', flexDirection: 'column' }}>
                    <h2 style={debugHeadingStyle}>Model:</h2>
                    <div className="dom" ref={modelRef} style={{ flex: '1 1 0', minHeight: 0, overflow: 'auto' }} />
                </div>

                <div style={{ flex: '1 1 0', minHeight: 0, display: 'flex', flexDirection: 'column' }}>
                    <h2 style={debugHeadingStyle}>
                        Test case:{' '}
                        <button type="button" onClick={testUtilities.onResetTestCase}>
                            Start from here
                        </button>
                    </h2>
                    <div className="testCase" ref={testRef} style={{ flex: '1 1 0', minHeight: 0, overflow: 'auto' }} />
                </div>

                {/* Message Content — fixed height at the bottom of the debug area */}
                <div style={{ flexShrink: 0, display: 'flex', flexDirection: 'column' }}>
                    <h2 style={debugHeadingStyle}>Message Content (Matrix HTML):</h2>
                    <div className="testCase" style={{ maxHeight: '80px', overflow: 'auto' }}>
                        <pre style={{ margin: 0, whiteSpace: 'pre-wrap', wordBreak: 'break-word' }}>
                            {snapshot.messageContent ?? '(empty)'}
                        </pre>
                    </div>
                </div>
            </div>

            {/* Composer area — pinned to the bottom */}
            <div style={{
                position: 'relative',
                flexShrink: 0,
                background: 'var(--cpd-color-bg-canvas-default)',
                borderTop: '1px solid var(--cpd-color-border-interactive-secondary)',
                padding: 'var(--cpd-space-3x)',
                display: 'flex',
                flexDirection: 'column',
                gap: 'var(--cpd-space-2x)',
            }}>
                {/* Formatting toolbar from shared-components */}
                <ComposerToolbarView vm={vm} />

                {/* Composer pill shell from shared-components */}
                <ComposerView vm={vm}>
                    <div
                        ref={editorRef}
                        contentEditable
                        role="textbox"
                        aria-label="Message"
                        aria-multiline="true"
                        className="rte-content"
                        style={{
                            outline: 'none',
                            width: '100%',
                            minHeight: '1.5em',
                            font: 'var(--cpd-font-body-md-regular)',
                        }}
                    />
                </ComposerView>

                <div style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: 'var(--cpd-space-2x)',
                    font: 'var(--cpd-font-body-sm-regular)',
                    color: 'var(--cpd-color-text-secondary)',
                    padding: '0 var(--cpd-space-2x)',
                }}>
                    <input
                        type="checkbox"
                        id="enterToSend"
                        checked={enterToSend}
                        onChange={onEnterToSendChanged}
                    />
                    <label htmlFor="enterToSend">Enter to "send" (if unchecked, use Ctrl+Enter)</label>
                </div>
            </div>
        </div>
        </TooltipProvider>
    );
}

const debugHeadingStyle: React.CSSProperties = {
    margin: '0',
    font: 'var(--cpd-font-body-md-semibold)',
    color: 'var(--cpd-color-text-secondary)',
};

export default App;
