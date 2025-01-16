// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use crate::dom::nodes::dom_node::DomNodeKind;
use crate::dom::nodes::{ContainerNode, DomNode, LineBreakNode, TextNode};
use crate::dom::range::DomLocation;
use crate::dom::unicode_string::UnicodeStrExt;
use crate::dom::{Dom, DomHandle, FindResult, Range};
use crate::UnicodeString;
use std::cmp::{max, min};

use super::nodes::MentionNode;

pub fn find_range<S>(dom: &Dom<S>, start: usize, end: usize) -> Range
where
    S: UnicodeString,
{
    if dom.children().is_empty() {
        return Range::new(Vec::new());
    }

    // If end < start, we swap start & end to make calculations easier, then
    // reverse the returned ranges
    let is_reversed = end < start;
    let (s, e) = if is_reversed {
        (end, start)
    } else {
        (start, end)
    };

    // TODO: is there really a difference between find_pos and find_range?
    let result = find_pos(dom, &dom.document_handle(), s, e);
    match result {
        FindResult::Found(locations) => {
            let locations: Vec<DomLocation> = if is_reversed {
                locations
                    .iter()
                    .map(|location| location.reversed())
                    .collect()
            } else {
                locations
            };
            Range::new(&locations)
        }
        FindResult::NotFound => Range::new(Vec::new()),
    }
}

/// Find a particular character range in the DOM
pub fn find_pos<S>(
    dom: &Dom<S>,
    node_handle: &DomHandle,
    start: usize,
    end: usize,
) -> FindResult
where
    S: UnicodeString,
{
    let mut offset = 0;
    let locations = do_find_pos(dom, node_handle, start, end, &mut offset);

    if locations.is_empty() {
        FindResult::NotFound
    } else {
        FindResult::Found(locations)
    }
}

fn do_find_pos<S>(
    dom: &Dom<S>,
    node_handle: &DomHandle,
    start: usize,
    end: usize,
    offset: &mut usize,
) -> Vec<DomLocation>
where
    S: UnicodeString,
{
    let node = dom.lookup_node(node_handle);
    let mut locations = Vec::new();
    if *offset > end {
        *offset += node.text_len();
        return locations;
    }
    match node {
        DomNode::Text(n) => {
            if let Some(location) = process_text_node(n, start, end, offset) {
                locations.push(location);
            }
        }
        DomNode::LineBreak(n) => {
            if let Some(location) =
                process_line_break_node(n, start, end, offset)
            {
                locations.push(location);
            }
        }
        DomNode::Mention(n) => {
            if let Some(location) = process_mention_node(n, start, end, offset)
            {
                locations.push(location);
            }
        }
        DomNode::Container(n) => {
            locations
                .extend(process_container_node(dom, n, start, end, offset));
        }
    }
    locations
}

fn process_container_node<S>(
    dom: &Dom<S>,
    node: &ContainerNode<S>,
    start: usize,
    end: usize,
    offset: &mut usize,
) -> Vec<DomLocation>
where
    S: UnicodeString,
{
    let mut results = Vec::new();
    let container_start = *offset;
    for child in node.children() {
        let child_handle = child.handle();
        assert!(!child_handle.is_root(), "Incorrect child handle!");
        let locations = do_find_pos(dom, &child_handle, start, end, offset);
        if !locations.is_empty() {
            results.extend(locations);
        }
    }
    // If container node is completely selected, include it
    let mut container_end = *offset;
    if node.is_block_node() && !node.handle().is_root() {
        container_end += 1;
        if !dom.is_last_in_parent(&node.handle()) {
            *offset = container_end;
        }
    }
    let container_node_len = container_end - container_start;
    // We never want to return the root node
    if container_end >= start && container_start <= end {
        let start_offset = max(start, container_start) - container_start;
        let end_offset = min(end, container_end) - container_start;
        results.push(DomLocation {
            node_handle: node.handle(),
            position: container_start,
            start_offset,
            end_offset,
            length: container_node_len,
            kind: DomNodeKind::from_container_kind(node.kind()),
        })
    }
    results
}

