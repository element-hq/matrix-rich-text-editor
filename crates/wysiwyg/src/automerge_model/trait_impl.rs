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

//! Implements [`ComposerModelInterface`] for [`AutomergeModel`],
//! bridging the trait's `String`-based API to the model's methods.

use std::collections::HashMap;

use super::AutomergeModel;
use crate::composer_model_interface::{Attribute, ComposerModelInterface};
use crate::{
    ActionState, ComposerAction, ComposerUpdate, LinkAction,
    MentionsState, SuggestionPattern,
};

impl ComposerModelInterface for AutomergeModel {
    // -------------------------------------------------------------------
    // Construction & content lifecycle
    // -------------------------------------------------------------------

    fn set_content_from_html(
        &mut self,
        html: &str,
    ) -> ComposerUpdate<String> {
        self.set_content_from_html(html)
    }

    fn set_content_from_markdown(
        &mut self,
        markdown: &str,
    ) -> ComposerUpdate<String> {
        self.set_content_from_markdown(markdown)
    }

    fn set_custom_suggestion_patterns(
        &mut self,
        custom_suggestion_patterns: Vec<String>,
    ) {
        self.custom_suggestion_patterns =
            custom_suggestion_patterns.into_iter().collect();
    }

    fn clear(&mut self) -> ComposerUpdate<String> {
        self.clear()
    }

    // -------------------------------------------------------------------
    // Content access (read-only)
    // -------------------------------------------------------------------

    fn get_content_as_html(&self) -> String {
        self.get_content_as_html()
    }

    fn get_content_as_message_html(&self) -> String {
        self.get_content_as_message_html()
    }

    fn get_content_as_markdown(&self) -> String {
        self.get_content_as_markdown()
    }

    fn get_content_as_message_markdown(&self) -> String {
        self.get_content_as_message_markdown()
    }

    fn get_content_as_plain_text(&self) -> String {
        self.get_content_as_plain_text()
    }

    // -------------------------------------------------------------------
    // Selection
    // -------------------------------------------------------------------

    fn select(
        &mut self,
        start: usize,
        end: usize,
    ) -> ComposerUpdate<String> {
        self.select(start, end)
    }

    fn get_selection(&self) -> (usize, usize) {
        self.get_selection()
    }

    // -------------------------------------------------------------------
    // Text manipulation
    // -------------------------------------------------------------------

    fn replace_text(
        &mut self,
        new_text: String,
    ) -> ComposerUpdate<String> {
        self.replace_text(&new_text)
    }

    fn replace_text_in(
        &mut self,
        new_text: String,
        start: usize,
        end: usize,
    ) -> ComposerUpdate<String> {
        self.replace_text_in(&new_text, start, end)
    }

    fn replace_text_suggestion(
        &mut self,
        new_text: String,
        suggestion: SuggestionPattern,
        append_space: bool,
    ) -> ComposerUpdate<String> {
        self.replace_text_suggestion(&new_text, &suggestion, append_space)
    }

    fn backspace(&mut self) -> ComposerUpdate<String> {
        self.backspace()
    }

    fn delete(&mut self) -> ComposerUpdate<String> {
        self.delete()
    }

    fn delete_in(
        &mut self,
        start: usize,
        end: usize,
    ) -> ComposerUpdate<String> {
        self.delete_in(start, end)
    }

    fn enter(&mut self) -> ComposerUpdate<String> {
        self.enter()
    }

    // -------------------------------------------------------------------
    // Inline formatting
    // -------------------------------------------------------------------

    fn bold(&mut self) -> ComposerUpdate<String> {
        self.bold()
    }

    fn italic(&mut self) -> ComposerUpdate<String> {
        self.italic()
    }

    fn strike_through(&mut self) -> ComposerUpdate<String> {
        self.strike_through()
    }

    fn underline(&mut self) -> ComposerUpdate<String> {
        self.underline()
    }

    fn inline_code(&mut self) -> ComposerUpdate<String> {
        self.inline_code()
    }

    // -------------------------------------------------------------------
    // Block formatting
    // -------------------------------------------------------------------

    fn ordered_list(&mut self) -> ComposerUpdate<String> {
        self.ordered_list()
    }

