// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

use crate::dom::nodes::dom_node::DomNodeKind;
use crate::dom::DomHandle;
use crate::{
    ComposerModel, ComposerUpdate, DomNode, Location, TableCellType,
    UnicodeString,
};

impl<S> ComposerModel<S>
where
    S: UnicodeString,
{
    /// Inserts a new table with the given number of rows and columns at the
    /// current cursor position. Existing content at the cursor is not
    /// wrapped by the table - unlike `quote()`/`code_block()`, a table is
    /// always brand new content rather than a transformation of the
    /// selection.
    pub fn insert_table(
        &mut self,
        rows: usize,
        columns: usize,
    ) -> ComposerUpdate<S> {
        let rows = rows.max(1);
        let columns = columns.max(1);
        self.push_state_to_history();

        let table_node = new_table_node(rows, columns);

        let (s, e) = self.safe_selection();
        let range = self.state.dom.find_range(s, e);
        let leaves: Vec<_> = range.leaves().collect();
        // The handle of the freshly-inserted table, when it can be
        // determined reliably - see the branches below.
        let mut table_handle: Option<DomHandle> = None;
        if leaves.is_empty() {
            if let Some(deepest_block_location) = range.deepest_block_node(None)
            {
                let mut block_node =
                    self.state.dom.remove(&deepest_block_location.node_handle);
                if block_node.is_list_item() {
                    let list_item = block_node.as_container_mut().unwrap();
                    list_item.remove_children();
                    list_item.append_child(table_node);
                    let list_item_handle = self.state.dom.insert_at(
                        &deepest_block_location.node_handle,
                        block_node,
                    );
                    table_handle = Some(list_item_handle.child_handle(0));
                } else {
                    table_handle = Some(self.state.dom.insert_at(
                        &deepest_block_location.node_handle,
                        table_node,
                    ));
                }
            } else {
                table_handle =
                    Some(self.state.dom.append_at_end_of_document(table_node));
            }
        } else {
            // The table is inserted as a new sibling of existing inline
            // content, so that content needs wrapping in a paragraph to
            // keep the invariant that a container's children are either
            // all block or all inline. This can shift sibling indices, so
            // unlike the branches above, the table's resulting handle
            // isn't reliably known here - the cursor is left where it was.
            let first_leaf_loc = leaves.first().unwrap();
            let insert_at = if first_leaf_loc.is_start() {
                first_leaf_loc.node_handle.next_sibling()
            } else {
                first_leaf_loc.node_handle.clone()
            };
            self.state.dom.insert_at(&insert_at, table_node);
            self.state.dom.wrap_inline_nodes_into_paragraphs_if_needed(
                &insert_at.parent_handle(),
            );
        }

        if let Some(table_handle) = table_handle {
            let first_cell_handle =
                table_handle.child_handle(0).child_handle(0);
            let position = self
                .state
                .dom
                .location_for_node(&first_cell_handle)
                .position;
            self.select(Location::from(position), Location::from(position));
        }

        self.create_update_replace_all()
    }

    pub fn add_table_row_before(&mut self) -> ComposerUpdate<S> {
        self.add_table_row(false)
    }

    pub fn add_table_row_after(&mut self) -> ComposerUpdate<S> {
        self.add_table_row(true)
    }

    fn add_table_row(&mut self, after: bool) -> ComposerUpdate<S> {
        let Some((_, row_handle)) = self.current_table_row_handle() else {
            return ComposerUpdate::keep();
        };
        self.push_state_to_history();

        let column_count = self
            .state
            .dom
            .lookup_container(&row_handle)
            .children()
            .len();
        let new_row = new_table_row_node(column_count);
        let insert_at = if after {
            row_handle.next_sibling()
        } else {
            row_handle.clone()
        };
        self.state.dom.insert_at(&insert_at, new_row);

        self.create_update_replace_all()
    }

    /// Removes the table row the cursor is currently in. If it is the only
    /// remaining row, the whole table is removed instead.
    pub fn remove_table_row(&mut self) -> ComposerUpdate<S> {
        let Some((table_handle, row_handle)) = self.current_table_row_handle()
        else {
            return ComposerUpdate::keep();
        };
        self.push_state_to_history();

        let row_count = self
            .state
            .dom
            .lookup_container(&table_handle)
            .children()
            .len();
        if row_count <= 1 {
            self.state.dom.remove(&table_handle);
        } else {
            self.state.dom.remove(&row_handle);
        }

        self.create_update_replace_all()
    }

    pub fn add_table_column_before(&mut self) -> ComposerUpdate<S> {
        self.add_table_column(false)
    }

    pub fn add_table_column_after(&mut self) -> ComposerUpdate<S> {
        self.add_table_column(true)
    }

    fn add_table_column(&mut self, after: bool) -> ComposerUpdate<S> {
        let Some((table_handle, column_index)) =
            self.current_table_and_column()
        else {
            return ComposerUpdate::keep();
        };
        self.push_state_to_history();

        let insert_index = if after {
            column_index + 1
        } else {
            column_index
        };
        let row_count = self
            .state
            .dom
            .lookup_container(&table_handle)
            .children()
            .len();
        for i in 0..row_count {
            let row_handle = table_handle.child_handle(i);
            let row_len = self
                .state
                .dom
                .lookup_container(&row_handle)
                .children()
                .len();
            let cell_handle =
                row_handle.child_handle(insert_index.min(row_len));
            self.state.dom.insert_at(
                &cell_handle,
                DomNode::new_table_cell(TableCellType::Data, Vec::new()),
            );
        }

        self.create_update_replace_all()
    }

    /// Removes the table column the cursor is currently in. If it is the
    /// only remaining column, the whole table is removed instead.
    pub fn remove_table_column(&mut self) -> ComposerUpdate<S> {
        let Some((table_handle, column_index)) =
            self.current_table_and_column()
        else {
            return ComposerUpdate::keep();
        };
        self.push_state_to_history();

        let table = self.state.dom.lookup_container(&table_handle);
        let row_count = table.children().len();
        let column_count = table
            .children()
            .first()
            .and_then(|r| r.as_container())
            .map(|r| r.children().len())
            .unwrap_or(0);

        if column_count <= 1 {
            self.state.dom.remove(&table_handle);
        } else {
            for i in 0..row_count {
                let row_handle = table_handle.child_handle(i);
                let cell_handle = row_handle.child_handle(column_index);
                self.state.dom.remove(&cell_handle);
            }
        }

        self.create_update_replace_all()
    }

    /// Toggles the current row between header cells (`<th>`) and data cells
    /// (`<td>`).
    pub fn toggle_table_header(&mut self) -> ComposerUpdate<S> {
        let Some((_, row_handle)) = self.current_table_row_handle() else {
            return ComposerUpdate::keep();
        };
        self.push_state_to_history();

        let row = self.state.dom.lookup_container(&row_handle);
        let is_header = !row.children().is_empty()
            && row.children().iter().all(|c| {
                matches!(
                    c,
                    DomNode::Container(cell)
                        if cell.get_cell_type() == Some(TableCellType::Header)
                )
            });
        let new_type = if is_header {
            TableCellType::Data
        } else {
            TableCellType::Header
        };

        let cell_count = row.children().len();
        for i in 0..cell_count {
            let cell_handle = row_handle.child_handle(i);
            if let DomNode::Container(cell) =
                self.state.dom.lookup_node_mut(&cell_handle)
            {
                cell.set_cell_type(new_type);
            }
        }

        self.create_update_replace_all()
    }

    /// Removes the table the cursor is currently inside of, if any.
    pub fn remove_table(&mut self) -> ComposerUpdate<S> {
        let (s, e) = self.safe_selection();
        let range = self.state.dom.find_range(s, e);
        let Some(table_location) = range
            .locations
            .iter()
            .find(|l| l.kind == DomNodeKind::Table)
        else {
            return ComposerUpdate::keep();
        };
        self.push_state_to_history();

        self.state.dom.remove(&table_location.node_handle);

        self.create_update_replace_all()
    }

    /// Moves the cursor to the start of the next cell in reading order
    /// (Tab), wrapping to the next row. Tabbing forward from the last cell
    /// of the last row inserts a new row and moves into its first cell,
    /// matching the convention used by Google Docs/Word.
    pub fn move_to_next_cell(&mut self) -> ComposerUpdate<S> {
        self.move_to_adjacent_cell(true)
    }

    /// Moves the cursor to the start of the previous cell in reading order
    /// (Shift+Tab), wrapping to the previous row.
    pub fn move_to_previous_cell(&mut self) -> ComposerUpdate<S> {
        self.move_to_adjacent_cell(false)
    }

    fn move_to_adjacent_cell(&mut self, forward: bool) -> ComposerUpdate<S> {
        let Some((table_handle, cell_handle)) = self.current_table_and_cell()
        else {
            return ComposerUpdate::keep();
        };
        let row_handle = cell_handle.parent_handle();
        let row_index = row_handle.index_in_parent();
        let column_index = cell_handle.index_in_parent();
        let row_count = self
            .state
            .dom
            .lookup_container(&table_handle)
            .children()
            .len();
        let column_count = self
            .state
            .dom
            .lookup_container(&row_handle)
            .children()
            .len();

        let target_cell_handle = if forward {
            if column_index + 1 < column_count {
                Some(row_handle.child_handle(column_index + 1))
            } else if row_index + 1 < row_count {
                Some(table_handle.child_handle(row_index + 1).child_handle(0))
            } else {
                self.push_state_to_history();
                let new_row = new_table_row_node(column_count);
                let new_row_handle = table_handle.child_handle(row_count);
                self.state.dom.insert_at(&new_row_handle, new_row);
                Some(new_row_handle.child_handle(0))
            }
        } else if column_index > 0 {
            Some(row_handle.child_handle(column_index - 1))
        } else if row_index > 0 {
            let prev_row_handle = table_handle.child_handle(row_index - 1);
            let prev_row_column_count = self
                .state
                .dom
                .lookup_container(&prev_row_handle)
                .children()
                .len();
            Some(
                prev_row_handle
                    .child_handle(prev_row_column_count.saturating_sub(1)),
            )
        } else {
            None
        };

        let Some(target_cell_handle) = target_cell_handle else {
            return ComposerUpdate::keep();
        };

        let position = self
            .state
            .dom
            .location_for_node(&target_cell_handle)
            .position;
        self.select(Location::from(position), Location::from(position))
    }

    /// Returns true if [cell_handle] is the first cell of the first row of
    /// its table, and the whole table has no content - used to decide
    /// whether backspacing at the very start of an empty table should
    /// remove it entirely.
    pub(crate) fn is_first_cell_of_entirely_empty_table(
        &self,
        cell_handle: &DomHandle,
    ) -> bool {
        if cell_handle.index_in_parent() != 0 {
            return false;
        }
        let row_handle = cell_handle.parent_handle();
        if row_handle.index_in_parent() != 0 {
            return false;
        }
        let Some(table_handle) =
            self.find_closest_ancestor_of_kind(&row_handle, DomNodeKind::Table)
        else {
            return false;
        };
        let table = self.state.dom.lookup_container(&table_handle);
        table.children().iter().all(|row| {
            row.as_container().is_some_and(|r| {
                r.children().iter().all(|cell| cell.is_empty())
            })
        })
    }

    /// Finds the (table, cell) handles the cursor is currently inside of.
    fn current_table_and_cell(&self) -> Option<(DomHandle, DomHandle)> {
        let handle = self.current_cursor_handle()?;
        let cell_handle = self.find_closest_ancestor_of_kind_or_self(
            &handle,
            DomNodeKind::TableCell,
        )?;
        let table_handle = self
            .find_closest_ancestor_of_kind(&cell_handle, DomNodeKind::Table)?;
        Some((table_handle, cell_handle))
    }

    /// Finds the (table, row) handles the cursor is currently inside of.
    fn current_table_row_handle(&self) -> Option<(DomHandle, DomHandle)> {
        let handle = self.current_cursor_handle()?;
        let row_handle = self.find_closest_ancestor_of_kind_or_self(
            &handle,
            DomNodeKind::TableRow,
        )?;
        let table_handle = self
            .find_closest_ancestor_of_kind(&row_handle, DomNodeKind::Table)?;
        Some((table_handle, row_handle))
    }

    /// Finds the table handle and the column index of the cell the cursor
    /// is currently inside of.
    fn current_table_and_column(&self) -> Option<(DomHandle, usize)> {
        let (table_handle, cell_handle) = self.current_table_and_cell()?;
        Some((table_handle, cell_handle.index_in_parent()))
    }

    fn current_cursor_handle(&self) -> Option<DomHandle> {
        let (s, e) = self.safe_selection();
        let range = self.state.dom.find_range(s, e);
        let leaf_handle = range.leaves().next().map(|l| l.node_handle.clone());
        leaf_handle.or_else(|| {
            range
                .deepest_block_node(None)
                .map(|l| l.node_handle.clone())
        })
    }
}