fn process_text_node<S>(
    node: &TextNode<S>,
    start: usize,
    end: usize,
    offset: &mut usize,
) -> Option<DomLocation>
where
    S: UnicodeString,
{
    process_textlike_node(
        node.handle(),
        node.data().len(),
        start,
        end,
        offset,
        DomNodeKind::Text,
    )
}

fn process_line_break_node<S>(
    node: &LineBreakNode<S>,
    start: usize,
    end: usize,
    offset: &mut usize,
) -> Option<DomLocation>
where
    S: UnicodeString,
{
    // Line breaks are like 1-character text nodes
    process_textlike_node(
        node.handle(),
        1,
        start,
        end,
        offset,
        DomNodeKind::LineBreak,
    )
}

fn process_mention_node<S>(
    node: &MentionNode<S>,
    start: usize,
    end: usize,
    offset: &mut usize,
) -> Option<DomLocation>
where
    S: UnicodeString,
{
    // Mentions are like 1-character text nodes
    process_textlike_node(
        node.handle(),
        1,
        start,
        end,
        offset,
        DomNodeKind::Mention,
    )
}

fn process_textlike_node(
    handle: DomHandle,
    node_len: usize,
    start: usize,
    end: usize,
    offset: &mut usize,
    kind: DomNodeKind,
) -> Option<DomLocation> {
    let node_start = *offset;
    let node_end = node_start + node_len;

    // Increase offset to keep track of the current position
    *offset += node_len;
    let is_cursor = start == end;

    let should_discard = match is_cursor {
        // for a cursor, discard when it is outside the node
        true => start < node_start || start > node_end,
        // for a selection, discard when it ends before (inclusive) or starts after (inclusive) the node
        false => end <= node_start || start >= node_end,
    };

    if should_discard {
        None
    } else {
        // Diff between selected position and the start position of the node
        let start_offset = max(start, node_start) - node_start;
        let end_offset = min(end, node_end) - node_start;

        Some(DomLocation {
            node_handle: handle,
            position: node_start,
            start_offset,
            end_offset,
            length: node_len,
            kind,
        })
    }
}

#[cfg(test)]
mod test {
    // TODO: more tests for start and end of ranges

    use widestring::Utf16String;

    use super::*;
    use crate::tests::testutils_composer_model::{cm, restore_whitespace_u16};
    use crate::tests::testutils_conversion::utf16;
    use crate::tests::testutils_dom::{b, dom, tn};
    use crate::{InlineFormatType, InlineFormatType::Italic, ToHtml};

    fn make_single_location(
        handle: DomHandle,
        position: usize,
        start_offset: usize,
        end_offset: usize,
        length: usize,
        kind: DomNodeKind,
    ) -> DomLocation {
        DomLocation {
            node_handle: handle,
            position,
            start_offset,
            end_offset,
            length,
            kind,
        }
    }

    fn ranges_to_html(
        dom: &Dom<Utf16String>,
        range: &Range,
    ) -> Vec<Utf16String> {
        range
            .locations
            .iter()
            .map(|location| {
                restore_whitespace_u16(
                    &dom.lookup_node(&location.node_handle).to_html(),
                )
            })
            .collect()
    }

    #[test]
    fn finding_a_node_within_an_empty_dom_returns_only_root_location() {
        let d = dom(&[]);
        assert_eq!(
            find_pos(&d, &d.document_handle(), 0, 0),
            FindResult::Found(vec![make_single_location(
                DomHandle::root(),
                0,
                0,
                0,
                0,
                DomNodeKind::Generic
            ),])
        );
    }

    #[test]
    fn finding_a_node_within_a_single_text_node_is_found() {
        let d = dom(&[tn("foo")]);
        assert_eq!(
            find_pos(&d, &d.document_handle(), 1, 1),
            FindResult::Found(vec![
                make_single_location(
                    DomHandle::from_raw(vec![0]),
                    0,
                    1,
                    1,
                    3,
                    DomNodeKind::Text
                ),
                make_single_location(
                    DomHandle::root(),
                    0,
                    1,
                    1,
                    3,
                    DomNodeKind::Generic
                ),
            ])
        );
    }