    fn unordered_list(&mut self) -> ComposerUpdate<String> {
        self.unordered_list()
    }

    fn indent(&mut self) -> ComposerUpdate<String> {
        self.indent()
    }

    fn unindent(&mut self) -> ComposerUpdate<String> {
        self.unindent()
    }

    fn code_block(&mut self) -> ComposerUpdate<String> {
        self.code_block()
    }

    fn quote(&mut self) -> ComposerUpdate<String> {
        self.quote()
    }

    // -------------------------------------------------------------------
    // Links
    // -------------------------------------------------------------------

    fn set_link(
        &mut self,
        url: String,
        attributes: Vec<Attribute>,
    ) -> ComposerUpdate<String> {
        self.set_link(&url, &attributes)
    }

    fn set_link_with_text(
        &mut self,
        url: String,
        text: String,
        attributes: Vec<Attribute>,
    ) -> ComposerUpdate<String> {
        self.set_link_with_text(&url, &text, &attributes)
    }

    fn remove_links(&mut self) -> ComposerUpdate<String> {
        self.remove_links()
    }

    fn get_link_action(&self) -> LinkAction<String> {
        self.get_link_action()
    }

    // -------------------------------------------------------------------
    // Mentions
    // -------------------------------------------------------------------

    fn insert_mention(
        &mut self,
        url: String,
        text: String,
        attributes: Vec<Attribute>,
    ) -> ComposerUpdate<String> {
        self.insert_mention(&url, &text, &attributes)
    }

    fn insert_mention_at_suggestion(
        &mut self,
        url: String,
        text: String,
        suggestion: SuggestionPattern,
        attributes: Vec<Attribute>,
    ) -> ComposerUpdate<String> {
        self.insert_mention_at_suggestion(
            &url,
            &text,
            &suggestion,
            &attributes,
        )
    }

    fn insert_at_room_mention(
        &mut self,
        attributes: Vec<Attribute>,
    ) -> ComposerUpdate<String> {
        self.insert_at_room_mention(&attributes)
    }

    fn insert_at_room_mention_at_suggestion(
        &mut self,
        suggestion: SuggestionPattern,
        attributes: Vec<Attribute>,
    ) -> ComposerUpdate<String> {
        self.insert_at_room_mention_at_suggestion(
            &suggestion,
            &attributes,
        )
    }

    fn get_mentions_state(&self) -> MentionsState {
        self.get_mentions_state()
    }

    // -------------------------------------------------------------------
    // Undo / Redo
    // -------------------------------------------------------------------

    fn undo(&mut self) -> ComposerUpdate<String> {
        self.undo()
    }

    fn redo(&mut self) -> ComposerUpdate<String> {
        self.redo()
    }

    // -------------------------------------------------------------------
    // State queries
    // -------------------------------------------------------------------

    fn action_states(&self) -> HashMap<ComposerAction, ActionState> {
        self.action_states()
    }

    // -------------------------------------------------------------------
    // Collaboration (CRDT sync)
    // -------------------------------------------------------------------

    fn save_document(&mut self) -> Vec<u8> {
        self.save_document()
    }

    fn load_document(&mut self, data: &[u8]) -> Result<(), String> {
        self.load_document(data)
    }

    fn save_incremental(&mut self) -> Vec<u8> {
        self.save_incremental()
    }

    fn save_after(&mut self, heads: &[String]) -> Result<Vec<u8>, String> {
        self.save_after(heads)
    }

    fn receive_changes(
        &mut self,
        data: &[u8],
    ) -> Result<ComposerUpdate<String>, String> {
        self.receive_changes(data)
    }

    fn merge_remote(
        &mut self,
        remote_bytes: &[u8],
    ) -> Result<ComposerUpdate<String>, String> {
        self.merge_remote(remote_bytes)
    }

    fn get_heads(&mut self) -> Vec<String> {
        self.get_heads()
    }

    fn get_actor_id(&self) -> String {
        self.get_actor_id()
    }

    fn set_actor_id(&mut self, actor_hex: &str) -> Result<(), String> {
        self.set_actor_id(actor_hex)
    }

    // -------------------------------------------------------------------
    // Debug
    // -------------------------------------------------------------------

    fn to_tree(&self) -> String {
        self.to_tree()
    }
}