fn new_table_node<S: UnicodeString>(rows: usize, columns: usize) -> DomNode<S> {
    let table_rows = (0..rows).map(|_| new_table_row_node(columns)).collect();
    DomNode::new_table(table_rows)
}

fn new_table_row_node<S: UnicodeString>(columns: usize) -> DomNode<S> {
    let cells = (0..columns)
        .map(|_| DomNode::new_table_cell(TableCellType::Data, Vec::new()))
        .collect();
    DomNode::new_table_row(cells)
}

#[cfg(test)]
mod test {
    use crate::tests::testutils_composer_model::{cm, tx};
    use crate::{Location, ToHtml};

    fn html(model: &crate::ComposerModel<widestring::Utf16String>) -> String {
        model.state.dom.to_html().to_string()
    }

    #[test]
    fn insert_table_into_empty_dom() {
        let mut model = cm("|");
        model.insert_table(2, 2);
        assert_eq!(
            html(&model),
            "<table><tbody><tr><td></td><td></td></tr><tr><td></td><td></td></tr></tbody></table>"
        );
    }

    #[test]
    fn insert_table_moves_cursor_into_first_cell() {
        let mut model = cm("|");
        model.insert_table(1, 2);
        assert_eq!(
            tx(&model),
            "<table><tbody><tr><td>|</td><td></td></tr></tbody></table>"
        );
    }

