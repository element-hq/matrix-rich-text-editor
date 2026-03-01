# Migration Plan: Matrix Rich Text Editor → Automerge CRDT Backend

## Executive Summary

This document outlines a plan to migrate the Matrix Rich Text Editor from its current single-writer, DOM-tree-based Rust core (`ComposerModel`) to an Automerge CRDT-backed document model. This migration would replace the custom document representation and editing logic with Automerge's conflict-free replicated data types, gaining real-time collaborative editing capabilities while preserving the existing platform integration layers (iOS, Android, Web).

---

## 1. Architecture Comparison

### Current Architecture

```
Platform UI (iOS / Android / Web)
        │
    FFI / WASM binding layer
        │
    ComposerModel<Utf16String>  ← single-writer, imperative
        │
    Custom DOM tree (DomNode / ContainerNode / TextNode)
        │
    HTML serialization / parsing (html5ever / DOMParser)
```

**Key characteristics:**
- Internal document model is a **mutable tree** of `DomNode` variants (Container, Text, LineBreak, Mention)
- Editing operations mutate the tree in-place and return a `ComposerUpdate`
- `ComposerUpdate::TextUpdate::ReplaceAll` sends **complete HTML** back to the platform on every edit
- Undo/redo clones the **entire** `ComposerState` per operation
- No collaboration support — single-writer only

### Target Architecture (Automerge)

```
Platform UI (iOS / Android / Web)
        │
    Adapter layer (new — translates platform events ↔ Automerge ops)
        │
    Automerge Document  ← CRDT, multi-writer
        │
    Text sequence + Marks (Peritext) + Block markers
        │
    Binary sync protocol / change exchange
```

**Key characteristics:**
- Document is an **immutable CRDT** — `change()` returns a new snapshot
- Rich text stored as a **character sequence** with overlay **marks** (Peritext algorithm) and inline **block markers**
- Granular **patch callbacks** enable incremental UI updates instead of full-HTML replacement
- Built-in sync protocol for multi-writer collaboration
- Deterministic conflict resolution across all peers

---

## 2. Concept Mapping

| Matrix RTE Concept | Automerge Equivalent | Notes |
|---|---|---|
| `Dom<S>` tree | `doc.text` (Text CRDT sequence) | Flat sequence + marks vs. nested tree |
| `ContainerNode::Formatting(Bold)` | `mark(d, ["text"], range, "bold", true)` | Marks are overlay, not structural wrappers |
| `ContainerNode::Link(url)` | `mark(d, ["text"], range, "link", url)` | Link as a mark with `expand: "none"` |
| `ContainerNode::Paragraph` | `splitBlock(d, ["text"], idx, {type:"paragraph"})` | Block markers as inline `\uFFFC` objects |
| `ContainerNode::List(Ordered)` | Block marker `{type: "ordered-list-item", parents: [...]}` | Nesting via `parents` array |
| `ContainerNode::CodeBlock` | Block marker `{type: "codeblock"}` + no marks | Need to suppress mark application inside |
| `ContainerNode::Quote` | Block marker `{type: "blockquote"}` | Or via `parents` nesting |
| `MentionNode` | Custom mark `"mention"` with URI value, or inline object | See §5 for discussion |
| `TextNode` | Characters in the text sequence | Individual character CRDTs |
| `LineBreakNode` | `\n` character or block split | Depends on semantics desired |
| `DomHandle` (path-based addressing) | `Cursor` (stable OpId-based position) | Cursors survive concurrent edits |
| `ComposerUpdate::TextUpdate::ReplaceAll` | `patchCallback` with `Patch[]` | Incremental vs. full replacement |
| `ComposerState.previous_states` (undo) | External undo manager tracking `Heads` | Automerge has no built-in undo |
| `Location` (UTF-16 offset) | UTF-16 index (WASM uses `utf16-indexing` feature) | Direct compatibility |
| `MenuState` / `ActionState` | Computed from `marksAt(doc, ["text"], cursor)` | Must be built in the adapter layer |
| `get_content_as_html()` | `spans(doc, ["text"])` → render to HTML | Need a spans-to-HTML renderer |
| `set_content_from_html(html)` | Parse HTML → `updateSpans(d, ["text"], spans)` | Need an HTML-to-spans parser |

---

## 3. Migration Strategy: Phased Approach

### Phase 0: Proof of Concept (Web Only)
**Goal:** Validate that Automerge's rich text model can represent all content the current editor supports, and that the editing experience is equivalent.

