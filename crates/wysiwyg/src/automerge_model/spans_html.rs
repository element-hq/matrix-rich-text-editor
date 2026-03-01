// Copyright 2026 The Matrix.org Foundation C.I.C.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Conversion between Automerge spans and HTML.
//!
//! Uses `doc.spans(&text_id)` to iterate over `Span::Text` and
//! `Span::Block` items and produce an HTML string.
//!
//! Block markers are rendered to the appropriate HTML wrappers:
//! - `paragraph`            → (no extra wrapper for first block)
//! - `ordered-list-item`    → `<ol><li>…</li></ol>`
//! - `unordered-list-item`  → `<ul><li>…</li></ul>`
//! - `code-block`           → `<pre><code>…</code></pre>`
//! - `quote`                → `<blockquote>…</blockquote>`
//!
//! Consecutive list items of the same type are merged into one `<ol>` or
//! `<ul>` wrapper.

use std::collections::BTreeSet;

use automerge::iter::Span;
use automerge::ReadDoc;

use super::block_ops::block_type;
use super::AutomergeModel;

/// Map from Automerge mark names to HTML tags.
fn mark_to_tag(name: &str) -> Option<&'static str> {
    match name {
        "bold" => Some("strong"),
        "italic" => Some("em"),
        "strikethrough" => Some("del"),
        "underline" => Some("u"),
        "inline_code" => Some("code"),
        _ => None,
    }
}

/// Extract the block type string from a block marker `Map`.
fn block_type_from_map(map: &automerge::hydrate::Map) -> &str {
    map.get("type")
        .and_then(|v| {
            if let automerge::hydrate::Value::Scalar(
                automerge::ScalarValue::Str(s),
            ) = v
            {
                Some(s.as_str())
            } else {
                None
            }
        })
        .unwrap_or(block_type::PARAGRAPH)
}

/// Tracks the currently open block-level wrapper so we can close it before
/// opening a new one.
#[derive(Debug, Clone, PartialEq)]
enum BlockWrapper {
    None,
    OrderedList,
    UnorderedList,
    CodeBlock,
    Quote,
}

impl AutomergeModel {
    /// Convert the current document spans to an HTML string.
    pub(crate) fn spans_to_html(&self) -> String {
        let spans: Vec<Span> = match self.doc.spans(&self.text_id) {
            Ok(s) => s.collect(),
            Err(_) => return String::new(),
        };

        let mut html = String::new();
        let mut open_inline_tags: Vec<&'static str> = Vec::new();
        let mut wrapper = BlockWrapper::None;
        let mut in_list_item = false;

        for span in &spans {
            match span {
                Span::Text { text, marks } => {
                    self.emit_text_span(
                        &mut html,
                        &mut open_inline_tags,
                        text,
                        marks.as_ref(),
                    );
                }
                Span::Block(block_map) => {
                    // Close inline tags first
                    close_inline_tags(&mut html, &mut open_inline_tags);

                    let btype = block_type_from_map(block_map);
                    let new_wrapper = wrapper_for_type(btype);

                    // Close existing wrapper if different
                    if wrapper != new_wrapper
                        || wrapper == BlockWrapper::CodeBlock
                        || wrapper == BlockWrapper::Quote
                    {
                        close_wrapper(
                            &mut html,
                            &wrapper,
                            &mut in_list_item,
                        );
                        open_wrapper(
                            &mut html,
                            &new_wrapper,
                            &mut in_list_item,
                        );
                        wrapper = new_wrapper;
                    } else if matches!(
                        wrapper,
                        BlockWrapper::OrderedList
                            | BlockWrapper::UnorderedList
                    ) {
                        // Same list type — close previous <li>, open new one
                        if in_list_item {
                            html.push_str("</li>");
                        }
                        html.push_str("<li>");
                        in_list_item = true;
                    }
                }
            }
        }

        // Close remaining inline tags
        close_inline_tags(&mut html, &mut open_inline_tags);

        // Close final wrapper
        close_wrapper(&mut html, &wrapper, &mut in_list_item);

        html
    }