    #[test]
    fn can_type_directly_into_an_empty_cell() {
        let mut model = cm("|");
        model.insert_table(1, 2);
        model.replace_text("hi".into());
        assert_eq!(
            html(&model),
            "<table><tbody><tr><td>hi</td><td></td></tr></tbody></table>"
        );
    }

    #[test]
    fn can_type_into_a_cell_after_tabbing() {
        let mut model = cm("|");
        model.insert_table(1, 2);
        model.move_to_next_cell();
        model.replace_text("hi".into());
        assert_eq!(
            html(&model),
            "<table><tbody><tr><td></td><td>hi</td></tr></tbody></table>"
        );
    }

    #[test]
    fn insert_table_after_text() {
        let mut model = cm("Some text|");
        model.insert_table(1, 1);
        assert_eq!(
            html(&model),
            "<p>Some text</p><table><tbody><tr><td></td></tr></tbody></table>"
        );
    }

    #[test]
    fn add_and_remove_table_row() {
        let mut model = cm("|");
        model.insert_table(1, 2);
        model.add_table_row_after();
        assert_eq!(
            html(&model),
            "<table><tbody><tr><td></td><td></td></tr><tr><td></td><td></td></tr></tbody></table>"
        );
        model.remove_table_row();
        assert_eq!(
            html(&model),
            "<table><tbody><tr><td></td><td></td></tr></tbody></table>"
        );
    }

