/*
Copyright 2024 New Vector Ltd.
Copyright 2022 The Matrix.org Foundation C.I.C.

SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
Please see LICENSE in the repository root for full details.
*/

// rust generated bindings
import {
    // eslint-disable-next-line camelcase
    new_composer_model,
} from '@vector-im/matrix-wysiwyg-wasm';

import { initOnce } from './useComposerModel.js';

const NEWLINE_CHAR = '\n';

// In plain text, markdown newlines (displays '\' character followed by
// a newline character) will be represented as \n for setting in the composer.
// If we have a trailing newline character, we trim it off so that the cursor
// will always be at the last character of the string, not on a new line in the
// composer.
export const markdownToPlain = (markdown: string): string => {
    let plainText = markdown;
    if (plainText.endsWith(NEWLINE_CHAR)) {
        plainText = plainText.slice(0, -1);
    }
    return plainText.replaceAll(/\\/g, '');
};

export async function richToPlain(
    richText: string,
    inMessageFormat: boolean,
): Promise<string> {
    if (richText.length === 0) {
        return '';
    }

    // this function could be called before initialising the WASM
    // so we need to try to initialise
    await initOnce();

    // the rich text in the web model is html so set the model with it
    const model = new_composer_model();
    model.set_content_from_html(richText);

    // get the markdown in either composer or message format as required
    const markdown = inMessageFormat
        ? model.get_content_as_message_markdown()
        : model.get_content_as_markdown();

    const plainText = markdownToPlain(markdown);

    return plainText;
}

export async function plainToRich(
    plainText: string,
    inMessageFormat: boolean,
): Promise<string> {
    if (plainText.length === 0) {
        return '';
    }

    // this function could be called before initialising the WASM
    // so we need to try to initialise
    await initOnce();

    // convert the plain text into markdown so that we can use it to
    // set the model
    const markdown = plainTextInnerHtmlToMarkdown(plainText);

    // set the model and return the rich text
    const model = new_composer_model();
    model.set_content_from_markdown(markdown);

    return inMessageFormat
        ? model.get_content_as_message_html()
        : model.get_content_as_html();
}

/*
The reason for requiring this function requires it's own explanation, so here it is.
When manipulating a content editable div in a browser, as we do for the plain text version
of the composer in element web, there is a limited subset of html that the composer can contain.
Currently, the innerHTML of the plain text composer can only contain:
  - text with `\n` line separation if `shift + enter` is used to insert the linebreak
    - in this case, inserting a newline after a single word will result in `word\n\n`, then
      subsequent typing will replace the final `\n` to give `word\nanother word`
  - text with <div> separation if `cmd + enter to send` is enabled and `enter` is used to insert
    the linebreak
    - in this case, inserting a newline inserts `<div><br></div>`, and then subsequent typing 
      replaces the <br> tag with the new content
  - mentions (ie <a> tags with special attributes) which can be at the top level, or nested inside
    a div 
What we need to do is to get this input into a good shape for the markdown parser in the rust model. 
Because of some of the intricacies of how text content is parsed when you use `.innerHTML` vs `.innerText`
we do it manually so that we can extract:
  - text content from any text nodes exactly as the user has written it, so that there is no escaping
    of html entities like < or &
  - mentions in their pure html form so that they can be passed through as valid html, as the mentions
    in the plain text composer can be parsed into mentions inside the rust model
*/
export function plainTextInnerHtmlToMarkdown(innerHtml: string): string {
    // Parse the innerHtml into a DOM and treat the `body` as the `composer`
    const { body: composer } = new DOMParser().parseFromString(
        innerHtml,
        'text/html',
    );

    // Create an iterator to allow us to traverse the DOM node by node, excluding the
    // text nodes inside mentions
    const iterator = document.createNodeIterator(
        composer,
        undefined,
        nodeFilter,
    );
    let node = iterator.nextNode();

    // Use this to store the manually built markdown output
    let markdownOutput = '';

    while (node !== null) {
        // TEXT NODES - `node` represents the text node
        const isTextNode = node.nodeName === '#text';

        // MENTION NODES - `node` represents the enclosing <a> tag
        const isMention = node.nodeName === 'A';

        // LINEBREAK DIVS - `node` represents the enclosing <div> tag
        const isDivContainingBreak =
            node.nodeName === 'DIV' &&
            node.childNodes.length === 1 &&
            node.firstChild?.nodeName === 'BR';

        // UNEXPECTED NODE - `node` represents an unexpected tag type
        const isUnexpectedNode = !expectedNodeNames.includes(node.nodeName);

        if (isDivContainingBreak) {
            markdownOutput += NEWLINE_CHAR;
        } else if (isTextNode) {
            // content is the text itself, unescaped i.e. > is >, not &gt;
            let content = node.textContent;
            if (shouldAddNewlineCharacter(node)) {
                content += NEWLINE_CHAR;
            }
            markdownOutput += content;
        } else if (isMention) {
            // content is the html of the mention i.e. <a ...attributes>text</a>
            let content = node.firstChild?.parentElement?.outerHTML ?? '';
            if (shouldAddNewlineCharacter(node)) {
                content += NEWLINE_CHAR;
            }
            markdownOutput += content;
        } else if (isUnexpectedNode) {
            console.debug(`Converting unexpected node type ${node.nodeName}`);
        }

        node = iterator.nextNode();
    }

    // After converting the DOM, we need to trim a single `\n` off the end of the
    // output if we have consecutive newlines, as this is a browser placeholder
    if (markdownOutput.endsWith(NEWLINE_CHAR.repeat(2))) {
        markdownOutput = markdownOutput.slice(0, -1);
    }

    return markdownOutput;
}

const expectedNodeNames = ['#text', 'BR', 'A', 'DIV', 'BODY'];

// When we parse the nodes, we need to manually add newlines if the node is either
// adjacent to a div or is the last child and it's parent is adjacent to a div
function shouldAddNewlineCharacter(node: Node): boolean {
    const nextSibling = node.nextSibling || node.parentElement?.nextSibling;

    return nextSibling?.nodeName === 'DIV';
}

// Filter to allow us to skip evaluating the text nodes inside mentions
function nodeFilter(node: Node): number {
    if (
        node.nodeName === '#text' &&
        node.parentElement?.hasAttribute('data-mention-type')
    ) {
        return NodeFilter.FILTER_REJECT;
    }
    return NodeFilter.FILTER_ACCEPT;
}