    #[test]
    fn finding_a_node_within_a_single_text_node_with_emoji_is_found() {
        let d = dom(&[tn("🤗")]);
        assert_eq!(
            find_pos(&d, &d.document_handle(), 2, 2),
            FindResult::Found(vec![
                make_single_location(
                    DomHandle::from_raw(vec![0]),
                    0,
                    2,
                    2,
                    2,
                    DomNodeKind::Text
                ),
                make_single_location(
                    DomHandle::root(),
                    0,
                    2,
                    2,
                    2,
                    DomNodeKind::Generic
                ),
            ])
        );
    }

    #[test]
    fn finding_first_node_within_flat_text_nodes_is_found() {
        let d = dom(&[tn("foo"), tn("bar")]);
        assert_eq!(
            find_pos(&d, &d.document_handle(), 0, 0),
            FindResult::Found(vec![
                make_single_location(
                    DomHandle::from_raw(vec![0]),
                    0,
                    0,
                    0,
                    3,
                    DomNodeKind::Text
                ),
                make_single_location(
                    DomHandle::root(),
                    0,
                    0,
                    0,
                    6,
                    DomNodeKind::Generic
                ),
            ])
        );
        assert_eq!(
            find_pos(&d, &d.document_handle(), 1, 1),
            FindResult::Found(vec![
                make_single_location(
                    DomHandle::from_raw(vec![0]),
                    0,
                    1,
                    1,
                    3,
                    DomNodeKind::Text
                ),
                make_single_location(
                    DomHandle::root(),
                    0,
                    1,
                    1,
                    6,
                    DomNodeKind::Generic
                ),
            ])
        );
        assert_eq!(
            find_pos(&d, &d.document_handle(), 2, 2),
            FindResult::Found(vec![
                make_single_location(
                    DomHandle::from_raw(vec![0]),
                    0,
                    2,
                    2,
                    3,
                    DomNodeKind::Text
                ),
                make_single_location(
                    DomHandle::root(),
                    0,
                    2,
                    2,
                    6,
                    DomNodeKind::Generic
                ),
            ])
        );
    }

    #[test]
    fn finding_second_node_within_flat_text_nodes_is_found() {
        let d = dom(&[tn("foo"), tn("bar")]);
        assert_eq!(
            find_pos(&d, &d.document_handle(), 4, 4),
            FindResult::Found(vec![
                make_single_location(
                    DomHandle::from_raw(vec![1]),
                    3,
                    1,
                    1,
                    3,
                    DomNodeKind::Text
                ),
                make_single_location(
                    DomHandle::root(),
                    0,
                    4,
                    4,
                    6,
                    DomNodeKind::Generic
                ),
            ])
        );
        assert_eq!(
            find_pos(&d, &d.document_handle(), 5, 5),
            FindResult::Found(vec![
                make_single_location(
                    DomHandle::from_raw(vec![1]),
                    3,
                    2,
                    2,
                    3,
                    DomNodeKind::Text
                ),
                make_single_location(
                    DomHandle::root(),
                    0,
                    5,
                    5,
                    6,
                    DomNodeKind::Generic
                ),
            ])
        );
        assert_eq!(
            find_pos(&d, &d.document_handle(), 6, 6),
            FindResult::Found(vec![
                make_single_location(
                    DomHandle::from_raw(vec![1]),
                    3,
                    3,
                    3,
                    3,
                    DomNodeKind::Text
                ),
                make_single_location(
                    DomHandle::root(),
                    0,
                    6,
                    6,
                    6,
                    DomNodeKind::Generic
                ),
            ])
        );
    }

    // TODO: comprehensive test like above for non-flat nodes

    #[test]
    fn finding_a_boundary_between_flat_text_nodes_finds_both() {
        let d = dom(&[tn("foo"), tn("bar")]);
        assert_eq!(
            find_pos(&d, &d.document_handle(), 3, 3),
            FindResult::Found(vec![
                make_single_location(
                    DomHandle::from_raw(vec![0]),
                    0,
                    3,
                    3,
                    3,
                    DomNodeKind::Text
                ),
                make_single_location(
                    DomHandle::from_raw(vec![1]),
                    3,
                    0,
                    0,
                    3,
                    DomNodeKind::Text
                ),
                make_single_location(
                    DomHandle::root(),
                    0,
                    3,
                    3,
                    6,
                    DomNodeKind::Generic
                ),
            ])
        );
    }

