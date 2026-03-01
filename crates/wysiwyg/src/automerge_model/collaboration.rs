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

//! Collaborative editing APIs for the Automerge-backed composer model.
//!
//! These methods expose Automerge's CRDT synchronisation primitives in a
//! way that is suitable for sending deltas over Matrix events:
//!
//! 1. **Full document** – [`save_document`] / [`load_document`] for
//!    initial room state or persistence.
//! 2. **Incremental deltas** – [`save_incremental`] / [`save_after`] to
//!    generate small change payloads, and [`receive_changes`] to apply
//!    changes received from other participants.
//! 3. **Version tracking** – [`get_heads`] returns the current Automerge
//!    heads (each a hex-encoded SHA-256 hash) so the sender can include
//!    them in the Matrix event to let recipients compute precise deltas.
//! 4. **Actor identity** – [`set_actor_id`] / [`get_actor_id`] tie the
//!    Automerge document to a Matrix user or device ID so concurrent
//!    edits are attributed correctly.

use automerge::{ActorId, AutoCommit, ChangeHash, ReadDoc};

use super::AutomergeModel;
use crate::ComposerUpdate;

impl AutomergeModel {
    // ------------------------------------------------------------------
    // Full document save & load
    // ------------------------------------------------------------------

    /// Serialise the entire document to a compact binary blob.
    ///
    /// The returned bytes can be stored (e.g. in Matrix room state) and
    /// later restored with [`load_document`].
    pub fn save_document(&mut self) -> Vec<u8> {
        self.doc.save()
    }

    /// Replace the current document with one loaded from the given bytes.
    ///
    /// This is typically used when joining a room: the full document
    /// state is fetched (e.g. from room state) and loaded here.
    ///
    /// Selection is reset to position 0 and undo/redo stacks are cleared.
    pub fn load_document(&mut self, data: &[u8]) -> Result<(), String> {
        let loaded =
            AutoCommit::load(data).map_err(|e| format!("load error: {e}"))?;

        // Re-resolve the text object ID.
        let text_id = loaded
            .get(automerge::ROOT, "content")
            .map_err(|e| format!("missing content key: {e}"))?
            .and_then(|(val, obj_id)| {
                if matches!(val, automerge::Value::Object(automerge::ObjType::Text)) {
                    Some(obj_id)
                } else {
                    None
                }
            })
            .ok_or_else(|| "document has no text object at 'content'".to_owned())?;

        self.doc = loaded;
        self.text_id = text_id;
        self.selection_start = 0;
        self.selection_end = 0;
        self.pending_formats.clear();
        self.undo_stack.clear();
        self.redo_stack.clear();

        Ok(())
    }

    // ------------------------------------------------------------------
    // Incremental deltas
    // ------------------------------------------------------------------

    /// Return the changes made since the last call to [`save_document`]
    /// or [`save_incremental`].
    ///
    /// The returned bytes are suitable for sending as a single Matrix
    /// event payload. Recipients apply them with [`receive_changes`].
    ///
    /// If nothing has changed since the last save, the returned `Vec`
    /// will be empty.
    pub fn save_incremental(&mut self) -> Vec<u8> {
        self.doc.save_incremental()
    }

    /// Return the changes made since the given heads.
    ///
    /// `heads` is a list of hex-encoded change hashes as returned by
    /// [`get_heads`]. This lets you compute a precise delta between
    /// any two points in the document history.
    pub fn save_after(&mut self, heads: &[String]) -> Result<Vec<u8>, String> {
        let hashes = Self::decode_heads(heads)?;
        Ok(self.doc.save_after(&hashes))
    }

    /// Apply remote changes (received e.g. from a Matrix event).
    ///
    /// `data` may be the output of either [`save_document`],
    /// [`save_incremental`], or [`save_after`].
    ///
    /// Returns a [`ComposerUpdate`] so the host UI can re-render.
    pub fn receive_changes(
        &mut self,
        data: &[u8],
    ) -> Result<ComposerUpdate<String>, String> {
        self.doc
            .load_incremental(data)
            .map_err(|e| format!("receive error: {e}"))?;

        Ok(self.create_update_for_current_state())
    }

