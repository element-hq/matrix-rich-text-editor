// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use md_parser::Event;
use pulldown_cmark as md_parser;

use crate::{dom::MarkdownParseError, UnicodeString};

pub struct MarkdownHTMLParser {}

impl MarkdownHTMLParser {
    pub fn to_html<S>(markdown: &S) -> Result<S, MarkdownParseError>
    where
        S: UnicodeString,
    {
        use md_parser::{html::push_html as compile_to_html, Options, Parser};

        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);

        let markdown = markdown.to_string();
        let parser_events: Vec<_> = Parser::new_ext(&markdown, options)
            .map(|event| match event {
                // this allows for line breaks to be parsed correctly from markdown
                Event::SoftBreak => Event::HardBreak,
                _ => event,
            })
            .collect();

        let mut html = String::new();

        compile_to_html(&mut html, parser_events.into_iter());

        // By default, there is a `<p>…</p>\n` around the HTML content. That's the
        // correct way to handle a text block in Markdown. But it breaks our
        // assumption regarding the HTML markup. So let's remove it.
        let html = {
            // only remove the external <p> if there is only one
            if html.starts_with("<p>") && html.matches("<p>").count() == 1 {
                let p = "<p>".len();
                let ppnl = "</p>\n".len();

                html[p..html.len() - ppnl].to_string()
            } else {
                html[..].to_string()
            }
        };

        let html = html
            // Allow for having a newline between paragraphs
            .replace("</p>\n<p>", "</p><p>&nbsp;</p><p>")
            // Remove any trailing newline characters from block tags
            .replace("<ul>\n", "<ul>")
            .replace("</ul>\n", "</ul>")
            .replace("<ol>\n", "<ol>")
            .replace("</ol>\n", "</ol>")
            .replace("</li>\n", "</li>")
            .replace("<br />\n", "<br />")
            .replace("<blockquote>\n", "<blockquote>")
            .replace("</blockquote>\n", "</blockquote>")
            .replace("<pre>\n", "<pre>")
            .replace("</pre>\n", "</pre>")
            .replace("<p>\n", "<p>")
            .replace("</p>\n", "</p>")
            // Remove the newline from the end of the single code tag that wraps the content
            // of a formatted codeblock
            .replace("\n</code>", "</code>");

        Ok(S::from(html))
    }
}