**Deliverables:**
1. A standalone web prototype using `@automerge/automerge` (JS) with a `contentEditable` div
2. Demonstrate: text editing, bold/italic/underline/strikethrough/inline-code, links, ordered/unordered lists, code blocks, quotes, mentions, undo/redo
3. Demonstrate: two-peer sync via the Automerge sync protocol
4. Benchmark: compare latency/frame-rate of patch-based updates vs. the current `innerHTML` replacement

**Key questions to answer:**
- Can Automerge marks model all five inline formats + links + mentions?
- Can block markers + `parents` model nested lists, code blocks, and quotes?
- Is `updateSpans()` reliable enough for HTML → Automerge import?
- What is the performance profile for typical chat-message-sized content?

### Phase 1: New Adapter Layer (Replace Core, Keep Bindings)
**Goal:** Build a new `ComposerModel` implementation backed by Automerge that satisfies the same `ComposerUpdate` contract, allowing existing platform layers to work unchanged.

#### 1a. Define the Automerge Document Schema

```typescript
// Automerge document schema
interface RichTextDocument {
  text: string  // Automerge Text type — has marks + blocks
}
```

**Mark names and expand behavior:**

| Format | Mark name | `expand` | Value type |
|--------|-----------|----------|------------|
| Bold | `"bold"` | `"both"` | `true` |
| Italic | `"italic"` | `"both"` | `true` |
| Underline | `"underline"` | `"both"` | `true` |
| Strikethrough | `"strikethrough"` | `"both"` | `true` |
| Inline code | `"inline_code"` | `"both"` | `true` |
| Link | `"link"` | `"none"` | `string` (URL) |
| Mention | `"mention"` | `"none"` | `string` (matrix URI) |

**Block marker schema:**

```typescript
interface BlockMarker {
  type: "paragraph" | "ordered_list_item" | "unordered_list_item" 
      | "code_block" | "quote"
  parents: string[]  // ancestor block types for nesting
  attrs: {
    // list-specific: indent level, etc.
    [key: string]: any
  }
}
```

#### 1b. Build `AutomergeComposerModel` (Rust)

Create a new crate or module that wraps `automerge::AutoCommit` and exposes the same method signatures as the current `ComposerModel`:

```rust
pub struct AutomergeComposerModel {
    doc: automerge::AutoCommit,
    /// Undo stack: stored as (Heads, selection) pairs
    undo_stack: Vec<(Vec<ChangeHash>, Selection)>,
    redo_stack: Vec<(Vec<ChangeHash>, Selection)>,
    /// Current selection (UTF-16 offsets)
    selection: Selection,
    /// Toggled formats for collapsed-cursor formatting
    pending_formats: HashSet<InlineFormatType>,
}

impl AutomergeComposerModel {
    // --- Text operations ---
    pub fn replace_text(&mut self, new_text: Utf16String) -> ComposerUpdate { ... }
    pub fn backspace(&mut self) -> ComposerUpdate { ... }
    pub fn delete(&mut self) -> ComposerUpdate { ... }
    pub fn enter(&mut self) -> ComposerUpdate { ... }

    // --- Inline formatting ---
    pub fn bold(&mut self) -> ComposerUpdate { ... }
    pub fn italic(&mut self) -> ComposerUpdate { ... }
    pub fn strike_through(&mut self) -> ComposerUpdate { ... }
    pub fn underline(&mut self) -> ComposerUpdate { ... }
    pub fn inline_code(&mut self) -> ComposerUpdate { ... }

    // --- Block formatting ---
    pub fn ordered_list(&mut self) -> ComposerUpdate { ... }
    pub fn unordered_list(&mut self) -> ComposerUpdate { ... }
    pub fn indent(&mut self) -> ComposerUpdate { ... }
    pub fn unindent(&mut self) -> ComposerUpdate { ... }
    pub fn code_block(&mut self) -> ComposerUpdate { ... }
    pub fn quote(&mut self) -> ComposerUpdate { ... }

    // --- Links & mentions ---
    pub fn set_link(&mut self, url: Utf16String, attrs: Vec<Attribute>) -> ComposerUpdate { ... }
    pub fn insert_mention(&mut self, url: Utf16String, text: Utf16String, attrs: Vec<Attribute>) -> ComposerUpdate { ... }

    // --- Collaboration (NEW) ---
    pub fn generate_sync_message(&mut self, sync_state: &mut SyncState) -> Option<Vec<u8>> { ... }
    pub fn receive_sync_message(&mut self, sync_state: &mut SyncState, message: Vec<u8>) -> ComposerUpdate { ... }
    pub fn get_last_change(&self) -> Option<Vec<u8>> { ... }
    pub fn apply_changes(&mut self, changes: Vec<Vec<u8>>) -> ComposerUpdate { ... }

    // --- Content access ---
    pub fn get_content_as_html(&self) -> Utf16String { ... }
    pub fn get_content_as_message_html(&self) -> Utf16String { ... }
    pub fn set_content_from_html(&mut self, html: &Utf16String) -> ComposerUpdate { ... }

    // --- Internal helpers ---
    fn compute_menu_state(&self) -> MenuState { ... }
    fn spans_to_html(&self) -> Utf16String { ... }
    fn html_to_spans(html: &str) -> Vec<Span> { ... }
}
```