    /// Merge a full remote document into this one.
    ///
    /// This is useful when two users have been editing independently
    /// and you want to reconcile their state. Both documents keep
    /// their full histories afterward.
    ///
    /// Returns a [`ComposerUpdate`] so the host UI can re-render.
    pub fn merge_remote(
        &mut self,
        remote_bytes: &[u8],
    ) -> Result<ComposerUpdate<String>, String> {
        let mut other = AutoCommit::load(remote_bytes)
            .map_err(|e| format!("load error: {e}"))?;
        self.doc
            .merge(&mut other)
            .map_err(|e| format!("merge error: {e}"))?;

        Ok(self.create_update_for_current_state())
    }

    // ------------------------------------------------------------------
    // Version tracking
    // ------------------------------------------------------------------

    /// Get the current document heads as hex-encoded SHA-256 hashes.
    ///
    /// Include these in your Matrix event so that recipients can use
    /// [`save_after`] to compute the minimal delta.
    pub fn get_heads(&mut self) -> Vec<String> {
        self.doc
            .get_heads()
            .iter()
            .map(|h| hex::encode(h.as_ref()))
            .collect()
    }

    // ------------------------------------------------------------------
    // Actor identity
    // ------------------------------------------------------------------

    /// Get the Automerge actor ID as a hex string.
    ///
    /// By default this is a random UUID. Call [`set_actor_id`] to tie
    /// it to a stable identifier such as the user's Matrix device ID.
    pub fn get_actor_id(&self) -> String {
        hex::encode(self.doc.get_actor().to_bytes())
    }

    /// Set the Automerge actor ID.
    ///
    /// `actor_hex` is a hex-encoded byte string. A good choice is
    /// the user's Matrix device ID or `{user_id}:{device_id}`.
    ///
    /// Must be called **before** any mutations; changing the actor
    /// after edits have been made will cause Automerge to reject
    /// future commits.
    pub fn set_actor_id(&mut self, actor_hex: &str) -> Result<(), String> {
        let bytes =
            hex::decode(actor_hex).map_err(|e| format!("bad hex: {e}"))?;
        self.doc.set_actor(ActorId::from(bytes.as_slice()));
        Ok(())
    }

    // ------------------------------------------------------------------
    // Helper: create an update reflecting the current document state
    // ------------------------------------------------------------------

    pub(crate) fn create_update_for_current_state(
        &self,
    ) -> ComposerUpdate<String> {
        // Delegate to the existing replace_all builder in base.rs
        self.create_update_replace_all()
    }

    // ------------------------------------------------------------------
    // Private helpers
    // ------------------------------------------------------------------

    /// Decode a list of hex-encoded change hash strings into
    /// `ChangeHash` values.
    fn decode_heads(heads: &[String]) -> Result<Vec<ChangeHash>, String> {
        heads
            .iter()
            .map(|h| {
                let bytes = hex::decode(h)
                    .map_err(|e| format!("bad head hex '{h}': {e}"))?;
                if bytes.len() != 32 {
                    return Err(format!(
                        "head hash must be 32 bytes, got {}",
                        bytes.len()
                    ));
                }
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&bytes);
                Ok(ChangeHash(arr))
            })
            .collect()
    }
}

// =====================================================================
// Hex encoding/decoding helpers (minimal, dependency-free)
// =====================================================================

