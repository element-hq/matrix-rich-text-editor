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

//! Defines the [`ComposerModelInterface`] trait — the public API contract
//! that any rich-text model backend must implement.
//!
//! The current [`ComposerModel`] (DOM-tree based) and the future
//! [`AutomergeComposerModel`] (CRDT-based) both target this interface.
//! FFI and platform layers consume the model through these methods.

use std::collections::HashMap;

use crate::{
    ActionState, ComposerAction, ComposerUpdate, InlineFormatType,
    LinkAction, ListType, Location, MentionsState, MenuAction, MenuState,
    PatternKey, SuggestionPattern, TextUpdate,
};

/// A key-value attribute pair, used for link and mention attributes.
/// Mirrors the FFI `Attribute` record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attribute {
    pub key: String,
    pub value: String,
}

/// The public API surface for a rich-text composer model.
///
/// This trait captures the operations that platform layers (iOS, Android, Web)
/// invoke on the model. Implementations handle the document representation,
/// formatting, serialization, and state tracking internally.
///
/// All UTF-16 code-unit offsets are used for positions and ranges, matching
/// platform text APIs.
///
/// Every mutating method returns a [`ComposerUpdate`] that tells the platform
/// what changed (text replacement, selection move, menu state update, etc.).
pub trait ComposerModelInterface {
    // -----------------------------------------------------------------------
    // Construction & content lifecycle
    // -----------------------------------------------------------------------

    /// Replace all content with HTML. Returns the update for the new state.
    fn set_content_from_html(
        &mut self,
        html: &str,
    ) -> ComposerUpdate<String>;

    /// Replace all content with Markdown.
    fn set_content_from_markdown(
        &mut self,
        markdown: &str,
    ) -> ComposerUpdate<String>;

    /// Register additional suggestion trigger patterns (beyond @, #, /).
    fn set_custom_suggestion_patterns(
        &mut self,
        custom_suggestion_patterns: Vec<String>,
    );

    /// Clear all content and return to an empty document.
    fn clear(&mut self) -> ComposerUpdate<String>;

    // -----------------------------------------------------------------------
    // Content access (read-only)
    // -----------------------------------------------------------------------

    /// Internal HTML representation of the document.
    fn get_content_as_html(&self) -> String;

    /// Clean HTML suitable for sending as a Matrix message.
    fn get_content_as_message_html(&self) -> String;

    /// Markdown representation of the document.
    fn get_content_as_markdown(&self) -> String;

    /// Clean Markdown suitable for sending as a Matrix message.
    fn get_content_as_message_markdown(&self) -> String;

    /// Plain text (all formatting stripped).
    fn get_content_as_plain_text(&self) -> String;

    // -----------------------------------------------------------------------
    // Selection
    // -----------------------------------------------------------------------

    /// Set the selection/cursor position (UTF-16 code unit offsets).
    fn select(
        &mut self,
        start: usize,
        end: usize,
    ) -> ComposerUpdate<String>;

    /// Get the current selection as (start, end) UTF-16 offsets.
    fn get_selection(&self) -> (usize, usize);

    // -----------------------------------------------------------------------
    // Text manipulation
    // -----------------------------------------------------------------------

    /// Replace the current selection with text.
    fn replace_text(
        &mut self,
        new_text: String,
    ) -> ComposerUpdate<String>;

    /// Replace a specific range with text (document-level UTF-16 offsets).
    fn replace_text_in(
        &mut self,
        new_text: String,
        start: usize,
        end: usize,
    ) -> ComposerUpdate<String>;

    /// Replace a suggestion pattern match with text.
    fn replace_text_suggestion(
        &mut self,
        new_text: String,
        suggestion: SuggestionPattern,
        append_space: bool,
    ) -> ComposerUpdate<String>;

    /// Delete backward from the cursor (backspace key).
    fn backspace(&mut self) -> ComposerUpdate<String>;

    /// Delete forward from the cursor (delete key).
    fn delete(&mut self) -> ComposerUpdate<String>;

    /// Delete a specific range.
    fn delete_in(
        &mut self,
        start: usize,
        end: usize,
    ) -> ComposerUpdate<String>;

    /// Insert a new line / paragraph break (enter key).
    fn enter(&mut self) -> ComposerUpdate<String>;