**Implementation approach for each operation:**

| Operation | Implementation |
|---|---|
| `replace_text(t)` | `automerge::splice(doc, ["text"], start, del_count, t)` where `start`/`del_count` come from current selection. Apply pending marks if any. |
| `backspace()` | `automerge::splice(doc, ["text"], pos-1, 1, "")` — handle block boundary joining via `joinBlock()` |
| `bold()` | Check `marksAt(cursor)` → if bold active: `unmark("bold", range)`, else `mark("bold", range, true)` |
| `enter()` | `splitBlock(doc, ["text"], cursor, {type: "paragraph", ...})` — inside a list, create new list item |
| `ordered_list()` | Convert current block's marker to `{type: "ordered_list_item"}` or insert new block marker |
| `undo()` | Pop `(heads, selection)` from undo stack → `fork(doc, heads)`, diff to get patches, merge back with conflict resolution |
| `get_content_as_html()` | Call `spans(doc, ["text"])` → walk spans → emit HTML tags |
| `set_content_from_html(html)` | Parse HTML → convert to Span array → `updateSpans(doc, ["text"], spans)` |

#### 1c. Spans ↔ HTML Conversion

Build a bidirectional converter between Automerge's `Span[]` representation and HTML:

**Spans → HTML** (`spans_to_html`):
```
Input:  [{ type: "text", value: "bold", marks: { bold: true } },
         { type: "block", value: { type: "paragraph" } },
         { type: "text", value: "normal" }]

Output: "<strong>bold</strong><p>normal</p>"
```

Rules:
- Text spans with no marks → plain text
- Text spans with marks → wrap in corresponding HTML tags (`bold` → `<strong>`, `italic` → `<em>`, etc.)
- Adjacent text spans with identical marks → merge into single tag
- Block markers → close previous block tag, open new block tag based on `type` and `parents`
- Mention marks → render as `<a href="matrix:..." data-mention-type="...">` pills

**HTML → Spans** (`html_to_spans`):
- Parse HTML into a DOM tree (reuse existing `html5ever` / browser `DOMParser` infrastructure)
- Walk the tree depth-first, collecting text content and active formatting context
- Emit `{type: "text", value, marks}` for text nodes
- Emit `{type: "block", value: {type, parents, attrs}}` at block element boundaries

This converter can largely reuse the existing `to_html.rs` and `parser/` code from the current crate.

#### 1d. Undo/Redo via Heads Tracking

Since Automerge doesn't have built-in undo, implement it by tracking document `Heads`:

```rust
fn push_undo(&mut self) {
    let heads = self.doc.get_heads();
    let selection = self.selection.clone();
    self.undo_stack.push((heads, selection));
    self.redo_stack.clear();
}

fn undo(&mut self) -> ComposerUpdate {
    if let Some((heads, selection)) = self.undo_stack.pop() {
        let current_heads = self.doc.get_heads();
        self.redo_stack.push((current_heads, self.selection.clone()));
        // Fork at the old heads and merge — the "undo" creates a new change
        // that returns the document to the old state
        // Alternative: use changeAt() to apply an inverse operation
        self.selection = selection;
        self.create_update()
    }
}
```

**Important caveat:** In a collaborative setting, undo should only undo the local user's changes, not remote changes. This requires tracking which changes belong to the local actor and computing selective inverses. This is a known hard problem in CRDT systems and may need to be deferred to a later phase.

### Phase 2: Platform Layer Updates

Once `AutomergeComposerModel` is API-compatible, the platform layers need minimal changes. However, we should also introduce **incremental updates** to replace the costly `ReplaceAll` pattern.

#### 2a. Extend `ComposerUpdate` with Patch-Based Updates

Add a new `TextUpdate` variant:

```rust
pub enum TextUpdate<S: UnicodeString> {
    Keep,
    ReplaceAll { replacement_html: S, start: u32, end: u32 },
    Select { start: u32, end: u32 },
    // NEW: incremental patches for efficient rendering
    Patches {
        patches: Vec<RichTextPatch>,
        start: u32,
        end: u32,
    },
}

pub enum RichTextPatch {
    InsertText { index: u32, text: String, marks: HashMap<String, MarkValue> },
    DeleteText { index: u32, length: u32 },
    AddMark { start: u32, end: u32, name: String, value: MarkValue },
    RemoveMark { start: u32, end: u32, name: String },
    InsertBlock { index: u32, block_type: String, parents: Vec<String> },
    RemoveBlock { index: u32 },
    UpdateBlock { index: u32, block_type: String, parents: Vec<String> },
}
```

#### 2b. Platform Patch Consumers

Each platform would add a new code path to apply patches incrementally:

- **Web:** Translate `RichTextPatch` → DOM mutations on the `contentEditable` div (insert text nodes, wrap in `<strong>`, etc.) instead of replacing `innerHTML`
- **iOS:** Translate `RichTextPatch` → `NSMutableAttributedString` mutations (insert/delete/addAttribute) instead of full `NSAttributedString` replacement
- **Android:** Translate `RichTextPatch` → `Editable` mutations (insert/delete/setSpan/removeSpan) instead of full `Spannable` replacement

This is optional for Phase 2 — the existing `ReplaceAll` path can remain as a fallback. Patch-based rendering becomes important for performance with larger documents or high-frequency collaboration updates.

#### 2c. Expose Collaboration APIs in Bindings

Add new methods to the FFI and WASM bindings:

```
// FFI (Swift/Kotlin)
func generateSyncMessage(syncState: SyncState) -> Data?
func receiveSyncMessage(syncState: SyncState, message: Data) -> ComposerUpdate
func save() -> Data
func load(data: Data)
func getLastChange() -> Data?
func applyChanges(changes: [Data]) -> ComposerUpdate

// WASM (JavaScript)
generate_sync_message(syncState: SyncState): Uint8Array | null
receive_sync_message(syncState: SyncState, message: Uint8Array): ComposerUpdate
save(): Uint8Array
load(data: Uint8Array): void
get_last_change(): Uint8Array | null
apply_changes(changes: Uint8Array[]): ComposerUpdate
```

### Phase 3: Collaboration Integration with Matrix

**Goal:** Wire the Automerge sync protocol into the Matrix event system for real-time collaborative editing.

#### 3a. Transport Layer Options

**Option A: Automerge changes as Matrix room events**
- Each `getLastLocalChange()` binary is sent as a custom Matrix event (e.g. `m.room.crdt_change`)
- Incoming events are fed to `applyChanges()`
- Pro: Uses existing Matrix event infrastructure
- Con: Event ordering may not match causal ordering; may need buffering

**Option B: Automerge sync messages via Matrix to-device or room messages**
- Use the full stateful sync protocol — `generateSyncMessage()` / `receiveSyncMessage()`
- One `SyncState` per peer (or per room member)
- Pro: Handles intermittent connectivity, catch-up, and state reconciliation natively
- Con: Requires per-peer state management

**Option C: Hybrid — changes for real-time, sync for catch-up**
- Broadcast `getLastLocalChange()` as events during active editing
- Use the sync protocol when a user joins or reconnects to catch up
- Pro: Best of both approaches
- Con: More complexity

**Recommendation:** Start with Option A (simplest), evolve to Option C.

#### 3b. Document Lifecycle

```
User opens composer:
  1. Load last known document state from local storage (Automerge.load(savedBytes))
  2. Or create fresh: Automerge.from({ text: "" })
  3. For collaborative editing: initSyncState() per peer

User edits:
  1. Automerge.change(doc, ...) — local edit
  2. getLastLocalChange() → send as Matrix event
  3. patchCallback → update UI

Remote change arrives (Matrix event):
  1. applyChanges(doc, [change]) with patchCallback
  2. Patches → incremental UI update

User sends message:
  1. spans(doc, ["text"]) → render to message HTML
  2. Send as normal m.room.message event
  3. Optionally: save(doc) → persist binary for edit history
```

### Phase 4: Advanced Features

#### 4a. Presence & Cursors
- Use Automerge `Cursor` objects for stable cursor positions
- Broadcast cursor positions via Matrix presence or ephemeral events
- Render remote cursors as colored markers in the editor

#### 4b. Operational Transform–Free Conflict UI
- When Automerge detects mark conflicts (same name, different values, overlapping ranges), surface these in the UI
- Allow users to resolve formatting conflicts manually

#### 4c. Document History & Time Travel
- Automerge preserves full edit history
- Expose a timeline/playback UI showing how the document evolved
- Use `fork(doc, historicalHeads)` to view past states

