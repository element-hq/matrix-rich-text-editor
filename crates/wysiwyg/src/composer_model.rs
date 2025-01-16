// Copyright 2024 New Vector Ltd.
// Copyright 2022 The Matrix.org Foundation C.I.C.
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.

pub mod base;
pub mod code_block;
pub mod delete_text;
pub mod example_format;
pub mod format;
mod format_inline_code;
pub mod hyperlinks;
pub mod lists;
pub mod mentions;
pub mod menu_action;
pub mod menu_state;
pub mod new_lines;
pub mod quotes;
pub mod replace_text;
pub mod selection;
pub mod undo_redo;

pub use base::ComposerModel;
