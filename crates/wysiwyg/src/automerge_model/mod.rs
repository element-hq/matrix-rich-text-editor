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

//! Automerge-backed rich text composer model.
//!
//! This module provides [`AutomergeModel`], an implementation of
//! [`ComposerModelInterface`] backed by an Automerge CRDT document.
//! Rich text is stored as an Automerge text object with Peritext-style
//! marks and inline block markers.

mod base;
mod block_ops;
mod block_projections;
mod content_access;
mod formatting;
mod links;
mod mentions;
mod selection;
mod spans_html;
mod state_query;
mod text_ops;
mod trait_impl;
mod undo_redo;

pub use base::AutomergeModel;
