// Copyright 2022 The Matrix.org Foundation C.I.C.
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
pub mod replace_html;
pub mod replace_text;
pub mod selection;
pub mod undo_redo;

pub use base::ComposerModel;