    // -----------------------------------------------------------------------
    // Inline formatting (toggles)
    // -----------------------------------------------------------------------

    /// Toggle bold on the current selection.
    fn bold(&mut self) -> ComposerUpdate<String>;

    /// Toggle italic on the current selection.
    fn italic(&mut self) -> ComposerUpdate<String>;

    /// Toggle strikethrough on the current selection.
    fn strike_through(&mut self) -> ComposerUpdate<String>;

    /// Toggle underline on the current selection.
    fn underline(&mut self) -> ComposerUpdate<String>;

    /// Toggle inline code on the current selection.
    fn inline_code(&mut self) -> ComposerUpdate<String>;

    // -----------------------------------------------------------------------
    // Block formatting
    // -----------------------------------------------------------------------

    /// Toggle or convert to an ordered list.
    fn ordered_list(&mut self) -> ComposerUpdate<String>;

    /// Toggle or convert to an unordered list.
    fn unordered_list(&mut self) -> ComposerUpdate<String>;

    /// Increase indentation (nest list item).
    fn indent(&mut self) -> ComposerUpdate<String>;

    /// Decrease indentation (unnest list item).
    fn unindent(&mut self) -> ComposerUpdate<String>;

    /// Toggle a code block around the current selection/block.
    fn code_block(&mut self) -> ComposerUpdate<String>;

    /// Toggle a block quote around the current selection/block.
    fn quote(&mut self) -> ComposerUpdate<String>;

    // -----------------------------------------------------------------------
    // Links
    // -----------------------------------------------------------------------

    /// Set a link URL on the current selection.
    fn set_link(
        &mut self,
        url: String,
        attributes: Vec<Attribute>,
    ) -> ComposerUpdate<String>;

    /// Set a link with explicit display text (replacing selection).
    fn set_link_with_text(
        &mut self,
        url: String,
        text: String,
        attributes: Vec<Attribute>,
    ) -> ComposerUpdate<String>;

    /// Remove all links from the current selection.
    fn remove_links(&mut self) -> ComposerUpdate<String>;

    /// Query what link action is available at the current cursor position.
    fn get_link_action(&self) -> LinkAction<String>;

    // -----------------------------------------------------------------------
    // Mentions
    // -----------------------------------------------------------------------

    /// Insert a user/room mention at the current cursor position.
    fn insert_mention(
        &mut self,
        url: String,
        text: String,
        attributes: Vec<Attribute>,
    ) -> ComposerUpdate<String>;

    /// Insert a mention replacing a suggestion pattern match.
    fn insert_mention_at_suggestion(
        &mut self,
        url: String,
        text: String,
        suggestion: SuggestionPattern,
        attributes: Vec<Attribute>,
    ) -> ComposerUpdate<String>;

    /// Insert an @room mention at the current cursor position.
    fn insert_at_room_mention(
        &mut self,
        attributes: Vec<Attribute>,
    ) -> ComposerUpdate<String>;

    /// Insert an @room mention replacing a suggestion pattern match.
    fn insert_at_room_mention_at_suggestion(
        &mut self,
        suggestion: SuggestionPattern,
        attributes: Vec<Attribute>,
    ) -> ComposerUpdate<String>;

    /// Get the current mentions state (which users/rooms are mentioned).
    fn get_mentions_state(&self) -> MentionsState;

    // -----------------------------------------------------------------------
    // Undo / Redo
    // -----------------------------------------------------------------------

    /// Undo the last editing operation.
    fn undo(&mut self) -> ComposerUpdate<String>;

    /// Redo a previously undone operation.
    fn redo(&mut self) -> ComposerUpdate<String>;

    // -----------------------------------------------------------------------
    // State queries
    // -----------------------------------------------------------------------

    /// Get the current action states for all toolbar buttons.
    /// Maps each [`ComposerAction`] to [`ActionState`] (Enabled / Reversed / Disabled).
    fn action_states(&self) -> HashMap<ComposerAction, ActionState>;

    // -----------------------------------------------------------------------
    // Debug / introspection
    // -----------------------------------------------------------------------

    /// Return a debug tree representation of the internal document model.
    fn to_tree(&self) -> String;
}