#### 4d. Offline Support
- Automerge documents persist as binary (`save()`) and load instantly
- Edits made offline are captured as changes and merge automatically when connectivity returns
- This is a natural capability of the CRDT — no additional conflict resolution needed

---

## 4. Block Projection / Reconciliation Architecture and Automerge

### 4.1 How the Current Block Projection + Diff System Works

The iOS platform (with plans to bring to Android and Web) has recently implemented a **block projection / inline diff** architecture that eliminates the old HTML → DTCoreText → NSAttributedString pipeline. This is documented in detail in [newdiffplan.md](newdiffplan.md). The key design:

**Outbound (model → editor rendering):**
```
Rust ComposerModel
  → get_block_projections()          ← flat list of BlockProjection structs
  → [BlockProjection]                   (each with InlineRuns carrying AttributeSet)
  → ProjectionRenderer.render()      ← Swift: direct to NSAttributedString
  → UITextView.attributedText        ← 1:1 UTF-16 offset mapping, no HTML intermediate
```

**Inbound (editor → model reconciliation):**
```
User types in UITextView
  → UIKit applies the text mutation itself (replaceText returns true)
  → didUpdateText() fires
  → reconcileNative():
      1. Compare committedAttributedText.string (what the model thinks) 
         vs textView.attributedText.string (what UIKit now has)
      2. computePrefixSuffixDiff(old, new) → InlineDiff { replaceStart, replaceEnd, replacement }
      3. model.replaceTextIn(start, end, replacement) → Rust updates its DOM
      4. Store new committedAttributedText
```

**Why this diff-based reconciliation exists (and why `shouldChangeTextIn` is insufficient):**
- iOS `shouldChangeTextIn:range:replacementText:` is unreliable for autocorrect, predictive text, dictation, CJK IME composition, double-space-to-dot, and other system-level text mutations
- The system text view silently mutates its content in ways the delegate never sees
- The only reliable approach is: let UIKit own the text mutation, then diff what UIKit has against what the model expects, and reconcile the model

### 4.2 Does This Architecture Still Apply With Automerge?

**Yes — the reconciliation problem doesn't change.** The fundamental issue is platform text view behavior, not the document model backend. With Automerge:

- UIKit still silently mutates text (autocorrect, predictive, IME)
- `shouldChangeTextIn` is still unreliable
- You still need to diff platform state against model state to reconcile

**What changes is what happens after the diff is computed:**

| Step | Current (DOM-based model) | With Automerge |
|------|--------------------------|----------------|
| 1. Diff | `computePrefixSuffixDiff(old, new)` → `InlineDiff` | **Same** — identical diff |
| 2. Apply to model | `model.replaceTextIn(start, end, replacement)` | `Automerge.splice(doc, ["text"], start, delCount, replacement)` inside `change()` |
| 3. Re-render | `get_block_projections()` → `ProjectionRenderer.render()` | `Automerge.spans(doc, ["text"])` → render to attributed string |
| 4. Store snapshot | `committedAttributedText = rendered` | **Same** |

The diff logic, IME composition guard, and block-scoped reconciliation all transfer directly.

### 4.3 Impact on Each Layer

#### Block Projection (Outbound Rendering)

The current `get_block_projections()` walks the Rust `Dom<S>` tree, flattening nested formatting containers into `BlockProjection` structs with `InlineRun`s and `AttributeSet`.

With Automerge, `spans(doc, ["text"])` already returns a flat representation:
```
[{ type: "text", value: "hello", marks: { bold: true } },
 { type: "block", value: { type: "paragraph", ... } },
 { type: "text", value: "world" }]
```

This is conceptually identical to `BlockProjection` — both are flat sequences of typed runs with formatting metadata. The **`ProjectionRenderer`** on the Swift side can be adapted to consume either format with minimal changes:

| BlockProjection field | Automerge Span equivalent |
|---|---|
| `BlockKind::Paragraph` | `{ type: "block", value: { type: "paragraph" } }` |
| `InlineRunKind::Text { text, attributes }` | `{ type: "text", value, marks }` |
| `AttributeSet { bold, italic, ... }` | `marks: { bold: true, italic: true, ... }` |
| `InlineRunKind::Mention { url, display_text }` | `{ type: "text", marks: { mention: "matrix:..." } }` |
| `BlockKind::ListItem { list_type, depth }` | `{ type: "block", value: { type: "ordered-list-item", parents: [...] } }` |

