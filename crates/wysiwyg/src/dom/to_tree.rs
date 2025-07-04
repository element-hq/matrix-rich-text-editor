// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use super::unicode_string::UnicodeStringExt;
use super::UnicodeString;

const DOUBLE_WHITESPACE: &str = "\u{0020}\u{0020}";
const UP_RIGHT_AND_GT: &str = "\u{2514}\u{003E}";
const VERTICAL_RIGHT_AND_GT: &str = "\u{251C}\u{003E}";
const VERTICAL_AND_WHITESPACE: &str = "\u{2502}\u{0020}";

pub trait ToTree<S>
where
    S: UnicodeString,
{
    /// Output tree representation from current item.
    fn to_tree(&self) -> S {
        self.to_tree_display(vec![])
    }

    /// Output tree representation from current item with
    /// given vector of ancestors that have extra children.
    fn to_tree_display(&self, continuous_positions: Vec<usize>) -> S;

    /// Output content of a tree line with given description
    /// with current depth in the tree as well as positions
    /// of ancestor that have extra children.
    fn tree_line(
        &self,
        description: S,
        depth: usize,
        continuous_positions: Vec<usize>,
    ) -> S {
        let mut tree_part = S::default();
        for i in 0..depth {
            if i == depth - 1 {
                if continuous_positions.contains(&i) {
                    tree_part.push(VERTICAL_RIGHT_AND_GT);
                } else {
                    tree_part.push(UP_RIGHT_AND_GT);
                }
            } else if continuous_positions.contains(&i) {
                tree_part.push(VERTICAL_AND_WHITESPACE);
            } else {
                tree_part.push(DOUBLE_WHITESPACE);
            }
        }
        tree_part.push(description);
        tree_part.push('\n');
        tree_part
    }
}
