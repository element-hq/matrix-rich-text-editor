use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::ffi_composer_state::ComposerState;
use crate::ffi_composer_update::ComposerUpdate;
use crate::ffi_link_actions::LinkAction;
use crate::ffi_mentions_state::MentionsState;
use crate::ffi_collaboration_error::CollaborationError;
use crate::into_ffi::IntoFfi;
use crate::{ActionState, ComposerAction, SuggestionPattern};

#[derive(uniffi::Object)]
pub struct ComposerModel {
    inner: Mutex<wysiwyg::AutomergeModel>,
}

impl Default for ComposerModel {
    fn default() -> Self {
        Self::new()
    }
}

impl ComposerModel {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(wysiwyg::AutomergeModel::new()),
        }
    }
}

/// Convert an FFI `Attribute` into the core crate's `Attribute`.
fn to_core_attrs(attrs: &[Attribute]) -> Vec<wysiwyg::Attribute> {
    attrs
        .iter()
        .map(|a| wysiwyg::Attribute {
            key: a.key.clone(),
            value: a.value.clone(),
        })
        .collect()
}

#[uniffi::export]
impl ComposerModel {
    pub fn set_content_from_html(
        self: &Arc<Self>,
        html: String,
    ) -> Result<Arc<ComposerUpdate>, crate::ffi_dom_creation_error::DomCreationError> {
        let update = self.inner.lock().unwrap().set_content_from_html(&html);
        Ok(Arc::new(ComposerUpdate::from(update)))
    }

    pub fn set_content_from_markdown(
        self: &Arc<Self>,
        markdown: String,
    ) -> Result<Arc<ComposerUpdate>, crate::ffi_dom_creation_error::DomCreationError> {
        let update = self
            .inner
            .lock()
            .unwrap()
            .set_content_from_markdown(&markdown);
        Ok(Arc::new(ComposerUpdate::from(update)))
    }

    pub fn set_custom_suggestion_patterns(
        self: &Arc<Self>,
        custom_suggestion_patterns: Vec<String>,
    ) {
        self.inner
            .lock()
            .unwrap()
            .custom_suggestion_patterns = custom_suggestion_patterns.into_iter().collect();
    }

    pub fn get_content_as_html(self: &Arc<Self>) -> String {
        self.inner.lock().unwrap().get_content_as_html()
    }

    pub fn get_content_as_message_html(self: &Arc<Self>) -> String {
        self.inner.lock().unwrap().get_content_as_message_html()
    }

    pub fn get_content_as_markdown(self: &Arc<Self>) -> String {
        self.inner.lock().unwrap().get_content_as_markdown()
    }

    pub fn get_content_as_message_markdown(self: &Arc<Self>) -> String {
        self.inner.lock().unwrap().get_content_as_message_markdown()
    }

    pub fn get_content_as_plain_text(self: &Arc<Self>) -> String {
        self.inner.lock().unwrap().get_content_as_plain_text()
    }