    #[test]
    fn finding_a_range_within_an_empty_dom_returns_no_nodes() {
        let d = dom(&[]);
        let range = d.find_range(0, 0);
        assert_eq!(range, Range::new(Vec::new()));
    }
    // TODO: comprehensive test like above for non-flat nodes

    #[test]
    fn finding_a_range_within_the_single_text_node_works() {
        let d = dom(&[tn("foo bar baz")]);
        let range = d.find_range(4, 7);

        let leaves: Vec<&DomLocation> = range.leaves().collect();
        assert_eq!(leaves.len(), 1);

        let loc = leaves[0];
        assert_eq!(loc.start_offset, 4);
        assert_eq!(loc.end_offset, 7);

        if let DomNode::Text(t) = d.lookup_node(&loc.node_handle) {
            assert_eq!(*t.data(), utf16("foo bar baz"));
        } else {
            panic!("Should have been a text node!")
        }

        assert_eq!(loc.node_handle.raw(), &vec![0]);
    }

    #[test]
    fn finding_a_range_that_includes_the_end_works_simple_case() {
        let d = dom(&[tn("foo bar baz")]);
        let range = d.find_range(4, 11);

        let leaves: Vec<&DomLocation> = range.leaves().collect();
        assert_eq!(leaves.len(), 1);

        let loc = leaves[0];
        assert_eq!(loc.start_offset, 4);
        assert_eq!(loc.end_offset, 11);

        if let DomNode::Text(t) = d.lookup_node(&loc.node_handle) {
            assert_eq!(*t.data(), utf16("foo bar baz"));
        } else {
            panic!("Should have been a text node!")
        }

        assert_eq!(loc.node_handle.raw(), &vec![0]);
    }

    #[test]
    fn finding_a_range_within_some_nested_node_works() {
        let d = dom(&[tn("foo "), b(&[tn("bar")]), tn(" baz")]);
        let range = d.find_range(5, 6);

        let leaves: Vec<&DomLocation> = range.leaves().collect();
        assert_eq!(leaves.len(), 1);

        let loc = leaves[0];
        assert_eq!(loc.start_offset, 1);
        assert_eq!(loc.end_offset, 2);

        if let DomNode::Text(t) = d.lookup_node(&loc.node_handle) {
            assert_eq!(*t.data(), utf16("bar"));
        } else {
            panic!("Should have been a text node!")
        }

        assert_eq!(loc.node_handle.raw(), &vec![1, 0]);
    }

    #[test]
    fn finding_a_range_across_several_nodes_works() {
        let d = cm("test<b>ing a </b>new feature|").state.dom;
        let range = d.find_range(2, 12);

        // 3 text nodes + bold node
        assert_eq!(5, range.locations.len());
        let html_of_ranges = ranges_to_html(&d, &range);
        assert_eq!(utf16("test"), html_of_ranges[0]);
        assert_eq!(utf16("ing a "), html_of_ranges[1]);
        assert_eq!(utf16("<b>ing a </b>"), html_of_ranges[2]);
        assert_eq!(utf16("new feature"), html_of_ranges[3]);
        assert_eq!(utf16("test<b>ing a </b>new feature"), html_of_ranges[4]);
    }

    #[test]
    fn finding_a_range_across_several_nested_nodes_works() {
        let d = cm("test<b>ing <i>a </i></b>new feature|").state.dom;
        let range = d.find_range(2, 12);
        // 4 text nodes + bold node + italic node
        assert_eq!(7, range.locations.len());
        let html_of_ranges = ranges_to_html(&d, &range);
        assert_eq!(utf16("test"), html_of_ranges[0]);
        assert_eq!(utf16("ing "), html_of_ranges[1]);
        assert_eq!(utf16("a "), html_of_ranges[2]);
        assert_eq!(utf16("<i>a </i>"), html_of_ranges[3]);
        assert_eq!(utf16("<b>ing <i>a </i></b>"), html_of_ranges[4]);
        assert_eq!(utf16("new feature"), html_of_ranges[5]);
        assert_eq!(
            utf16("test<b>ing <i>a </i></b>new feature"),
            html_of_ranges[6]
        );
    }

