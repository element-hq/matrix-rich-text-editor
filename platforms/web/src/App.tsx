/*
Copyright 2024 New Vector Ltd.
Copyright 2022 The Matrix.org Foundation C.I.C.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

import { MouseEventHandler, ReactElement, useState } from 'react';

import { useWysiwyg } from '../lib/useWysiwyg';
import boldImage from './images/bold.svg';
import undoImage from './images/undo.svg';
import redoImage from './images/redo.svg';
import italicImage from './images/italic.svg';
import underlineImage from './images/underline.svg';
import strikeTroughImage from './images/strike_through.svg';
import listUnorderedImage from './images/list_unordered.svg';
import listOrderedImage from './images/list_ordered.svg';
import inlineCodeImage from './images/inline_code.svg';
import codeBlockImage from './images/code_block.svg';
import quoteImage from './images/quote.svg';
import indentImage from './images/indent.svg';
import unindentImage from './images/unindent.svg';
import { Wysiwyg, WysiwygEvent } from '../lib/types';

type ButtonProps = {
    onClick: MouseEventHandler<HTMLButtonElement>;
    imagePath: string;
    alt: string;
    state: 'enabled' | 'disabled' | 'reversed';
};

function Button({ onClick, imagePath, alt, state }: ButtonProps): ReactElement {
    const isReversed = state === 'reversed';
    const isDisabled = state === 'disabled';
    return (
        <button
            type="button"
            onClick={onClick}
            style={{
                ...(isReversed && { backgroundColor: 'lightgray' }),
                ...(isDisabled && { backgroundColor: 'firebrick' }),
            }}
        >
            <img alt={alt} src={imagePath} />
        </button>
    );
}
const emojiSuggestions = new Map<string, string>([[':)', '🙂']]);
function App(): ReactElement {
    const [enterToSend, setEnterToSend] = useState(true);

    const inputEventProcessor = (
        e: WysiwygEvent,
        wysiwyg: Wysiwyg,
    ): WysiwygEvent | null => {
        if (e instanceof ClipboardEvent) {
            return e;
        }

        if (
            !(e instanceof KeyboardEvent) &&
            ((enterToSend && e.inputType === 'insertParagraph') ||
                e.inputType === 'sendMessage')
        ) {
            if (debug.testRef.current) {
                debug.traceAction(null, 'send', `${wysiwyg.content()}`);
            }
            console.log(`SENDING MESSAGE HTML: ${wysiwyg.messageContent()}`);
            wysiwyg.actions.clear();
            return null;
        }

        return e;
    };

    const { ref, isWysiwygReady, actionStates, wysiwyg, debug, suggestion } =
        useWysiwyg({
            isAutoFocusEnabled: true,
            inputEventProcessor,
            emojiSuggestions: emojiSuggestions,
        });

    const onEnterToSendChanged = (): void => {
        setEnterToSend((prevValue) => !prevValue);
    };

    const isInList =
        actionStates.unorderedList === 'reversed' ||
        actionStates.orderedList === 'reversed';

    const commandExists = suggestion && suggestion.type === 'command';
    const mentionExists = suggestion && suggestion.type === 'mention';
    const shouldDisplayAtMention = mentionExists && suggestion.keyChar === '@';
    const shouldDisplayHashMention =
        mentionExists && suggestion.keyChar === '#';
    return (
        <div className="wrapper">
            <div>
                <div className="editor_container">
                    <div className="editor_toolbar">
                        <Button
                            onClick={wysiwyg.undo}
                            alt="undo"
                            imagePath={undoImage}
                            state={actionStates.undo}
                        />
                        <Button
                            onClick={wysiwyg.redo}
                            alt="redo"
                            imagePath={redoImage}
                            state={actionStates.redo}
                        />
                        <Button
                            onClick={wysiwyg.bold}
                            alt="bold"
                            imagePath={boldImage}
                            state={actionStates.bold}
                        />
                        <Button
                            onClick={wysiwyg.italic}
                            alt="italic"
                            imagePath={italicImage}
                            state={actionStates.italic}
                        />
                        <Button
                            onClick={wysiwyg.underline}
                            alt="underline"
                            imagePath={underlineImage}
                            state={actionStates.underline}
                        />
                        <Button
                            onClick={wysiwyg.strikeThrough}
                            alt="strike through"
                            imagePath={strikeTroughImage}
                            state={actionStates.strikeThrough}
                        />
                        <Button
                            onClick={wysiwyg.unorderedList}
                            alt="list unordered"
                            imagePath={listUnorderedImage}
                            state={actionStates.unorderedList}
                        />
                        <Button
                            onClick={wysiwyg.orderedList}
                            alt="list ordered"
                            imagePath={listOrderedImage}
                            state={actionStates.orderedList}
                        />
                        {isInList && (
                            <Button
                                onClick={wysiwyg.indent}
                                alt="indent"
                                imagePath={indentImage}
                                state={actionStates.indent}
                            />
                        )}
                        {isInList && (
                            <Button
                                onClick={wysiwyg.unindent}
                                alt="unindent"
                                imagePath={unindentImage}
                                state={actionStates.unindent}
                            />
                        )}
                        <Button
                            onClick={wysiwyg.quote}
                            alt="quote"
                            imagePath={quoteImage}
                            state={actionStates.quote}
                        />
                        <Button
                            onClick={wysiwyg.inlineCode}
                            alt="inline code"
                            imagePath={inlineCodeImage}
                            state={actionStates.inlineCode}
                        />
                        <Button
                            onClick={wysiwyg.codeBlock}
                            alt="code block"
                            imagePath={codeBlockImage}
                            state={actionStates.codeBlock}
                        />
                        <button
                            type="button"
                            onClick={(_e): void => wysiwyg.clear()}
                        >
                            clear
                        </button>
                        {shouldDisplayAtMention && (
                            <>
                                <button
                                    type="button"
                                    onClick={(_e): void =>
                                        wysiwyg.mention(
                                            'https://matrix.to/#/@alice_user:element.io',
                                            'Alice',
                                            new Map([
                                                [
                                                    'style',
                                                    'background-color:#d5f9d5',
                                                ],
                                                ['data-mention-type', 'user'],
                                            ]),
                                        )
                                    }
                                >
                                    Add User mention
                                </button>
                                <button
                                    type="button"
                                    onClick={(_e): void =>
                                        wysiwyg.mentionAtRoom(
                                            new Map([
                                                [
                                                    'style',
                                                    'background-color:#d5f9d5',
                                                ],
                                            ]),
                                        )
                                    }
                                >
                                    Add at-room mention
                                </button>
                            </>
                        )}
                        {shouldDisplayHashMention && (
                            <button
                                type="button"
                                onClick={(_e): void =>
                                    wysiwyg.mention(
                                        'https://matrix.to/#/#my_room:element.io',
                                        'My room',
                                        new Map([
                                            [
                                                'style',
                                                'background-color:#d5f9d5',
                                            ],
                                        ]),
                                    )
                                }
                            >
                                Add Room mention
                            </button>
                        )}
                        {commandExists && (
                            <button
                                type="button"
                                onClick={(_e): void =>
                                    wysiwyg.command('/spoiler')
                                }
                            >
                                Add /spoiler command
                            </button>
                        )}
                    </div>
                    <div
                        className="editor"
                        ref={ref}
                        contentEditable={isWysiwygReady}
                        role="textbox"
                    />
                </div>
                <div className="editor_options">
                    <input
                        type="checkbox"
                        id="enterToSend"
                        checked={enterToSend}
                        onChange={onEnterToSendChanged}
                    />
                    <label htmlFor="enterToSend">
                        Enter to "send" (if unchecked, use Ctrl+Enter)
                    </label>
                </div>
            </div>
            <h2>Model:</h2>
            <div className="dom" ref={debug.modelRef} />
            <h2>
                Test case:{' '}
                <button type="button" onClick={debug.resetTestCase}>
                    Start from here
                </button>
            </h2>
            <div className="testCase" ref={debug.testRef}>
                let mut model = cm("");
                <br />
                assert_eq!(tx(&amp;model), "");
            </div>
        </div>
    );
}

export default App;
