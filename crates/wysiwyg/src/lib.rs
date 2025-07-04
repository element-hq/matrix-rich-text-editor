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

mod action_state;
mod char;
mod composer_action;
mod composer_model;
mod composer_state;
mod composer_update;
mod dom;
mod format_type;
mod link_action;
mod list_type;
mod location;
mod mentions_state;
mod menu_action;
mod menu_state;
mod pattern_key;
mod suggestion_pattern;
mod tests;
mod text_update;

pub use crate::action_state::ActionState;
pub use crate::composer_action::ComposerAction;
pub use crate::composer_model::ComposerModel;
pub use crate::composer_state::ComposerState;
pub use crate::composer_update::ComposerUpdate;
pub use crate::dom::nodes::DomNode;
pub use crate::dom::parser::parse;
pub use crate::dom::DomCreationError;
pub use crate::dom::DomHandle;
pub use crate::dom::HtmlParseError;
pub use crate::dom::HtmlSource;
pub use crate::dom::MarkdownParseError;
pub use crate::dom::ToHtml;
pub use crate::dom::ToRawText;
pub use crate::dom::ToTree;
pub use crate::dom::UnicodeString;
pub use crate::dom::{MarkdownError, ToMarkdown};
pub use crate::format_type::InlineFormatType;
pub use crate::link_action::LinkAction;
pub use crate::link_action::LinkActionUpdate;
pub use crate::list_type::ListType;
pub use crate::location::Location;
pub use crate::mentions_state::MentionsState;
pub use crate::menu_action::MenuAction;
pub use crate::menu_action::MenuActionSuggestion;
pub use crate::menu_state::MenuState;
pub use crate::menu_state::MenuStateUpdate;
pub use crate::pattern_key::PatternKey;
pub use crate::suggestion_pattern::SuggestionPattern;
pub use crate::text_update::ReplaceAll;
pub use crate::text_update::Selection;
pub use crate::text_update::TextUpdate;