**Recommendation:** Build a thin `spans_to_block_projections()` adapter in Rust that converts Automerge's `Span[]` into the existing `BlockProjection` / `FfiBlockProjection` types. This lets `ProjectionRenderer.swift` (and future Android/Web equivalents) work unchanged. Over time, platforms could consume spans directly.

#### Reconciliation (Inbound Diff)

The `reconcileNative()` → `computePrefixSuffixDiff()` → `model.replaceTextIn()` pipeline translates directly:

```swift
// Current
let diff = computePrefixSuffixDiff(old: committedText, new: currentText)
let update = model.replaceTextIn(
    newText: diff.replacement,
    start: UInt32(diff.replaceStart),
    end: UInt32(diff.replaceEnd)
)

// With Automerge — same diff, different model call
let diff = computePrefixSuffixDiff(old: committedText, new: currentText)
model.spliceText(
    start: UInt32(diff.replaceStart),
    deleteCount: UInt32(diff.replaceEnd - diff.replaceStart),
    insertText: diff.replacement
)
// Internally: Automerge.splice(doc, ["text"], start, deleteCount, insertText) 
```

The diff computation, IME guard, and committed-snapshot tracking remain identical.

#### Block-Scoped Optimization

The current plan includes an optimization where inline edits within a single block only diff that block's substring (not the full document). This optimization still applies with Automerge — `block_at_offset()` would resolve against the Automerge spans structure instead of the DOM tree, and the block-scoped diff narrows the comparison range.

### 4.4 The CRDT Merge Quality Trade-Off

There is one important subtlety. Automerge's documentation explicitly warns:

> `updateText()` calculates diffs which **merge less well** than direct `splice()` calls

The reconciliation-based approach computes a diff and sends a bulk replacement to the model. This is semantically equivalent to Automerge's `updateText()` — it loses the user's actual intent (which specific characters were typed, in what order) and reconstructs a plausible edit from the before/after states.

For **single-user local editing**, this is fine — there's no concurrent edit to merge with.

For **collaborative editing**, the quality of merge depends on the granularity of operations:
- **Best merge:** Individual `splice()` per keystroke (captures exact user intent)
- **Acceptable merge:** Per-reconciliation `splice()` (captures the batch of changes since last sync)
- **Worst merge:** Full-document `updateText()`/`updateSpans()` (loses all positional intent)

The block-scoped prefix/suffix diff falls in the "acceptable" middle ground — it produces a single splice per reconciliation cycle, scoped to the changed region. For chat-message-sized content with one local user typing, this should merge well. For longer documents with multiple concurrent editors, more granular operation capture would be beneficial.

**Mitigation strategies for improved collaborative merge quality:**

1. **Reconcile frequently** — the smaller each diff, the closer it approximates individual keystrokes. The current `didUpdateText()` fires on every keystroke, so diffs are typically single-character insertions.

2. **Capture intent when possible** — for formatting operations (`bold()`, `enter()`, list conversion), the platform calls explicit model methods. These translate to precise Automerge operations (`mark()`, `splitBlock()`) with full intent. Only plain text input goes through the diff path.

3. **Use `splice()` not `updateText()`** — even though the diff is computed after the fact, sending it as `splice(start, deleteCount, replacement)` is better than `updateText(newFullText)` because it's positionally specific.

4. **Future: platform-level operation capture** — on platforms where `beforeinput` events or `InputConnection` methods provide reliable edit operations (Web and Android respectively), those could be used to send precise splices, bypassing the diff entirely. iOS would remain diff-based due to `shouldChangeTextIn` unreliability.

### 4.5 Updated Architecture Diagram

```
                      ┌──────────────────────────────────────┐
                      │         Platform Text View            │
                      │   (UITextView / EditText / contentE.) │
                      └──────────┬──────────┬────────────────┘
                                 │          │
                    User types   │          │  Model changed
                    (inbound)    │          │  (outbound)
                                 ▼          │
                      ┌──────────────────┐  │
                      │  reconcileNative  │  │
                      │  prefix/suffix    │  │
                      │  diff             │  │
                      └────────┬─────────┘  │
                               │            │
                     InlineDiff│            │
                               ▼            │
                      ┌──────────────────────────────────────┐
                      │         Adapter Layer (Rust)          │
                      │  AutomergeComposerModel               │
                      │                                       │
                      │  inbound:  splice() / mark()          │
                      │  outbound: spans() → BlockProjection  │
                      │  sync:     generateSyncMessage() /    │
                      │            receiveSyncMessage()       │
                      └────────┬──────────┬──────────────────┘
                               │          │
                               │          │ Automerge patches
                               ▼          ▼
                      ┌──────────────────────────────────────┐
                      │      Automerge Document (CRDT)        │
                      │  Text sequence + Marks + Blocks       │
                      └──────────────────────────────────────┘
                               │          ▲
                               │          │
                    save() /   │          │  applyChanges() /
                    getLastChange()       │  receiveSyncMessage()
                               ▼          │
                      ┌──────────────────────────────────────┐
                      │      Matrix Event Transport           │
                      │  (m.room.crdt_change events)          │
                      └──────────────────────────────────────┘
```