    #[test]
    fn add_and_remove_table_column() {
        let mut model = cm("|");
        model.insert_table(2, 1);
        model.add_table_column_after();
        assert_eq!(
            html(&model),
            "<table><tbody><tr><td></td><td></td></tr><tr><td></td><td></td></tr></tbody></table>"
        );
        model.remove_table_column();
        assert_eq!(
            html(&model),
            "<table><tbody><tr><td></td></tr><tr><td></td></tr></tbody></table>"
        );
    }

    #[test]
    fn toggle_table_header() {
        let mut model = cm("|");
        model.insert_table(1, 2);
        model.toggle_table_header();
        assert_eq!(
            html(&model),
            "<table><thead><tr><th></th><th></th></tr></thead></table>"
        );
        model.toggle_table_header();
        assert_eq!(
            html(&model),
            "<table><tbody><tr><td></td><td></td></tr></tbody></table>"
        );
    }

    #[test]
    fn remove_table() {
        let mut model = cm("|");
        model.insert_table(1, 1);
        model.remove_table();
        assert_eq!(html(&model), "");
    }

    #[test]
    fn remove_last_row_removes_table() {
        let mut model = cm("|");
        model.insert_table(1, 1);
        model.remove_table_row();
        assert_eq!(html(&model), "");
    }

    #[test]
    fn remove_last_column_removes_table() {
        let mut model = cm("|");
        model.insert_table(1, 1);
        model.remove_table_column();
        assert_eq!(html(&model), "");
    }