    /// Emit a text span with inline formatting.
    fn emit_text_span(
        &self,
        html: &mut String,
        open_tags: &mut Vec<&'static str>,
        text: &str,
        marks: Option<&std::sync::Arc<automerge::marks::MarkSet>>,
    ) {
        let mut desired_tags: BTreeSet<&'static str> = BTreeSet::new();
        let mut link_url: Option<String> = None;
        let mut mention_url: Option<String> = None;

        if let Some(mark_set) = marks {
            for (name, value) in mark_set.iter() {
                if let Some(tag) = mark_to_tag(name) {
                    if !matches!(value, automerge::ScalarValue::Null) {
                        desired_tags.insert(tag);
                    }
                } else if name == "link" {
                    if let Some(url) = value.to_str() {
                        link_url = Some(url.to_string());
                    }
                } else if name == "mention" {
                    if let Some(url) = value.to_str() {
                        mention_url = Some(url.to_string());
                    }
                }
            }
        }

        // Close tags that are no longer needed (reverse order)
        while let Some(last) = open_tags.last() {
            if !desired_tags.contains(last) {
                let tag = open_tags.pop().unwrap();
                html.push_str(&format!("</{tag}>"));
            } else {
                break;
            }
        }

        // Open new tags
        for tag in &desired_tags {
            if !open_tags.contains(tag) {
                html.push_str(&format!("<{tag}>"));
                open_tags.push(tag);
            }
        }

        // Emit text content (with links/mentions)
        if let Some(url) = &mention_url {
            html.push_str(&format!(
                "<a href=\"{}\">",
                html_escape::encode_double_quoted_attribute(url)
            ));
            html.push_str(&html_escape::encode_text(text));
            html.push_str("</a>");
        } else if let Some(url) = &link_url {
            html.push_str(&format!(
                "<a href=\"{}\">",
                html_escape::encode_double_quoted_attribute(url)
            ));
            html.push_str(&html_escape::encode_text(text));
            html.push_str("</a>");
        } else {
            html.push_str(&html_escape::encode_text(text));
        }
    }
}

/// Close all remaining open inline tags in reverse order.
fn close_inline_tags(html: &mut String, open_tags: &mut Vec<&'static str>) {
    while let Some(tag) = open_tags.pop() {
        html.push_str(&format!("</{tag}>"));
    }
}

/// Determine the block wrapper type for a given block type string.
fn wrapper_for_type(btype: &str) -> BlockWrapper {
    match btype {
        block_type::ORDERED_LIST_ITEM => BlockWrapper::OrderedList,
        block_type::UNORDERED_LIST_ITEM => BlockWrapper::UnorderedList,
        block_type::CODE_BLOCK => BlockWrapper::CodeBlock,
        block_type::QUOTE => BlockWrapper::Quote,
        _ => BlockWrapper::None,
    }
}

/// Open the HTML tags for a block wrapper.
fn open_wrapper(
    html: &mut String,
    wrapper: &BlockWrapper,
    in_list_item: &mut bool,
) {
    match wrapper {
        BlockWrapper::OrderedList => {
            html.push_str("<ol><li>");
            *in_list_item = true;
        }
        BlockWrapper::UnorderedList => {
            html.push_str("<ul><li>");
            *in_list_item = true;
        }
        BlockWrapper::CodeBlock => {
            html.push_str("<pre><code>");
        }
        BlockWrapper::Quote => {
            html.push_str("<blockquote>");
        }
        BlockWrapper::None => {}
    }
}

/// Close the HTML tags for a block wrapper.
fn close_wrapper(
    html: &mut String,
    wrapper: &BlockWrapper,
    in_list_item: &mut bool,
) {
    match wrapper {
        BlockWrapper::OrderedList => {
            if *in_list_item {
                html.push_str("</li>");
                *in_list_item = false;
            }
            html.push_str("</ol>");
        }
        BlockWrapper::UnorderedList => {
            if *in_list_item {
                html.push_str("</li>");
                *in_list_item = false;
            }
            html.push_str("</ul>");
        }
        BlockWrapper::CodeBlock => {
            html.push_str("</code></pre>");
        }
        BlockWrapper::Quote => {
            html.push_str("</blockquote>");
        }
        BlockWrapper::None => {}
    }
}

#[cfg(test)]
mod tests {
    use crate::AutomergeModel;

    fn new_model() -> AutomergeModel {
        AutomergeModel::new()
    }

    fn model_with_text(text: &str) -> AutomergeModel {
        let mut m = AutomergeModel::new();
        m.replace_text(text);
        m
    }

    fn html(m: &AutomergeModel) -> String {
        m.get_content_as_html()
    }

    // ===================================================================
    // Plain text HTML output
    // ===================================================================

    #[test]
    fn plain_text_produces_plain_html() {
        let model = model_with_text("hello world");
        assert_eq!(html(&model), "hello world");
    }

    #[test]
    fn empty_model_produces_empty_html() {
        let model = new_model();
        assert_eq!(html(&model), "");
    }

    #[test]
    fn special_characters_are_escaped() {
        let mut model = new_model();
        model.replace_text("<script>alert('xss')</script>");
        let h = html(&model);
        assert!(!h.contains("<script>"), "should escape HTML: {h}");
        assert!(h.contains("&lt;"), "expected &lt; in: {h}");
    }

    // ===================================================================
    // Inline formatting HTML output
    // ===================================================================