    #[test]
    fn finding_a_range_inside_several_nested_nodes_returns_text_node() {
        let d = cm("test<b>ing <i>a </i></b>new feature|").state.dom;
        let range = d.find_range(9, 10);
        // Selected the 'a' character inside the <i> tag, but as it only
        // covers it partially, only the text node is selected
        assert_eq!(
            range,
            Range {
                locations: vec![
                    DomLocation {
                        node_handle: DomHandle::from_raw(vec![1, 1, 0]),
                        start_offset: 1,
                        end_offset: 2,
                        position: 8,
                        length: 2,
                        kind: DomNodeKind::Text,
                    },
                    DomLocation {
                        node_handle: DomHandle::from_raw(vec![1, 1]),
                        start_offset: 1,
                        end_offset: 2,
                        position: 8,
                        length: 2,
                        kind: DomNodeKind::Formatting(InlineFormatType::Italic),
                    },
                    DomLocation {
                        node_handle: DomHandle::from_raw(vec![1]),
                        start_offset: 5,
                        end_offset: 6,
                        position: 4,
                        length: 6,
                        kind: DomNodeKind::Formatting(InlineFormatType::Bold),
                    },
                    DomLocation {
                        node_handle: DomHandle::root(),
                        start_offset: 9,
                        end_offset: 10,
                        position: 0,
                        length: 21,
                        kind: DomNodeKind::Generic,
                    }
                ]
            }
        );
    }

    #[test]
    fn finding_a_range_spanning_nested_nodes_selects_text_node_and_parent() {
        let d = cm("test<b>ing <i>a </i></b>new feature|").state.dom;
        // The range of the whole <i> tag
        let range = d.find_range(8, 11);
        // 2 text nodes + italic node
        assert_eq!(5, range.locations.len());
        let html_of_ranges = ranges_to_html(&d, &range);
        assert_eq!(utf16("a "), html_of_ranges[0]);
        assert_eq!(utf16("<i>a </i>"), html_of_ranges[1]);
        assert_eq!(utf16("<b>ing <i>a </i></b>"), html_of_ranges[2]);
        assert_eq!(utf16("new feature"), html_of_ranges[3]);
        assert_eq!(
            utf16("test<b>ing <i>a </i></b>new feature"),
            html_of_ranges[4]
        );
    }

    #[test]
    fn find_range_builds_dom_location_with_expected_length() {
        let model = cm("<em>remains |<em>all<em>of<em>the<em>rest</em>goes</em>away</em>x</em>y</em>");
        let (s, e) = model.safe_selection();
        let range = model.state.dom.find_range(s, e);
        assert_eq!(
            range,
            Range {
                locations: vec![
                    DomLocation {
                        node_handle: DomHandle::from_raw(vec![0, 0]),
                        start_offset: 8,
                        end_offset: 8,
                        position: 0,
                        length: 8,
                        kind: DomNodeKind::Text
                    },
                    DomLocation {
                        node_handle: DomHandle::from_raw(vec![0, 1, 0]),
                        start_offset: 0,
                        end_offset: 0,
                        position: 8,
                        length: 3,
                        kind: DomNodeKind::Text
                    },
                    DomLocation {
                        node_handle: DomHandle::from_raw(vec![0, 1]),
                        start_offset: 0,
                        end_offset: 0,
                        position: 8,
                        length: 21,
                        kind: DomNodeKind::Formatting(Italic)
                    },
                    DomLocation {
                        node_handle: DomHandle::from_raw(vec![0]),
                        start_offset: 8,
                        end_offset: 8,
                        position: 0,
                        length: 30,
                        kind: DomNodeKind::Formatting(Italic)
                    },
                    DomLocation {
                        node_handle: DomHandle::root(),
                        start_offset: 8,
                        end_offset: 8,
                        position: 0,
                        length: 30,
                        kind: DomNodeKind::Generic
                    },
                ]
            }
        )
    }
}