    #[test]
    fn tab_moves_to_next_cell() {
        let mut model = cm("|");
        model.insert_table(1, 2);
        model.move_to_next_cell();
        assert_eq!(
            tx(&model),
            "<table><tbody><tr><td></td><td>|</td></tr></tbody></table>"
        );
    }

    #[test]
    fn tab_wraps_to_next_row() {
        let mut model = cm("|");
        model.insert_table(2, 2);
        model.move_to_next_cell();
        model.move_to_next_cell();
        assert_eq!(
            tx(&model),
            "<table><tbody><tr><td></td><td></td></tr><tr><td>|</td><td></td></tr></tbody></table>"
        );
    }

    #[test]
    fn tab_in_last_cell_inserts_a_new_row() {
        let mut model = cm("|");
        model.insert_table(1, 1);
        model.move_to_next_cell();
        assert_eq!(
            tx(&model),
            "<table><tbody><tr><td></td></tr><tr><td>|</td></tr></tbody></table>"
        );
    }

    #[test]
    fn shift_tab_moves_to_previous_cell() {
        let mut model = cm("|");
        model.insert_table(1, 2);
        model.move_to_next_cell();
        model.move_to_previous_cell();
        assert_eq!(
            tx(&model),
            "<table><tbody><tr><td>|</td><td></td></tr></tbody></table>"
        );
    }

    #[test]
    fn shift_tab_wraps_to_previous_row() {
        let mut model = cm("|");
        model.insert_table(2, 2);
        model.move_to_next_cell();
        model.move_to_next_cell();
        model.move_to_previous_cell();
        assert_eq!(
            tx(&model),
            "<table><tbody><tr><td></td><td>|</td></tr><tr><td></td><td></td></tr></tbody></table>"
        );
    }

    #[test]
    fn shift_tab_in_first_cell_does_nothing() {
        let mut model = cm("|");
        model.insert_table(1, 1);
        model.move_to_previous_cell();
        assert_eq!(
            tx(&model),
            "<table><tbody><tr><td>|</td></tr></tbody></table>"
        );
    }

    #[test]
    fn backspace_at_start_of_empty_table_removes_it() {
        let mut model = cm("<p>|</p>");
        model.insert_table(1, 1);
        model.backspace();
        assert_eq!(tx(&model), "|");
    }

    #[test]
    fn backspace_inside_non_empty_table_does_not_remove_it() {
        let mut model = cm("|");
        model.insert_table(1, 1);
        model.replace_text("x".into());
        model.backspace();
        assert_eq!(
            tx(&model),
            "<table><tbody><tr><td>|</td></tr></tbody></table>"
        );
    }

    #[test]
    fn backspace_at_start_of_already_empty_cell_does_not_remove_it() {
        // Regression test: backspacing while the cursor sits at the start of an
        // already-empty cell (as happens naturally after backspacing away its last
        // character) used to merge/remove the cell itself, shrinking the row.
        let mut model = cm("|");
        model.insert_table(1, 2);
        model.replace_text("a".into());
        model.move_to_next_cell();
        model.replace_text("b".into());
        model.backspace(); // empties the 2nd cell
        model.backspace(); // cursor is now at the start of that empty cell
        assert_eq!(
            tx(&model),
            "<table><tbody><tr><td>a</td><td>|</td></tr></tbody></table>"
        );
    }

    #[test]
    fn backspace_at_start_of_non_empty_cell_does_nothing() {
        let mut model = cm("|");
        model.insert_table(1, 2);
        model.replace_text("a".into());
        model.move_to_next_cell();
        model.replace_text("b".into());
        model.move_to_previous_cell(); // cursor at the start of the 1st cell ("a")
        model.backspace();
        assert_eq!(
            tx(&model),
            "<table><tbody><tr><td>|a</td><td>b</td></tr></tbody></table>"
        );
    }

    #[test]
    fn enter_in_empty_cell_inserts_line_break() {
        let mut model = cm("|");
        model.insert_table(1, 1);
        model.enter();
        assert_eq!(
            tx(&model),
            "<table><tbody><tr><td><br />|</td></tr></tbody></table>"
        );
    }

    #[test]
    fn enter_in_cell_with_text_inserts_line_break() {
        let mut model = cm("|");
        model.insert_table(1, 1);
        model.replace_text("ab".into());
        model.select(Location::from(1), Location::from(1));
        model.enter();
        assert_eq!(
            tx(&model),
            "<table><tbody><tr><td>a<br />|b</td></tr></tbody></table>"
        );
    }
}