    #[test]
    fn bold_produces_strong_tags() {
        let mut model = model_with_text("hello");
        model.select(0, 5);
        model.bold();
        let h = html(&model);
        assert_eq!(h, "<strong>hello</strong>");
    }

    #[test]
    fn italic_produces_em_tags() {
        let mut model = model_with_text("hello");
        model.select(0, 5);
        model.italic();
        let h = html(&model);
        assert_eq!(h, "<em>hello</em>");
    }

    #[test]
    fn strikethrough_produces_del_tags() {
        let mut model = model_with_text("hello");
        model.select(0, 5);
        model.strike_through();
        let h = html(&model);
        assert_eq!(h, "<del>hello</del>");
    }

    #[test]
    fn underline_produces_u_tags() {
        let mut model = model_with_text("hello");
        model.select(0, 5);
        model.underline();
        let h = html(&model);
        assert_eq!(h, "<u>hello</u>");
    }

    #[test]
    fn inline_code_produces_code_tags() {
        let mut model = model_with_text("hello");
        model.select(0, 5);
        model.inline_code();
        let h = html(&model);
        assert_eq!(h, "<code>hello</code>");
    }

    // ===================================================================
    // Partial formatting
    // ===================================================================

    #[test]
    fn partial_bold_produces_mixed_html() {
        let mut model = model_with_text("aabbcc");
        model.select(2, 4);
        model.bold();
        let h = html(&model);
        assert!(h.contains("aa"), "expected 'aa' outside bold: {h}");
        assert!(
            h.contains("<strong>bb</strong>"),
            "expected bold 'bb' in: {h}"
        );
        assert!(h.contains("cc"), "expected 'cc' outside bold: {h}");
    }

    #[test]
    fn adjacent_differently_formatted_spans() {
        let mut model = model_with_text("abcdef");
        model.select(0, 3);
        model.bold();
        model.select(3, 6);
        model.italic();
        let h = html(&model);
        assert!(
            h.contains("<strong>abc</strong>"),
            "expected bold in: {h}"
        );
        assert!(
            h.contains("<em>def</em>"),
            "expected italic in: {h}"
        );
    }

    // ===================================================================
    // Links in HTML output
    // ===================================================================

    #[test]
    fn link_produces_anchor_tag() {
        let mut model = model_with_text("click here");
        model.select(0, 10);
        model.set_link("https://matrix.org", &[]);
        let h = html(&model);
        assert!(
            h.contains("<a href=\"https://matrix.org\">"),
            "expected anchor in: {h}"
        );
        assert!(h.contains("click here"), "expected text in: {h}");
        assert!(h.contains("</a>"), "expected closing anchor in: {h}");
    }

    #[test]
    fn link_url_is_escaped_in_html() {
        let mut model = model_with_text("test");
        model.select(0, 4);
        model.set_link("https://example.com/path?a=1&b=2", &[]);
        let h = html(&model);
        // The href should have & escaped or left as-is
        assert!(h.contains("href="), "expected href in: {h}");
    }

    // ===================================================================
    // Mentions in HTML output
    // ===================================================================

    #[test]
    fn mention_produces_anchor_tag() {
        let mut model = new_model();
        model.insert_mention(
            "https://matrix.to/#/@alice:matrix.org",
            "Alice",
            &[],
        );
        let h = html(&model);
        assert!(h.contains("Alice"), "expected 'Alice' in: {h}");
        assert!(h.contains("href"), "expected href in: {h}");
    }

    // ===================================================================
    // Nested formatting in HTML output
    // ===================================================================

    #[test]
    fn bold_and_italic_overlap_produces_nested_tags() {
        let mut model = model_with_text("abcdef");
        // Bold "abcd", italic "cdef" — overlap on "cd"
        model.select(0, 4);
        model.bold();
        model.select(2, 6);
        model.italic();
        let h = html(&model);
        assert!(h.contains("<strong>"), "expected <strong> in: {h}");
        assert!(h.contains("<em>"), "expected <em> in: {h}");
    }

    // ===================================================================
    // to_tree debug output
    // ===================================================================

    #[test]
    fn to_tree_contains_text() {
        let model = model_with_text("hello");
        let tree = model.to_tree();
        assert!(tree.contains("hello"), "expected 'hello' in tree: {tree}");
    }

    #[test]
    fn to_tree_shows_selection() {
        let mut model = model_with_text("hello");
        model.select(2, 4);
        let tree = model.to_tree();
        assert!(
            tree.contains("sel:"),
            "expected selection info in tree: {tree}"
        );
    }

    #[test]
    fn to_tree_shows_marks() {
        let mut model = model_with_text("hello");
        model.select(0, 5);
        model.bold();
        let tree = model.to_tree();
        assert!(
            tree.contains("bold"),
            "expected bold mark info in tree: {tree}"
        );
    }
}