mod hex {
    /// Encode bytes as lowercase hex string.
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }

    /// Decode a hex string into bytes.
    pub fn decode(s: &str) -> Result<Vec<u8>, String> {
        if s.len() % 2 != 0 {
            return Err("odd-length hex string".to_owned());
        }
        (0..s.len())
            .step_by(2)
            .map(|i| {
                u8::from_str_radix(&s[i..i + 2], 16)
                    .map_err(|e| format!("invalid hex at pos {i}: {e}"))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ----- helpers -----

    fn model_with_text(text: &str) -> AutomergeModel {
        let mut m = AutomergeModel::new();
        m.replace_text(text);
        m
    }

    // ----- save / load round-trip -----

    #[test]
    fn save_and_load_round_trip() {
        let mut m = model_with_text("hello world");
        m.select(0, 5);
        m.bold();
        let bytes = m.save_document();

        let mut m2 = AutomergeModel::new();
        m2.load_document(&bytes).unwrap();

        assert_eq!(m2.get_content_as_html(), m.get_content_as_html());
    }

    #[test]
    fn load_document_resets_selection() {
        let mut m = model_with_text("abc");
        let bytes = m.save_document();

        let mut m2 = model_with_text("xyz");
        m2.select(1, 2);
        m2.load_document(&bytes).unwrap();

        let (start, end) = m2.get_selection();
        assert_eq!(start, 0);
        assert_eq!(end, 0);
    }

    #[test]
    fn load_document_clears_undo_redo() {
        let mut m = model_with_text("abc");
        let bytes = m.save_document();

        let mut m2 = model_with_text("xyz");
        // create some undo history
        m2.replace_text("more");
        m2.load_document(&bytes).unwrap();

        // undo should produce a no-op (empty stack)
        let _update = m2.undo();
        // content should still be "abc" from loaded doc
        let html = m2.get_content_as_html();
        assert!(html.contains("abc"), "should still contain abc: {html}");
    }

    #[test]
    fn load_bad_bytes_returns_error() {
        let mut m = AutomergeModel::new();
        let result = m.load_document(b"not valid automerge bytes");
        assert!(result.is_err());
    }

    // ----- incremental deltas -----

    #[test]
    fn save_incremental_captures_changes() {
        let mut m = model_with_text("hello");
        // flush the initial save cursor
        let _full = m.save_document();

        // Now make a change
        m.select(5, 5);
        m.replace_text(" world");

        let delta = m.save_incremental();
        assert!(!delta.is_empty(), "incremental save should be non-empty");
    }

    #[test]
    fn save_incremental_empty_when_no_changes() {
        let mut m = model_with_text("hello");
        let _full = m.save_document();

        // No changes after save
        let delta = m.save_incremental();
        assert!(delta.is_empty(), "incremental save should be empty");
    }

    #[test]
    fn receive_changes_applies_remote_edits() {
        let mut m1 = model_with_text("hello");
        let full = m1.save_document();

        // m2 starts from the same state
        let mut m2 = AutomergeModel::new();
        m2.load_document(&full).unwrap();

        // m1 appends text
        m1.select(5, 5);
        m1.replace_text(" world");
        let delta = m1.save_incremental();

        // m2 receives the delta
        let update = m2.receive_changes(&delta).unwrap();
        assert!(m2.get_content_as_html().contains("world"));
    }

    #[test]
    fn receive_empty_changes_is_ok() {
        // Automerge's load_incremental is lenient with malformed
        // data (it may silently ignore it).  An empty payload is
        // always accepted and treated as a no-op.
        let mut m = model_with_text("hello");
        let result = m.receive_changes(&[]);
        assert!(result.is_ok());
        assert!(m.get_content_as_html().contains("hello"));
    }

    // ----- save_after with specific heads -----

    #[test]
    fn save_after_and_receive() {
        let mut m1 = model_with_text("base");
        let base_bytes = m1.save_document();
        let heads_before = m1.get_heads();

        m1.select(4, 4);
        m1.replace_text(" extended");

        let delta = m1.save_after(&heads_before).unwrap();
        assert!(!delta.is_empty());

        // m2 loads the exact same saved state (same history), then applies delta
        let mut m2 = AutomergeModel::new();
        m2.load_document(&base_bytes).unwrap();
        m2.receive_changes(&delta).unwrap();

        assert!(
            m2.get_content_as_html().contains("extended"),
            "m2 should contain the appended text: {}",
            m2.get_content_as_html()
        );
    }

    #[test]
    fn save_after_bad_hex_returns_error() {
        let mut m = model_with_text("hello");
        let result = m.save_after(&["not_hex_ZZ".to_owned()]);
        assert!(result.is_err());
    }

    #[test]
    fn save_after_wrong_length_returns_error() {
        let mut m = model_with_text("hello");
        let result = m.save_after(&["aabb".to_owned()]);
        assert!(result.is_err());
    }

    // ----- merge -----

    #[test]
    fn merge_remote_combines_documents() {
        let mut m1 = model_with_text("hello");
        // Give m1 a known actor so edits don't conflict on actor id
        m1.set_actor_id("aa".repeat(16).as_str()).unwrap();
        let base = m1.save_document();

        let mut m2 = AutomergeModel::new();
        m2.load_document(&base).unwrap();
        m2.set_actor_id("bb".repeat(16).as_str()).unwrap();

        // m1 edits the beginning
        m1.select(0, 0);
        m1.replace_text("A ");

        // m2 edits the end
        m2.select(5, 5);
        m2.replace_text(" Z");

        let m2_bytes = m2.save_document();
        m1.merge_remote(&m2_bytes).unwrap();

        let html = m1.get_content_as_html();
        assert!(html.contains("A "), "should contain m1's edit");
        assert!(html.contains(" Z"), "should contain m2's edit");
    }

    #[test]
    fn merge_bad_bytes_returns_error() {
        let mut m = AutomergeModel::new();
        let result = m.merge_remote(b"not valid");
        assert!(result.is_err());
    }

    // ----- heads -----

    #[test]
    fn get_heads_returns_hex_strings() {
        let mut m = model_with_text("hello");
        let heads = m.get_heads();
        assert!(!heads.is_empty());
        for h in &heads {
            assert_eq!(h.len(), 64, "SHA-256 hex should be 64 chars");
            assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
        }
    }

    #[test]
    fn heads_change_after_mutation() {
        let mut m = model_with_text("hello");
        let h1 = m.get_heads();
        m.select(5, 5);
        m.replace_text("!");
        let h2 = m.get_heads();
        assert_ne!(h1, h2);
    }

    // ----- actor identity -----

    #[test]
    fn set_and_get_actor_id() {
        let mut m = AutomergeModel::new();
        let id = "aa".repeat(16); // 32 bytes = 64 hex chars
        m.set_actor_id(&id).unwrap();
        assert_eq!(m.get_actor_id(), id);
    }

    #[test]
    fn set_actor_bad_hex_returns_error() {
        let mut m = AutomergeModel::new();
        let result = m.set_actor_id("not_hex!");
        assert!(result.is_err());
    }

    #[test]
    fn default_actor_id_is_valid_hex() {
        let m = AutomergeModel::new();
        let id = m.get_actor_id();
        assert!(!id.is_empty());
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    // ----- concurrent editing scenario -----

    #[test]
    fn concurrent_edits_via_incremental() {
        // Simulate two users editing concurrently via incremental deltas
        let mut m1 = AutomergeModel::new();
        m1.set_actor_id("aa".repeat(16).as_str()).unwrap();
        m1.replace_text("hello");
        let base = m1.save_document();

        let mut m2 = AutomergeModel::new();
        m2.load_document(&base).unwrap();
        m2.set_actor_id("bb".repeat(16).as_str()).unwrap();

        // flush save cursors
        let _ = m1.save_incremental();
        let _ = m2.save_incremental();

        // Both make edits concurrently
        m1.select(5, 5);
        m1.replace_text(" from user1");
        let delta1 = m1.save_incremental();

        m2.select(0, 0);
        m2.replace_text("greeting: ");
        let delta2 = m2.save_incremental();

        // Exchange deltas
        m1.receive_changes(&delta2).unwrap();
        m2.receive_changes(&delta1).unwrap();

        // Both should converge to the same content
        assert_eq!(
            m1.get_content_as_html(),
            m2.get_content_as_html(),
            "documents should converge after exchanging deltas"
        );
    }

    // ----- hex helpers -----

    #[test]
    fn hex_round_trip() {
        let original = b"\x00\x01\x0a\xff";
        let encoded = hex::encode(original);
        assert_eq!(encoded, "00010aff");
        let decoded = hex::decode(&encoded).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn hex_decode_odd_length_error() {
        assert!(hex::decode("abc").is_err());
    }
}