    pub fn clear(self: &Arc<Self>) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(self.inner.lock().unwrap().clear()))
    }

    pub fn select(
        self: &Arc<Self>,
        start_utf16_codeunit: u32,
        end_utf16_codeunit: u32,
    ) -> Arc<ComposerUpdate> {
        let start = usize::try_from(start_utf16_codeunit).unwrap();
        let end = usize::try_from(end_utf16_codeunit).unwrap();
        Arc::new(ComposerUpdate::from(
            self.inner.lock().unwrap().select(start, end),
        ))
    }

    pub fn replace_text(
        self: &Arc<Self>,
        new_text: String,
    ) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(
            self.inner.lock().unwrap().replace_text(&new_text),
        ))
    }

    pub fn replace_text_in(
        self: &Arc<Self>,
        new_text: String,
        start: u32,
        end: u32,
    ) -> Arc<ComposerUpdate> {
        let start = usize::try_from(start).unwrap();
        let end = usize::try_from(end).unwrap();
        Arc::new(ComposerUpdate::from(
            self.inner
                .lock()
                .unwrap()
                .replace_text_in(&new_text, start, end),
        ))
    }

    pub fn replace_text_suggestion(
        self: &Arc<Self>,
        new_text: String,
        suggestion: SuggestionPattern,
        append_space: bool,
    ) -> Arc<ComposerUpdate> {
        let suggestion = wysiwyg::SuggestionPattern::from(suggestion);
        Arc::new(ComposerUpdate::from(
            self.inner.lock().unwrap().replace_text_suggestion(
                &new_text,
                &suggestion,
                append_space,
            ),
        ))
    }

    pub fn backspace(self: &Arc<Self>) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(self.inner.lock().unwrap().backspace()))
    }

    pub fn delete(self: &Arc<Self>) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(self.inner.lock().unwrap().delete()))
    }

    pub fn delete_in(
        self: &Arc<Self>,
        start: u32,
        end: u32,
    ) -> Arc<ComposerUpdate> {
        let start = usize::try_from(start).unwrap();
        let end = usize::try_from(end).unwrap();
        Arc::new(ComposerUpdate::from(
            self.inner.lock().unwrap().delete_in(start, end),
        ))
    }

    pub fn enter(self: &Arc<Self>) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(self.inner.lock().unwrap().enter()))
    }

    pub fn bold(self: &Arc<Self>) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(self.inner.lock().unwrap().bold()))
    }

    pub fn italic(self: &Arc<Self>) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(self.inner.lock().unwrap().italic()))
    }

    pub fn strike_through(self: &Arc<Self>) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(
            self.inner.lock().unwrap().strike_through(),
        ))
    }

    pub fn underline(self: &Arc<Self>) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(self.inner.lock().unwrap().underline()))
    }

    pub fn inline_code(self: &Arc<Self>) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(
            self.inner.lock().unwrap().inline_code(),
        ))
    }

    pub fn code_block(self: &Arc<Self>) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(
            self.inner.lock().unwrap().code_block(),
        ))
    }

    pub fn quote(self: &Arc<Self>) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(self.inner.lock().unwrap().quote()))
    }

    pub fn ordered_list(self: &Arc<Self>) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(
            self.inner.lock().unwrap().ordered_list(),
        ))
    }

    pub fn unordered_list(self: &Arc<Self>) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(
            self.inner.lock().unwrap().unordered_list(),
        ))
    }

    pub fn undo(self: &Arc<Self>) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(self.inner.lock().unwrap().undo()))
    }

    pub fn redo(self: &Arc<Self>) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(self.inner.lock().unwrap().redo()))
    }

    pub fn set_link(
        self: &Arc<Self>,
        url: String,
        attributes: Vec<Attribute>,
    ) -> Arc<ComposerUpdate> {
        let attrs = to_core_attrs(&attributes);
        Arc::new(ComposerUpdate::from(
            self.inner.lock().unwrap().set_link(&url, &attrs),
        ))
    }

    pub fn set_link_with_text(
        self: &Arc<Self>,
        url: String,
        text: String,
        attributes: Vec<Attribute>,
    ) -> Arc<ComposerUpdate> {
        let escaped = html_escape::encode_safe(&text).to_string();
        let attrs = to_core_attrs(&attributes);
        Arc::new(ComposerUpdate::from(
            self.inner
                .lock()
                .unwrap()
                .set_link_with_text(&url, &escaped, &attrs),
        ))
    }

    /// Creates an at-room mention node and inserts it into the composer at the current selection
    pub fn insert_at_room_mention(self: &Arc<Self>) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(
            self.inner.lock().unwrap().insert_at_room_mention(&[]),
        ))
    }

    /// Creates a mention node and inserts it into the composer at the current selection
    pub fn insert_mention(
        self: &Arc<Self>,
        url: String,
        text: String,
        _attributes: Vec<Attribute>, // TODO remove attributes
    ) -> Arc<ComposerUpdate> {
        let escaped = html_escape::encode_safe(&text).to_string();
        Arc::new(ComposerUpdate::from(
            self.inner.lock().unwrap().insert_mention(&url, &escaped, &[]),
        ))
    }

    /// Creates an at-room mention node and inserts it into the composer, replacing the
    /// text content defined by the suggestion
    pub fn insert_at_room_mention_at_suggestion(
        self: &Arc<Self>,
        suggestion: SuggestionPattern,
    ) -> Arc<ComposerUpdate> {
        let suggestion = wysiwyg::SuggestionPattern::from(suggestion);
        Arc::new(ComposerUpdate::from(
            self.inner
                .lock()
                .unwrap()
                .insert_at_room_mention_at_suggestion(&suggestion, &[]),
        ))
    }

    /// Creates a mention node and inserts it into the composer, replacing the
    /// text content defined by the suggestion
    pub fn insert_mention_at_suggestion(
        self: &Arc<Self>,
        url: String,
        text: String,
        suggestion: SuggestionPattern,
        _attributes: Vec<Attribute>, // TODO remove attributes
    ) -> Arc<ComposerUpdate> {
        let escaped = html_escape::encode_safe(&text).to_string();
        let suggestion = wysiwyg::SuggestionPattern::from(suggestion);
        Arc::new(ComposerUpdate::from(
            self.inner
                .lock()
                .unwrap()
                .insert_mention_at_suggestion(&url, &escaped, &suggestion, &[]),
        ))
    }

    pub fn remove_links(self: &Arc<Self>) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(
            self.inner.lock().unwrap().remove_links(),
        ))
    }

    pub fn indent(self: &Arc<Self>) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(self.inner.lock().unwrap().indent()))
    }

    pub fn unindent(self: &Arc<Self>) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(self.inner.lock().unwrap().unindent()))
    }

    pub fn to_example_format(self: &Arc<Self>) -> String {
        // Not available on AutomergeModel — return the HTML representation instead
        self.inner.lock().unwrap().get_content_as_html()
    }

    pub fn to_tree(self: &Arc<Self>) -> String {
        self.inner.lock().unwrap().to_tree()
    }

    pub fn get_current_dom_state(self: &Arc<Self>) -> ComposerState {
        let inner = self.inner.lock().unwrap();
        let html = inner.get_content_as_html();
        let (start, end) = inner.get_selection();
        ComposerState {
            html: html.encode_utf16().collect(),
            start: u32::try_from(start).unwrap(),
            end: u32::try_from(end).unwrap(),
        }
    }

    pub fn action_states(
        self: &Arc<Self>,
    ) -> HashMap<ComposerAction, ActionState> {
        self.inner.lock().unwrap().action_states().into_ffi()
    }

    pub fn get_link_action(self: &Arc<Self>) -> LinkAction {
        self.inner.lock().unwrap().get_link_action().into()
    }

    pub fn get_mentions_state(self: &Arc<Self>) -> MentionsState {
        self.inner.lock().unwrap().get_mentions_state().into()
    }

    /// Returns a flat projection of all blocks and their inline runs.
    /// Offsets are UTF-16 code units, consistent with select() and replace_text_in().
    pub fn get_block_projections(self: &Arc<Self>) -> Vec<crate::ffi_block_projection::FfiBlockProjection> {
        let inner = self.inner.lock().unwrap();
        inner.get_block_projections()
            .iter()
            .map(crate::ffi_block_projection::FfiBlockProjection::from)
            .collect()
    }

    // ------------------------------------------------------------------
    // Collaboration (CRDT sync)
    // ------------------------------------------------------------------

    /// Serialise the entire document to a compact binary blob.
    ///
    /// Store the returned bytes in Matrix room state or as a file.
    /// Restore with `load_document()`.
    pub fn save_document(self: &Arc<Self>) -> Vec<u8> {
        self.inner.lock().unwrap().save_document()
    }

    /// Replace the current document state with one loaded from bytes
    /// previously returned by `save_document()`.
    pub fn load_document(
        self: &Arc<Self>,
        data: Vec<u8>,
    ) -> Result<(), CollaborationError> {
        self.inner
            .lock()
            .unwrap()
            .load_document(&data)
            .map_err(|e| CollaborationError::LoadError { reason: e })
    }

    /// Return the changes made since the last save.
    ///
    /// Send the returned bytes as a Matrix event payload.
    /// Recipients apply them with `receive_changes()`.
    /// Returns an empty `Vec` if nothing has changed.
    pub fn save_incremental(self: &Arc<Self>) -> Vec<u8> {
        self.inner.lock().unwrap().save_incremental()
    }

    /// Return the changes made since the given document heads.
    ///
    /// `heads` is a list of hex-encoded SHA-256 hashes as returned by
    /// `get_heads()`.
    pub fn save_after(
        self: &Arc<Self>,
        heads: Vec<String>,
    ) -> Result<Vec<u8>, CollaborationError> {
        self.inner
            .lock()
            .unwrap()
            .save_after(&heads)
            .map_err(|e| CollaborationError::InvalidHeads { reason: e })
    }

    /// Apply remote changes received from another participant.
    ///
    /// `data` can be from `save_document()`, `save_incremental()`, or
    /// `save_after()`.  Returns a `ComposerUpdate` so the UI can re-render.
    pub fn receive_changes(
        self: &Arc<Self>,
        data: Vec<u8>,
    ) -> Result<Arc<ComposerUpdate>, CollaborationError> {
        let update = self
            .inner
            .lock()
            .unwrap()
            .receive_changes(&data)
            .map_err(|e| CollaborationError::ReceiveError { reason: e })?;
        Ok(Arc::new(ComposerUpdate::from(update)))
    }

    /// Merge a complete remote document into this one.
    ///
    /// Useful for reconciling two diverged documents.
    pub fn merge_remote(
        self: &Arc<Self>,
        remote_bytes: Vec<u8>,
    ) -> Result<Arc<ComposerUpdate>, CollaborationError> {
        let update = self
            .inner
            .lock()
            .unwrap()
            .merge_remote(&remote_bytes)
            .map_err(|e| CollaborationError::MergeError { reason: e })?;
        Ok(Arc::new(ComposerUpdate::from(update)))
    }

    /// Get the current document heads as hex-encoded SHA-256 hashes.
    ///
    /// Include these in your Matrix event so recipients can compute
    /// the minimal delta with `save_after()`.
    pub fn get_heads(self: &Arc<Self>) -> Vec<String> {
        self.inner.lock().unwrap().get_heads()
    }

    /// Get the Automerge actor ID as a hex string.
    pub fn get_actor_id(self: &Arc<Self>) -> String {
        self.inner.lock().unwrap().get_actor_id()
    }

    /// Set the Automerge actor ID.
    ///
    /// Pass a hex-encoded byte string. A good choice is the user's
    /// Matrix device ID or `{user_id}:{device_id}` encoded as hex.
    /// Must be called **before** any mutations.
    pub fn set_actor_id(
        self: &Arc<Self>,
        actor_hex: String,
    ) -> Result<(), CollaborationError> {
        self.inner
            .lock()
            .unwrap()
            .set_actor_id(&actor_hex)
            .map_err(|e| CollaborationError::InvalidActorId { reason: e })
    }

    /// Force a panic for test purposes
    pub fn debug_panic(self: &Arc<Self>) {
        #[cfg(debug_assertions)]
        panic!("This should only happen in tests.");
    }
}

#[derive(uniffi::Record)]
pub struct Attribute {
    pub key: String,
    pub value: String,
}