### 4.6 Summary: What Transfers, What Changes, What's New

| Component | Transfers as-is | Needs adaptation | New with Automerge |
|---|---|---|---|
| `computePrefixSuffixDiff()` | ✅ | | |
| `reconcileNative()` flow | ✅ | Model call changes from `replaceTextIn` → `splice` | |
| IME composition guard | ✅ | | |
| `committedAttributedText` snapshot | ✅ | | |
| `ProjectionRenderer.swift` | | Consume Automerge spans instead of `BlockProjection` (or adapt via thin converter) | |
| `BlockProjection` types | | Optional — can be generated from spans, or replaced by spans directly | |
| `block_at_offset()` | | Walks spans instead of DOM tree | |
| `patchProjections()` offset patching | | May be replaced by Automerge patch callbacks | |
| Automerge sync protocol | | | ✅ Entirely new |
| `patchCallback` for remote changes | | | ✅ Incremental UI update from remote edits |
| `Cursor` (stable positions) | | | ✅ Replaces fragile `DomHandle` paths |

---

## 5. Detailed Implementation Tasks

### Phase 0 Tasks (PoC — ~2-3 weeks)
- [ ] Set up a standalone web project using `@automerge/automerge`
- [ ] Implement text editing (splice) with a `contentEditable` div
- [ ] Implement all five inline formats as marks (bold/italic/underline/strikethrough/inline-code)
- [ ] Implement links as marks with `expand: "none"`
- [ ] Implement paragraphs, headings, and line breaks via `splitBlock`/`joinBlock`
- [ ] Implement ordered/unordered lists with nesting via `parents` array
- [ ] Implement code blocks and block quotes as block markers
- [ ] Implement mentions (explore: mark vs. inline object)
- [ ] Implement basic undo/redo via heads tracking
- [ ] Implement two-peer sync over a simple WebSocket transport
- [ ] Build spans→HTML renderer and HTML→spans parser
- [ ] Benchmark against current editor for typical chat messages

### Phase 1 Tasks (Core Migration — ~4-6 weeks)
- [ ] Add `automerge` as a dependency to the workspace (Rust crate)
- [ ] Create `AutomergeComposerModel` struct with `automerge::AutoCommit` backend
- [ ] Implement all text manipulation methods (`replace_text`, `backspace`, `delete`, `enter`)
- [ ] Implement all inline formatting methods (`bold`, `italic`, etc.) using `mark`/`unmark`
- [ ] Implement all block formatting methods using `splitBlock`/`joinBlock`/`updateBlock`
- [ ] Implement lists with indentation using `parents` array manipulation
- [ ] Implement links using marks
- [ ] Implement mentions (likely as marks with custom expand behavior)
- [ ] Implement spans→HTML conversion for `ComposerUpdate::TextUpdate::ReplaceAll`
- [ ] Implement HTML→spans conversion for `set_content_from_html`
- [ ] Implement undo/redo using Automerge heads tracking
- [ ] Implement `select()`, `get_content_as_*()`, `set_content_from_*()` methods
- [ ] Implement `MenuState` computation from `marksAt()` and current block context
- [ ] Implement suggestion pattern detection for `MenuAction`
- [ ] Implement `LinkAction` computation from current marks
- [ ] Port existing tests to validate behavioral equivalence
- [ ] Feature-flag the new backend (`automerge` feature) alongside the old one

### Phase 2 Tasks (Platform Updates — ~3-4 weeks)
- [ ] Update FFI bindings to expose collaboration methods
- [ ] Update WASM bindings to expose collaboration methods
- [ ] (Optional) Add `TextUpdate::Patches` variant and patch-based rendering
- [ ] iOS: Add patch-based `NSAttributedString` update path
- [ ] Android: Add patch-based `Spannable` update path
- [ ] Web: Add patch-based DOM mutation path
- [ ] Integration testing on all three platforms
- [ ] Performance testing: compare ReplaceAll vs. patch-based updates

### Phase 3 Tasks (Matrix Integration — ~4-6 weeks)
- [ ] Design Matrix event schema for Automerge changes
- [ ] Implement change broadcasting via Matrix events
- [ ] Implement change reception and `applyChanges()` integration
- [ ] Implement sync protocol for catch-up/reconnection scenarios
- [ ] Handle multi-room, multi-document state management
- [ ] Implement document save/load for persistence
- [ ] End-to-end testing with multiple Matrix clients

---

## 6. Open Design Questions

### 6.1 Mentions: Mark vs. Inline Object

**Option A: Mark-based mentions**
```typescript
mark(d, ["text"], {start: 5, end: 10, expand: "none"}, "mention", "matrix:u/@user:server.com")
```
- Pro: Simple, works with existing mark infrastructure
- Con: The mention text (display name) is mutable — users could type inside it and corrupt the mention

**Option B: Replicate current `MentionNode` as an atomic object**
- Use a single character (e.g. `\uFFFC`) with a mark or block marker carrying the mention data
- Pro: Atomic, can't be partially edited
- Con: More complex, doesn't map cleanly to Automerge's text model

**Recommendation:** Start with Option A with UI-level guards preventing editing inside mentions (similar to how the current `MentionNode` is handled). The `expand: "none"` semantics prevent the mark from bleeding into adjacent typed text.

### 6.2 Undo/Redo Semantics

In a single-user context, undo is straightforward (track heads). In a collaborative context:
- Should undo revert only the local user's last change? (selective undo)
- Or revert the document to its prior state regardless of who made the last change? (global undo)

**Recommendation:** Implement global undo first (simpler). Selective undo requires tracking operation ownership and computing inverses, which is a significant undertaking.

### 6.3 Code Block Formatting Exclusion

Inside a code block, inline formatting (bold, italic, etc.) should be disabled. Automerge doesn't natively enforce this constraint.

**Recommendation:** Enforce at the adapter layer — when the cursor is inside a code block marker range, `bold()` et al. return `ComposerUpdate` with `MenuState::Disabled` and don't apply marks.

### 6.4 Matrix Message HTML Compatibility

The current editor produces specific HTML for Matrix messages (e.g. `<mx-reply>`, `data-mx-*` attributes, Matrix URI schemes). The Automerge adapter must produce identical HTML output.

**Recommendation:** The spans→HTML renderer must be built with Matrix HTML spec compliance as a primary requirement. Use the existing `to_message_html()` logic as the reference implementation.

### 6.5 Migration Path for Existing Messages

Existing messages were composed without CRDT metadata. When editing an existing message:
1. Load its HTML content
2. Parse HTML → spans → `updateSpans()` to create an Automerge document
3. Edit within the CRDT model
4. Export as HTML for the edited message event

This means `set_content_from_html()` → `updateSpans()` must be robust for all existing Matrix HTML patterns.

---

## 7. Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Automerge performance for large documents | Low (chat messages are small) | Medium | Benchmark in Phase 0; chat messages rarely exceed a few KB |
| Mark conflict resolution surprises | Medium | Medium | Document the "arbitrary but deterministic" resolution; consider UI indicators |
| Undo/redo complexity in collaborative mode | High | High | Start with global undo; defer selective undo |
| Mention atomicity | Medium | Medium | UI-level guards + `expand: "none"` marks |
| HTML round-trip fidelity | Medium | High | Extensive test suite comparing old vs. new HTML output |
| Binary size increase (Automerge + existing code) | Medium | Low | Feature-flag; eventually remove old code |
| Platform patch rendering complexity | Medium | Medium | Keep `ReplaceAll` fallback; incremental patches are optional |

---

## 8. Success Criteria

1. **Feature parity:** All existing editing operations produce identical HTML output
2. **Test parity:** All existing Rust tests pass against the new backend
3. **Performance:** No perceptible latency regression for typical chat messages (< 10KB)
4. **Collaboration:** Two users can edit the same message simultaneously with automatic merge
5. **Platform compatibility:** iOS, Android, and Web all work without platform layer rewrites
6. **Sync robustness:** Documents converge after network partitions and reconnections

---

## 9. Recommended Starting Point

Begin with **Phase 0** (web-only proof of concept) to validate the approach with minimal investment. The key deliverable is a working demo that proves:

1. Automerge's rich text model can represent all Matrix RTE content types
2. The editing UX (formatting toggles, list manipulation, mention insertion) feels equivalent
3. Two-peer sync works reliably
4. Performance is acceptable

If the PoC succeeds, proceed to Phase 1 with confidence that the architectural approach is sound. The feature-flag strategy (`#[cfg(feature = "automerge")]`) allows the new backend to coexist with the old one during migration, enabling gradual rollout and A/B testing.
