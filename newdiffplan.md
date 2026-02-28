# Rich Text Editor Diff Refactor Plan
## Block-Aware Inline Diff Architecture (Rust Core + iOS)

---

# Scope

This plan covers the **iOS rendering and edit-reconciliation path only**. Android and Web continue
using the existing HTML pipeline unchanged.

**Constraints agreed:**

- Build `BlockProjection` as a read-only view over the existing nested-container DOM — **no rewrite
  of `DomNode` or formatting model**.
- `BlockProjection` is exposed as a **new, separate FFI API**. Existing `TextUpdate::ReplaceAll`
  and all current FFI methods are kept intact.
- HTML serialisation is preserved for message sending, paste, web, and
  `setContentFromHtml`/`getContentAsHtml`. Only the iOS rendering path
  (`HTML → DTCoreText → NSAttributedString`) is replaced.

---

# Problem Statement

## Current iOS Pipeline

```
Rust Dom::to_html()          (composer_model/base.rs:175)
  → Vec<u16> via ReplaceAll  (ffi_text_update.rs)
  → HTMLParser.parse()        (HTMLParser.swift)
  → DTCoreText builder        (DTHTMLAttributedStringBuilder)
  → Post-processing           (NSMutableAttributedString.swift)
  → NSAttributedString
  → UITextView.attributedText
  → textViewDidChange
  → reconciliateIfNeeded()    (WysiwygComposerViewModel.swift:683)
  → StringDiffer global diff  (StringDiffer.swift)
  → model.replaceTextIn()
```

Problems observed in the existing code:

- Every keystroke triggers a full HTML round-trip and full document re-render.
- `StringDiffer.replacement(from:to:)` runs a Myers diff over the entire document string
  (`htmlChars`), not just the affected block. A corruption anywhere in the document affects all
  blocks.
- Index mapping between attributed positions and HTML/UTF-16 positions is complex and fragile
  (`NSAttributedString+Range.swift` — discardable text ranges, mention replacement offsets, ZWSP
  placeholders).
- Accumulated workarounds for iOS auto-corrections: `updateForDoubleSpaceToDotConversionIfNeeded`,
  `updateDotAfterInlineTextPredicationIfNeeded`, `isReplacingWordWithSuggestion`,
  `isExitingPredictiveText`, NBSP normalisation.
- No guard against mutating Rust during IME composition — reconciliation happens post-hoc.

## Target iOS Pipeline

```
Rust get_block_projections()   (new API on ComposerModel FFI)
  → [BlockProjection]
  → Swift AttributedString builder (ProjectionRenderer.swift — new)
  → NSAttributedString
  → UITextView.attributedText

User edit
  → shouldChangeTextIn → block_at_offset() → EditHint
  → block-scoped prefix/suffix diff
  → apply_inline_edit()        (new Rust API)
  → get_block_projections() for affected block only
  → patch NSAttributedString in place
```

---

# Current Codebase Reference Map

| Concept | File(s) |
|---|---|
| DOM node types (`DomNode`, `ContainerNode`, `TextNode`, `MentionNode`) | `crates/wysiwyg/src/dom/nodes/` |
| Block classification `is_block_kind()` | `crates/wysiwyg/src/dom/nodes/dom_node.rs` |
| Block helper methods | `crates/wysiwyg/src/dom/dom_block_nodes.rs` |
| Offset resolution `find_range()` | `crates/wysiwyg/src/dom/find_range.rs` |
| Path-based handles `DomHandle` | `crates/wysiwyg/src/dom/dom_handle.rs` |
| Core mutation `replace_text_in()` | `crates/wysiwyg/src/dom/dom_methods.rs` |
| Formatting (nested containers) | `crates/wysiwyg/src/composer_model/format.rs` |
| New lines / block split | `crates/wysiwyg/src/composer_model/new_lines.rs` |
| `ComposerUpdate` / `ReplaceAll` construction | `crates/wysiwyg/src/composer_model/base.rs` |
| FFI model wrapper | `bindings/wysiwyg-ffi/src/ffi_composer_model.rs` |
| FFI `TextUpdate` | `bindings/wysiwyg-ffi/src/ffi_text_update.rs` |
| UDL namespace | `bindings/wysiwyg-ffi/src/wysiwyg_composer.udl` |
| iOS ViewModel | `platforms/ios/lib/WysiwygComposer/Sources/WysiwygComposer/Components/WysiwygComposerView/WysiwygComposerViewModel.swift` |
| iOS text view | `platforms/ios/lib/WysiwygComposer/Sources/WysiwygComposer/Components/WysiwygComposerView/WysiwygTextView.swift` |
| HTML parser (DTCoreText) | `platforms/ios/lib/WysiwygComposer/Sources/HTMLParser/HTMLParser.swift` |
| Range mapping | `platforms/ios/lib/WysiwygComposer/Sources/HTMLParser/Extensions/NSAttributedString+Range.swift` |
| Global differ | `platforms/ios/lib/WysiwygComposer/Sources/WysiwygComposer/Tools/StringDiffer.swift` |
| SwiftUI bridge / Coordinator | `platforms/ios/lib/WysiwygComposer/Sources/WysiwygComposer/Components/WysiwygComposerView/WysiwygComposerView.swift` |

---

# Phase 1 — BlockProjection in Rust (read-only, no DOM change)

**Goal:** Expose a flat view of the document's block structure and inline formatting without
modifying `DomNode`, `ContainerNode`, or any formatting logic.

## 1.1 New Rust types — `crates/wysiwyg/src/block_projection.rs`

```rust
// Block identifier — stable across edits within a session.
// Wraps DomHandle path (e.g. [0], [1], [2]) for top-level block nodes.
// NOTE: DomHandle paths shift on structural edits; see BlockId stability note below.
pub type BlockId = DomHandle;

#[derive(Clone, Debug, PartialEq)]
pub struct AttributeSet {
    pub bold: bool,
    pub italic: bool,
    pub strike_through: bool,
    pub underline: bool,
    pub inline_code: bool,
    pub link_url: Option<String>,
}

#[derive(Clone, Debug)]
pub enum InlineRunKind {
    Text {
        text: String,
        attributes: AttributeSet,
    },
    Mention {
        url: String,
        display_text: String,
        // display_length is always 1 UTF-16 code unit in the Rust model
    },
    LineBreak,
}

#[derive(Clone, Debug)]
pub struct InlineRun {
    pub node_handle: DomHandle,    // handle into the underlying DOM node
    pub start_utf16: usize,        // absolute UTF-16 offset (document-level)
    pub end_utf16: usize,
    pub kind: InlineRunKind,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BlockKind {
    Paragraph,
    Quote,
    CodeBlock,
    ListItem { list_type: ListType, depth: usize },
    Generic,   // root-level content not yet wrapped in a block
}

#[derive(Clone, Debug)]
pub struct BlockProjection {
    pub block_id: BlockId,          // DomHandle of the block container
    pub kind: BlockKind,
    pub start_utf16: usize,         // absolute offset of first content code unit
    pub end_utf16: usize,           // exclusive, does NOT include the block separator
    pub inline_runs: Vec<InlineRun>,
}
```

## 1.2 Building projections — tree walk

Implement on `Dom<Utf16String>` (or as free functions in the new module):

```rust
impl<S: UnicodeString> Dom<S> {
    /// Returns a projection for every block-level child of the root Generic node.
    /// Formatting nesting is flattened into AttributeSet — the underlying tree is not modified.
    pub fn get_block_projections(&self) -> Vec<BlockProjection> { ... }

    /// Returns the BlockId (DomHandle) of the innermost block node that contains
    /// the given UTF-16 offset.  Reuses find_range() → DomLocation → walk ancestors.
    pub fn block_at_offset(&self, offset_utf16: usize) -> Option<BlockId> { ... }
}
```

**Flattening nested formatting containers:**

The existing DOM uses nested containers:
`<em><strong>text</strong></em>` → `Container(Italic) { children: [Container(Bold) { children: [Text("text")] }] }`

The projection must flatten this recursively, accumulating `AttributeSet` from ancestor containers,
without mutating the tree:

```rust
fn collect_runs(
    node: &DomNode<S>,
    inherited: AttributeSet,
    cursor: &mut usize,  // running UTF-16 offset
    runs: &mut Vec<InlineRun>,
) {
    match node {
        DomNode::Text(t) => {
            let len = t.text_len();
            runs.push(InlineRun {
                node_handle: t.handle().clone(),
                start_utf16: *cursor,
                end_utf16: *cursor + len,
                kind: InlineRunKind::Text {
                    text: t.data().to_string(),
                    attributes: inherited,
                },
            });
            *cursor += len;
        }
        DomNode::Container(c) if c.kind().is_formatting_node() => {
            let mut attrs = inherited.clone();
            apply_format_to_attrs(c.kind(), &mut attrs); // set bold/italic/etc.
            for child in c.children() {
                collect_runs(child, attrs.clone(), cursor, runs);
            }
        }
        DomNode::Container(c) if matches!(c.kind(), ContainerNodeKind::Link(url)) => {
            let mut attrs = inherited.clone();
            attrs.link_url = Some(url.to_string());
            for child in c.children() {
                collect_runs(child, attrs.clone(), cursor, runs);
            }
        }
        DomNode::Mention(m) => {
            runs.push(InlineRun { kind: InlineRunKind::Mention { ... }, ... });
            *cursor += 1; // mentions are always 1 code unit
        }
        DomNode::LineBreak(_) => {
            runs.push(InlineRun { kind: InlineRunKind::LineBreak, ... });
            *cursor += 1;
        }
        _ => {}
    }
}
```

## 1.3 `block_at_offset` implementation

`Dom::find_range(offset, offset)` (in `find_range.rs`) returns `Vec<DomLocation>`. Walk up the
handle path using `DomHandle::parent_handle()` until a node satisfying `is_block_kind()` is found:

```rust
pub fn block_at_offset(&self, offset_utf16: usize) -> Option<BlockId> {
    let range = self.find_range(offset_utf16, offset_utf16);
    let loc = range.locations.first()?;
    let mut handle = loc.node_handle.clone();
    loop {
        let node = self.lookup_node(&handle);
        if node.is_block_node() {
            return Some(handle);
        }
        if handle.is_root() { return None; }
        handle = handle.parent_handle();
    }
}
```

## 1.4 `apply_inline_edit` — new Rust mutation entry point

Adds a block-scoped mutation API. Delegates to the existing `Dom::replace_text_in()` in
`dom_methods.rs` (which already handles node splitting, merging, invariant normalisation):

```rust
// On ComposerModel<Utf16String>:
pub fn apply_inline_edit(
    &mut self,
    block_id: &BlockId,
    replace_start_utf16: usize,
    replace_end_utf16: usize,
    replacement: &S,
) -> ComposerUpdate<S> {
    self.push_state_to_history();
    // The offsets are document-level UTF-16, same as replace_text_in expects.
    self.state.dom.replace_text_in(
        replacement,
        replace_start_utf16,
        replace_end_utf16,
    );
    self.state.start = Location::from(replace_start_utf16 + replacement.len());
    self.state.end = self.state.start;
    self.create_update_replace_all()
    // NOTE: ReplaceAll is returned so existing iOS code (applyUpdate) continues to work.
    // Once the projection renderer is stable, this can be changed to a new UpdateBlock variant.
}
```

**BlockId stability note:** `DomHandle` is a path (`Vec<usize>`) that can shift when siblings are
inserted or removed. For the block-scoped diff path, the `block_id` captured in `shouldChangeTextIn`
remains valid until the next structural edit because inline edits do not change block ordering.
After any structural edit (`enter()`, paste of multi-line text), Swift must call
`get_block_projections()` again to refresh all block IDs.

---

# Phase 2 — Expose via UniFFI

Add new FFI record types and methods to `bindings/wysiwyg-ffi/src/`.

All new types use `#[derive(uniffi::Record)]` — no UDL changes needed (the UDL file
`wysiwyg_composer.udl` only declares the namespace; all types are proc-macro based).

## 2.1 New FFI types — `ffi_block_projection.rs`

```rust
#[derive(uniffi::Record, Clone)]
pub struct FfiAttributeSet {
    pub bold: bool,
    pub italic: bool,
    pub strike_through: bool,
    pub underline: bool,
    pub inline_code: bool,
    pub link_url: Option<String>,
}

#[derive(uniffi::Enum, Clone)]
pub enum FfiInlineRunKind {
    Text { text: String, attributes: FfiAttributeSet },
    Mention { url: String, display_text: String },
    LineBreak,
}

#[derive(uniffi::Record, Clone)]
pub struct FfiInlineRun {
    pub node_id: String,      // DomHandle serialised as "0,2,1" for debuggability
    pub start_utf16: u32,
    pub end_utf16: u32,
    pub kind: FfiInlineRunKind,
}

#[derive(uniffi::Enum, Clone)]
pub enum FfiBlockKind {
    Paragraph,
    Quote,
    CodeBlock,
    ListItemOrdered { depth: u32 },
    ListItemUnordered { depth: u32 },
    Generic,
}

#[derive(uniffi::Record, Clone)]
pub struct FfiBlockProjection {
    pub block_id: String,     // DomHandle serialised as "0", "1", "2" etc.
    pub kind: FfiBlockKind,
    pub start_utf16: u32,
    pub end_utf16: u32,
    pub inline_runs: Vec<FfiInlineRun>,
}
```

## 2.2 New FFI methods on `ComposerModel`

Add to `ffi_composer_model.rs`:

```rust
#[uniffi::export]
impl ComposerModel {
    /// Returns a flat projection of all blocks and their inline runs.
    /// Offsets are UTF-16 code units, consistent with select() and replace_text_in().
    pub fn get_block_projections(&self) -> Vec<FfiBlockProjection> { ... }

    /// Returns the serialised block_id for the block containing the given UTF-16 offset,
    /// or None if the offset is out of range.
    pub fn block_at_offset(&self, offset_utf16: u32) -> Option<String> { ... }

    /// Applies a text replacement scoped to a single block.
    /// replace_start / replace_end are document-level UTF-16 offsets.
    /// Returns a ComposerUpdate (ReplaceAll for now; UpdateBlock in future).
    pub fn apply_inline_edit(
        &self,
        block_id: String,
        replace_start_utf16: u32,
        replace_end_utf16: u32,
        replacement_text: String,
    ) -> Arc<ComposerUpdate> { ... }
}
```

---

# Phase 3 — Swift: Build AttributedString from Projection

**New file:** `ProjectionRenderer.swift` in the `WysiwygComposer` target.

Replaces the rendering pipeline that currently flows through `HTMLParser.parse()` and
`applyReplaceAll()` in `WysiwygComposerViewModel.swift` (~line 598).

```swift
struct ProjectionRenderer {
    let style: HTMLParserStyle
    let mentionReplacer: HTMLMentionReplacer?

    func render(projections: [FfiBlockProjection]) -> NSAttributedString {
        let result = NSMutableAttributedString()
        for (i, block) in projections.enumerated() {
            let blockAttr = blockAttributes(for: block.kind)
            for run in block.inlineRuns {
                switch run.kind {
                case .text(let text, let attrs):
                    result.append(NSAttributedString(
                        string: text,
                        attributes: inlineAttributes(for: attrs, block: blockAttr)
                    ))
                case .mention(let url, let display):
                    let pill = mentionReplacer?.replacementForMention(url: url, text: display)
                        ?? NSAttributedString(string: display)
                    result.append(pill)
                case .lineBreak:
                    result.append(NSAttributedString(string: "\n"))
                }
            }
            if i < projections.count - 1 {
                result.append(NSAttributedString(string: "\n",
                    attributes: blockAttributes(for: block.kind)))
            }
        }
        return result
    }
}
```

**Block kind → NSAttributedString attributes:**

- `Paragraph` → default paragraph style
- `CodeBlock` → monospace font, code background, no autocorrect
- `Quote` → left border paragraph style (replaces DTCoreText CSS workarounds)
- `ListItem` → paragraph indent style, list prefix marker (replaces `.discardableText` workaround)

**This eliminates:**

- `HTMLParser.swift` and the entire `HTMLParser/` SPM target
- DTCoreText dependency (`DTHTMLAttributedStringBuilder`, custom CSS)
- All post-processing in `NSMutableAttributedString.swift`
  (link colour fix, code/quote background hack, ZWSP insertion, discardable text marking)
- The `NSAttributedString+Range.swift` bidirectional offset mapping
  (discardable text ranges, mention `rustLength`, `htmlChars`)

**Store snapshots on the ViewModel:**

```swift
var committedProjections: [FfiBlockProjection] = []
var committedAttributedString: NSAttributedString = NSAttributedString()
```

---

# Phase 4 — Block Identification in shouldChangeTextIn

Replace the routing logic in `WysiwygComposerViewModel.replaceText(range:replacementText:)`
(currently lines 324–392) with block-aware routing.

```swift
func replaceText(range: NSRange, replacementText: String) -> Bool {
    guard !replacementText.contains("\n") || replacementText.isEmpty else {
        // Structural: delegate to Rust
        handleStructuralEdit(range: range, replacement: replacementText)
        return false
    }

    let startOffset = UInt32(range.location)
    let endOffset = UInt32(range.location + range.length)

    let startBlock = model.blockAtOffset(startOffset)
    let endBlock = range.length > 0
        ? model.blockAtOffset(endOffset - 1)
        : startBlock

    if startBlock != nil && startBlock == endBlock {
        // Inline edit within one block — let UIKit apply, diff in didUpdateText
        self.pendingEditHint = EditHint(
            blockId: startBlock!,
            range: range,
            replacement: replacementText
        )
        return true
    }

    // Cross-block replace — explicit structural operation
    handleStructuralEdit(range: range, replacement: replacementText)
    return false
}
```

---

# Phase 5 — Block-Scoped Inline Diff

In `textViewDidChange` / `didUpdateText()`, replace `reconciliateIfNeeded()` for the inline case:

```swift
func reconcileInlineEdit(hint: EditHint) {
    guard !isComposing else { return }

    // 1. Find the old block range in the committed attributed string
    guard let oldBlock = committedProjections.first(where: { $0.blockId == hint.blockId }) else {
        fallbackFullReconcile(); return
    }
    let oldBlockRange = NSRange(location: Int(oldBlock.startUtf16),
                                length: Int(oldBlock.endUtf16 - oldBlock.startUtf16))

    // 2. Extract old and new block substrings (plain UTF-16 strings for diffing)
    let oldText = committedAttributedString
        .attributedSubstring(from: oldBlockRange).string
    let newText = textView.attributedText
        .attributedSubstring(from: oldBlockRange).string  // UIKit has already applied the edit

    // 3. Prefix / suffix diff (O(n) in block length, never full document)
    let diff = computePrefixSuffixDiff(old: oldText, new: newText)

    // 4. Send to Rust — document-level offsets
    let absStart = UInt32(Int(oldBlock.startUtf16) + diff.replaceStart)
    let absEnd   = UInt32(Int(oldBlock.startUtf16) + diff.replaceEnd)
    let update = model.applyInlineEdit(
        blockId: hint.blockId,
        replaceStartUtf16: absStart,
        replaceEndUtf16: absEnd,
        replacementText: diff.replacement
    )

    // 5. Refresh only the affected block's attributed substring
    applyUpdate(update, affectedBlockId: hint.blockId)
}

/// O(n) prefix/suffix diff — replaces StringDiffer.replacement(from:to:)
func computePrefixSuffixDiff(old: String, new: String) -> InlineDiff {
    let oldUTF16 = Array(old.utf16)
    let newUTF16 = Array(new.utf16)
    var prefixLen = 0
    while prefixLen < oldUTF16.count && prefixLen < newUTF16.count
          && oldUTF16[prefixLen] == newUTF16[prefixLen] { prefixLen += 1 }
    var suffixLen = 0
    while suffixLen < (oldUTF16.count - prefixLen)
          && suffixLen < (newUTF16.count - prefixLen)
          && oldUTF16[oldUTF16.count - 1 - suffixLen] == newUTF16[newUTF16.count - 1 - suffixLen]
    { suffixLen += 1 }
    return InlineDiff(
        replaceStart: prefixLen,
        replaceEnd: oldUTF16.count - suffixLen,
        replacement: String(utf16CodeUnits: Array(newUTF16[prefixLen..<(newUTF16.count - suffixLen)]),
                            encoding: .utf16)!
    )
}
```

**Replaces:** `StringDiffer.swift` global Myers diff and `reconciliateIfNeeded()` in the ViewModel.

---

# Phase 6 — IME Composition Guard

Add to the `Coordinator` in `WysiwygComposerView.swift` (replaces implicit reconciliation via
`hasUncommitedText` / `StringDiffer` for CJK):

```swift
private var isComposing = false

func textViewDidChange(_ textView: UITextView) {
    if textView.markedTextRange != nil {
        isComposing = true
        return  // Do NOT call into Rust model during composition
    }
    if isComposing {
        isComposing = false
        // Composition ended — reconcile the whole block via full re-diff
        viewModel.reconcileBlockAfterComposition()
        return
    }
    viewModel.didUpdateText()
}
```

`reconcileBlockAfterComposition()` uses the block-scoped diff (Phase 5) but must handle the full
replacement the IME may have produced (possibly longer than what `pendingEditHint` captured).

---

# Phase 7 — Structural Operations

Structural edits are already handled by the existing Rust API:

| Operation | Existing Rust method | Notes |
|---|---|---|
| Split block (Enter) | `enter()` → `new_lines.rs` → `split_sub_tree_from()` | Already correct |
| Delete across blocks | `replace_text_in(start, end, "")` in `dom_methods.rs` | Covers selection spanning blocks |
| Paste multiline | `replaceHtml(html)` (WASM) / manual multi-call on FFI | Need `replaceHtml` in FFI (currently WASM only — see limitations) |

After any structural edit, Swift must call `get_block_projections()` to rebuild the full snapshot
(block IDs may have shifted because `DomHandle` paths are position-based).

---

# Phase 8 — Offset Update Strategy

After an inline edit:

- Only the affected block's `inline_runs` and `end_utf16` change.
- All subsequent blocks shift by `delta = new_block_length - old_block_length`.
- **Swift-side:** patch `committedProjections` without calling `get_block_projections()` again:

```swift
func patchProjections(blockId: String, newBlock: FfiBlockProjection) {
    guard let idx = committedProjections.firstIndex(where: { $0.blockId == blockId }) else { return }
    let delta = Int(newBlock.endUtf16) - Int(committedProjections[idx].endUtf16)
    committedProjections[idx] = newBlock
    for i in (idx + 1)..<committedProjections.count {
        committedProjections[i].startUtf16 = UInt32(Int(committedProjections[i].startUtf16) + delta)
        committedProjections[i].endUtf16   = UInt32(Int(committedProjections[i].endUtf16)   + delta)
    }
}
```

Do NOT rebuild the entire projection unless a structural edit occurred.

---

# Phase 9 — Invariants

1. Rust owns all UTF-16 offset arithmetic.
2. Swift never walks the AST or inspects `DomHandle` paths for content.
3. All offsets are UTF-16 code units at every API boundary.
4. Mentions are atomic: `text_len() == 1` in Rust; the projection encodes display text separately.
5. Adjacent inline runs with identical `AttributeSet` are merged (enforced in Rust projection
   builder).
6. Structural edits are explicit: never inferred from a text diff.
7. Inline edits never cross block boundaries; cross-block edits always go through the structural
   path.
8. Rust is never called during IME composition (`markedTextRange != nil`).

---

# Phase 10 — Testing Strategy

## Rust Unit Tests (`crates/wysiwyg/tests/`)

- `get_block_projections()` on a document with paragraphs, list, code block, quote
- Projection offsets are contiguous and correct across blocks
- Nested formatting flattened correctly (`<em><strong>` → both bold and italic in `AttributeSet`)
- `block_at_offset` resolves to correct block for offsets at start, middle, end, boundary
- `apply_inline_edit` inside a bold run
- `apply_inline_edit` at an attribute boundary
- `apply_inline_edit` — delete spanning two formatting containers
- `apply_inline_edit` — attempt to partially delete a mention (must preserve atomicity)
- Unicode: grapheme clusters at block boundaries, emoji, CJK surrogate pairs
- Projection after structural edit (block IDs reset, offsets correct)

## iOS Integration Tests

- Rapid keystrokes — no accumulated offset drift
- Autocorrect replacement (single-word replace)
- Japanese IME composition end-to-end
- Bold toggle mid-word — projection reflects new `AttributeSet`
- Backspace at block start — triggers structural path, not inline diff
- Paste of multiline content — structural path
- Mention atomicity — backspace adjacent to mention does not corrupt it
- `committedProjections` snapshot stays in sync after patch

---

# Phase 11 — Migration Order

Steps are designed to be individually shippable without breaking existing behaviour.

1. ✅ **Add `BlockProjection` types and `get_block_projections()` to Rust** — read-only, no
   behaviour change. Add Rust unit tests.
2. ✅ **Expose via UniFFI** — add `FfiBlockProjection`, `get_block_projections()`,
   `block_at_offset()`, `apply_inline_edit()` to `ffi_composer_model.rs`. Build XCFramework.
3. ✅ **Build `ProjectionRenderer.swift` and `InlineReconciliation.swift`** — compiles as part
   of the target. Not yet wired in.
4. ✅ **Add IME composition guard** in `Coordinator.textViewDidChange`.
5. ✅ **Add new methods to `ComposerModelWrapper`** — `getBlockProjections()`,
   `blockAtOffset()`, `applyInlineEdit()`.
6. ✅ **Add ViewModel state** — `committedProjections`, `pendingEditHint`.
   (Note: `ProjectionRenderer` is constructed fresh per call — no stored instance needed.)
7. ✅ **Switch rendering: replace `applyReplaceAll()` body** — calls
   `model.getBlockProjections()` → `ProjectionRenderer.render()`. `applySelect()` and
   `select(range:)` use direct UTF-16 offsets. `useProjectionRenderer` flag added then removed
   (projection is now the sole path).
8. ✅ **Add iOS unit tests** — `ProjectionRenderer`, `computePrefixSuffixDiff`,
   `patchProjections`, wrapper methods.
9. **Switch reconciliation: replace `reconciliateIfNeeded()` body** — see Phase C in the
   Deletion Plan below. Currently `reconciliateIfNeeded()` still uses `StringDiffer` +
   `.htmlChars`. The new `reconcileInlineEdit()` exists but is only wired into the
   `!alwaysReconcile` path (never taken by default). **This is the current focus.**
10. ✅ **Implement `reconcileBlockAfterComposition()`** on ViewModel — completes the IME guard.
11. **Implement `patchProjections` offset updates** — remove full `get_block_projections()` calls
    on every keystroke. (`patchProjections` is implemented but not yet wired into the active path.)
12. **Add explicit structural operation APIs** — `replace_across_blocks()`, `replaceHtml` to FFI.
13. **Remove all HTML rendering workarounds** from the iOS ViewModel — partially done:
    `updateForDoubleSpaceToDot…` and `updateDotAfterInlineTextPredication…` removed in Phase B.
    Remaining: `isReplacingWordWithSuggestion` (dead code under `alwaysReconcile = true`),
    NBSP normalisation in `StringDiffer`. See Phase C step C.1 and C.3.
14. **Remove `NSAttributedString+Range.swift`** offset mapping complexity. See Phase C step C.3.
15. ✅ **Remove DTCoreText** from `Package.swift` dependencies. (Phase B complete. `HTMLParser/`
    SPM target still exists but only contains shared files — see Phase D.)
16. **Optimise undo memory** — consider rebuilding projections on undo rather than storing
    snapshots.

---

# Known Risks and Open Questions

| # | Risk | Detail | Mitigation |
|---|---|---|---|
| 1 | **BlockId stability** | `DomHandle` is a path (`[0]`, `[1]`) that shifts on structural edits | After every structural edit (`enter()`, paste, `undo()`) Swift must call `get_block_projections()` to refresh all IDs |
| 2 | **List prefix rendering** | Currently list prefixes are `.discardableText` attributed regions — complex offset math in `NSAttributedString+Range.swift` | Projection must encode list depth/type in `BlockKind.ListItem`; Swift renders prefix as a separate paragraph-style attribute, keeping it out of the text content |
| 3 | **Mention display length** | Mentions are 1 UTF-16 code unit in Rust but render as multi-character pills in UIKit | `InlineRunKind::Mention` carries `display_text`; Swift uses it for the pill label; Rust offsets always count 1 per mention |
| 4 | **Code/quote block backgrounds** | Rendered via `drawBackgroundStyleLayers()` in `WysiwygTextView` — currently driven by DTCoreText-parsed attributes | `FfiBlockKind.CodeBlock` / `.Quote` tells the renderer to apply background; `drawBackgroundStyleLayers()` continues to work since it reads from `NSAttributedString` attributes |
| 5 | **`replaceHtml` missing from FFI** | Paste handling calls `replaceHtml()` in WASM but it is absent from UniFFI | Add `replace_html(html, external_source)` to `ffi_composer_model.rs` as part of Phase 11 step 9 |
| 6 | **`backspace_word` / `delete_word` missing from FFI** | Present in WASM, absent in UniFFI | Add to FFI if required; not on critical path for this plan |
| 7 | **Undo stack includes full DOM** | Each undo point clones the entire `ComposerState` — memory-heavy for long documents | Out of scope for this plan; can be addressed separately with diff-based undo |

---

# Current Integration Status (Gap Analysis)

> **Updated 2026-02-24:** Phase A and B are complete. The projection renderer is now the sole
> rendering path (`useProjectionRenderer` flag removed). DTCoreText is deleted. However, the
> **reconciliation pipeline is still the old one** — `reconciliateIfNeeded()` + `StringDiffer` +
> `.htmlChars` run on every keystroke via `alwaysReconcile = true`. The new block-scoped
> reconciliation (`reconcileInlineEdit`, `computePrefixSuffixDiff`, `patchProjections`) compiles
> but is only wired into the `!alwaysReconcile` code path, which is never taken by default.
> Phase C (replacing the old reconciliation dependencies) is the current focus.

> **Previous bottom line (pre-Phase B):** All the building blocks exist (Rust projection API, FFI
> bindings, Swift renderer, Swift diff helpers) but none of them are wired into the actual iOS
> pipeline. The 103 existing iOS tests all pass because they still exercise the old DTCoreText /
> StringDiffer code path exclusively. The new code compiles but is dead code.

## What exists and works

| Component | File | Status |
|---|---|---|
| Rust `get_block_projections()` | `crates/wysiwyg/src/dom/dom_struct.rs` | ✅ Implemented, 18 unit tests passing |
| Rust `block_at_offset()` | `crates/wysiwyg/src/dom/dom_struct.rs` | ✅ Implemented, tested |
| Rust `apply_inline_edit()` | `crates/wysiwyg/src/composer_model/` | ✅ Implemented, tested |
| FFI `FfiBlockProjection` types | `bindings/wysiwyg-ffi/src/ffi_block_projection.rs` | ✅ Compiles, XCFramework built |
| FFI methods on `ComposerModel` | `bindings/wysiwyg-ffi/src/ffi_composer_model.rs` | ✅ Exported, Swift types generated |
| `ProjectionRenderer.swift` | `Sources/WysiwygComposer/Tools/ProjectionRenderer.swift` | ✅ **Active** — sole rendering path |
| `InlineReconciliation.swift` | `Sources/WysiwygComposer/Tools/InlineReconciliation.swift` | ✅ Compiles. Wired into `!alwaysReconcile` path only. |
| `isComposing` guard in Coordinator | `WysiwygComposerView.swift` | ✅ In place. Calls `reconcileBlockAfterComposition()`. |
| `ComposerModelWrapper` new methods | `ComposerModelWrapper.swift` | ✅ Implemented |
| `committedProjections` | `WysiwygComposerViewModel.swift` | ✅ Stored, updated in `applyReplaceAll()` |
| `pendingEditHint` | `WysiwygComposerViewModel.swift` | ✅ Stored, set in `replaceText()` |

## What is still actively in use (old pipeline remnants)

| Old component | Where it's called | What calls it | Phase C target? |
|---|---|---|---|
| ~~`HTMLParser.parse(html:…)`~~ | ~~`applyReplaceAll()`~~ | ~~every Rust update~~ | **Deleted** (Phase B) |
| `NSAttributedString+Range.swift` (`.htmlChars`) | `WysiwygComposerViewModel.swift` ~L700 | `reconciliateIfNeeded()`, `hasUncommitedText` | **Yes — Phase C** |
| `StringDiffer.replacement(from:to:)` | `WysiwygComposerViewModel.swift` ~L703 | `reconciliateIfNeeded()` | **Yes — Phase C** |
| `.withNBSP` normalisation | `StringDiffer.swift`, `hasUncommitedText` | `reconciliateIfNeeded()` | **Yes — Phase C** |
| ~~DTCoreText~~ | ~~`HTMLParser.swift`~~ | ~~`HTMLParser.parse()`~~ | **Deleted** (Phase B) |
| ~~`NSMutableAttributedString.swift`~~ | ~~`HTMLParser/Extensions/`~~ | ~~`HTMLParser.parse()`~~ | **Deleted** (Phase B) |

## Missing integration points (status as of 2026-02-24)

### 1. ~~`ComposerModelWrapper` does not expose the new FFI methods~~ — ✅ DONE

### 2. ~~`applyReplaceAll()` still calls `HTMLParser.parse()`~~ — ✅ DONE

Now calls `model.getBlockProjections()` → `ProjectionRenderer.render()`. Stores snapshot in
`committedProjections`. Uses direct UTF-16 offsets for selection.

### 3. ~~`replaceText(range:replacementText:)` still uses `htmlRange` offset mapping~~ — ✅ DONE

Uses direct offset pass-through with block-aware routing via `model.blockAtOffset()`.

### 4. `reconciliateIfNeeded()` still uses `StringDiffer` over `htmlChars` — **OPEN (Phase C)**

The full-document Myers diff (~L700) must be replaced with `computePrefixSuffixDiff()` using
direct UTF-16 offsets. See Phase C step C.2 in the Deletion Plan below.

### 5. ~~`applySelect()` still uses `attributedRange` mapping~~ — ✅ DONE

Uses direct `NSRange` from UTF-16 offsets.

### 6. ~~`reconcileBlockAfterComposition()` does not exist~~ — ✅ DONE

### 7. ~~No `committedProjections` state on the ViewModel~~ — ✅ DONE

### 8. ~~No `pendingEditHint` state on the ViewModel~~ — ✅ DONE

### 9. `parserStyle` / `mentionReplacer` didSet not wired to `ProjectionRenderer` — **OPEN**

When `parserStyle` changes, the ViewModel should re-render from `committedProjections`.
Currently works indirectly via the `setContentFromHtml` path calling `applyReplaceAll`.

### 10. ~~No iOS tests for any new component~~ — ✅ DONE

Zero test coverage for `ProjectionRenderer`, `InlineReconciliation`, `computePrefixSuffixDiff`,
`patchProjections`, or any FFI method via the wrapper.

---

# Current State Analysis (2026-02-24)

This section documents the **actual runtime state** after Phases A and B are complete but
before Phase C. It covers what's actively running, what's transitional, and what's dead code.

## What changed since the original gap analysis

| Item | Original status | Current status |
|---|---|---|
| Rendering | `HTMLParser.parse()` → DTCoreText | `ProjectionRenderer.render()` from `[FfiBlockProjection]` |
| `useProjectionRenderer` flag | Planned | **Removed** — projection is the only path |
| DTCoreText dependency | Active | **Deleted** from `Package.swift` |
| `applyReplaceAll()` | Called `HTMLParser.parse()` | Calls `model.getBlockProjections()` → `ProjectionRenderer.render()` |
| `applySelect()` | Used `attributedRange(from:)` mapping | Uses direct UTF-16 offsets (1:1 with Rust) |
| `select(range:)` | Used `htmlRange(from:)` mapping | Passes offsets directly to `model.select()` |
| `committedProjections` | Not implemented | ✅ Stored on ViewModel, updated in `applyReplaceAll()` |
| `pendingEditHint` | Not implemented | ✅ Stored on ViewModel, set in `replaceText()` |
| `reconciliateIfNeeded()` | Only reconciliation path | **Still the active reconciliation path** (unchanged) |
| `StringDiffer` | Core of reconciliation | **Still the active differ** (unchanged, but see fix below) |
| `.htmlChars` | Used everywhere | **Still used** by `reconciliateIfNeeded()` and `hasUncommitedText` |
| Old keyboard workarounds | Active | `updateForDoubleSpaceToDot…` and `updateDotAfterInlineTextPredication…` **deleted** |

## The `alwaysReconcile` situation

`alwaysReconcile` is a **public mutable property** on `WysiwygComposerViewModel` (line 28),
defaulting to `true`. It controls which reconciliation path runs on every keystroke.

### When `alwaysReconcile = true` (current default)

1. `replaceText(range:replacementText:)` returns `true` immediately (after two guards:
   `shouldReplaceText` and `plainTextMode`). UIKit applies the text edit itself.
2. `didUpdateText()` calls `reconciliateIfNeeded(ignoreLatinCharsCheck: true)` on **every**
   keystroke — Latin, CJK, autocorrect, predictive text, everything.
3. `reconciliateIfNeeded()` runs `StringDiffer.replacement(from: .htmlChars, to: .htmlChars)`
   — a full-document Myers diff. For simple Latin typing where Rust agrees with UIKit, the diff
   returns `nil` (fast no-op). For CJK/autocorrect, it pushes the diff into Rust.
4. The new block-scoped code (`reconcileInlineEdit`, `pendingEditHint`, `committedProjections`
   for diffing) is **never reached** because `replaceText` returns before setting a hint, and
   `didUpdateText` takes the `alwaysReconcile` branch.

### When `alwaysReconcile = false` (not the default)

1. `replaceText()` falls through to the Option B per-keystroke path: IME detection, debouncing,
   backspace handling, `isReplacingWordWithSuggestion`, direct `model.replaceText()` calls.
2. `didUpdateText()` calls `reconcileInlineEdit(hint:)` if a `pendingEditHint` exists, otherwise
   falls back to `reconciliateIfNeeded()` for CJK/IME.
3. This path uses `pendingEditHint`, `committedProjections`, `computePrefixSuffixDiff()`,
   `patchProjections()` — the new block-scoped machinery.

### Recommendation: keep `alwaysReconcile = true`

**`alwaysReconcile = true` is simpler and more robust:**
- One code path handles all keyboard types uniformly (Latin, CJK, Arabic, autocorrect,
  predictive text, dictation)
- No special-casing for IME, no debouncing, no `isReplacingWordWithSuggestion` workaround
- For Latin input the diff is usually `nil` — the `StringDiffer` work is O(n) but trivially fast
- The complexity it eliminates in `replaceText()` is substantial

**The per-keystroke path (Option B) adds complexity without meaningful performance gain:**
- IME detection and special-casing
- `isReplacingWordWithSuggestion` + `hasUncommitedText` workaround
- Debounce tracking for duplicate `replaceText` calls
- `pendingEditHint` / `reconcileInlineEdit` / `committedProjections` diffing machinery
- Link state change detection forcing reconciliation anyway
- Multiple fallback paths back to `reconciliateIfNeeded()`

**For Phase C:** Replace `reconciliateIfNeeded()` internals (swap `StringDiffer` + `.htmlChars`
for a projection-native approach using direct offsets) while keeping the `alwaysReconcile = true`
flow. Once Phase C is complete, the Option B code path and all its supporting machinery can be
deleted as dead code.

## `committedProjections` — clarification

`committedProjections: [FfiBlockProjection]` on the ViewModel is a **new-pipeline artifact**
(added as part of Phase 3/Phase 11 step 6). It stores the last projection snapshot from Rust
after each `applyReplaceAll()` call.

**Its role is dual:**
1. **Block-scoped diffing (Phase 5):** When `reconcileInlineEdit()` runs, it uses
   `committedProjections` to find the old block's text range, extract the old substring, and
   compute a block-scoped prefix/suffix diff against the current UIKit text. This is the
   *new-pipeline replacement* for the global `StringDiffer` Myers diff.
2. **Offset patching (Phase 8):** `patchProjections(blockId:newBlock:)` updates
   `committedProjections` in-place after an inline edit, shifting subsequent block offsets by
   delta, avoiding a full `get_block_projections()` call.

**The old equivalent was `committedAttributedText: NSAttributedString`** — stored after each
`applyReplaceAll()` in the old pipeline. `committedAttributedText` is still present and still
used by `hasUncommitedText` (via `.htmlChars.withNBSP` comparison). It serves the same "last
known good state" purpose but operates at the attributed-string level rather than the
projection level.

**Migration path:**
- `committedAttributedText` is used by `hasUncommitedText` → used by
  `isReplacingWordWithSuggestion` → only reached when `alwaysReconcile = false`.
  **Dead code under current defaults.** Can be removed when the Option B code path is deleted.
- `committedProjections` is used by `reconcileInlineEdit()` and `patchProjections()` → only
  called when `alwaysReconcile = false`. **Also dead code under current defaults.**
  Will become the *active* diffing state once Phase C replaces `reconciliateIfNeeded()` with
  projection-native reconciliation.

**Summary:** Both `committedProjections` (new) and `committedAttributedText` (old) coexist.
Neither is truly active under the current `alwaysReconcile = true` default — they are artefacts
of different pipeline generations. Phase C will make `committedProjections` the sole active
snapshot and delete `committedAttributedText`.

## NBSP / ZWSP / Whitespace Mechanics Analysis

The old DTCoreText pipeline introduced several whitespace-related mechanics. Under the projection
pipeline, many are obsolete but some remain necessary.

### NBSP (`\u{00A0}`)

| Mechanism | Where | Still needed? | Why |
|---|---|---|---|
| `String.nbsp` / `Character.nbsp` | `String+Character.swift` | **Yes** | Shared constant, used by `.withNBSP` |
| `.withNBSP` property | `String.swift` (WysiwygComposer) | **Yes (transitional)** | Rust emits NBSP in empty paragraphs and after mentions; UIKit uses regular spaces. Without normalisation, every such position appears as a false diff in `StringDiffer`. **Needed as long as `reconciliateIfNeeded()` uses `StringDiffer`.** |
| NBSP in Rust output | Rust core | **Permanent** | Rust inserts `\u{00A0}` after mentions and commands — this is Rust-side behaviour, not an iOS concern |
| NBSP in UI tests | `WysiwygUITests+Suggestions.swift` | **Valid** | Tests correctly assert NBSP after mentions/commands |

**StringDiffer NBSP fix (applied 2026-02-24):** `StringDiffer.replacement(from:to:)` was
normalising both strings to NBSP via `.withNBSP` for comparison, but was then extracting the
replacement text from the *normalised* copy. This caused regular spaces typed by users to be
sent to Rust as NBSP (`\u{00A0}`). **Fix:** Extract replacement text from the *original*
`newText` using `(newText as NSString).substring(with: insertion.range)` instead of
`insertion.text`. This works because space and NBSP have identical UTF-16 width, so range
indices are preserved between original and normalised strings. The `.withNBSP` normalisation
itself remains necessary for comparison — only the extraction source changed.

### ZWSP (`\u{200B}`)

| Mechanism | Where | Still needed? | Why |
|---|---|---|---|
| `String.zwsp` / `Character.zwsp` | `String+Character.swift` | **Obsolete** | `ProjectionRenderer` never injects ZWSP |
| ZWSP special case in `discardableTextRanges()` | `NSAttributedString+Range.swift` L77 | **Dead code** | Guarded against entire-string-is-ZWSP edge case from old pipeline; never triggered under projection rendering |
| ZWSP strip in UI tests | `WysiwygUITests.swift` L133 | **Harmless no-op** | Can be removed but doesn't break anything |
| ZWSP in code comments | ViewModel, `NSAttributedString.Key.swift`, `NSAttributedString+Range.swift` | **Stale** | Comments reference ZWSP placeholders and list prefixes that no longer exist in the string |

### `.discardableText` attribute

| Aspect | Detail |
|---|---|
| Defined | `NSAttributedString.Key.swift` L18 |
| Consumed by | `discardableTextRanges()` → `removeDiscardableContent()` → `htmlChars` |
| Set by | **Nobody.** In the old pipeline, DTCoreText post-processing marked ZWSP and list prefixes as `.discardableText`. `ProjectionRenderer` **never applies this attribute**. |
| Verdict | **Dead code.** The enumeration always finds nothing. Can be deleted in Phase C. |

### `htmlChars` property

```swift
public var htmlChars: String {
    NSMutableAttributedString(attributedString: self)
        .removeDiscardableContent()      // always a no-op (nothing is discardable)
        .addPlaceholderForReplacements() // may still be relevant for mention pills
        .string
}
```

**Current callers:**
1. `hasUncommitedText` — `textView.attributedText.htmlChars.withNBSP != committedAttributedText.htmlChars.withNBSP`
2. `reconciliateIfNeeded()` — `StringDiffer.replacement(from: attributedContent.text.htmlChars, to: textView.attributedText.htmlChars)`

**Under projection rendering:** `.removeDiscardableContent()` is always a no-op (no
`.discardableText` attributes exist). `.addPlaceholderForReplacements()` may still be needed
if mention pills have a display length different from Rust's model length (1 UTF-16 code unit).
However, with the projection renderer, mention display is handled by `InlineRunKind.Mention`
and offsets are managed by Rust — making the placeholder step increasingly redundant.

**Phase C simplification:** Replace `htmlChars` usage entirely. Since projection output has 1:1
UTF-16 offset mapping with Rust, `reconciliateIfNeeded()` replacement can use
`textView.text` directly (or `textView.attributedText.string`). `hasUncommitedText` can be
replaced with `textView.text != committedAttributedText.string` (or removed entirely since
it's dead code under `alwaysReconcile = true`).

## What can be removed now (safe, dead under `alwaysReconcile = true`)

These are dead code under the current default and can be removed immediately:

| Symbol | Reason it's dead |
|---|---|
| `hasUncommitedText` | Only caller is `isReplacingWordWithSuggestion`, which is in Option B (`!alwaysReconcile`) |
| `isReplacingWordWithSuggestion` | In Option B code path, never reached |
| Option B code in `replaceText()` (~lines 340-458) | `alwaysReconcile` returns `true` at line 333 before this code |
| `isExitingPredictiveText` | Already removed (search returns zero matches) |

These are dead code but should wait for Phase C (they're wired into the new reconciliation
that Phase C will activate):

| Symbol | Reason to keep for now |
|---|---|
| `committedProjections` | Will become the active snapshot in Phase C reconciliation |
| `pendingEditHint` / `EditHint` | Will be used by Phase C block-scoped reconciliation |
| `reconcileInlineEdit()` | Will replace `reconciliateIfNeeded()` in Phase C |
| `computePrefixSuffixDiff()` | Used by `reconcileInlineEdit()` |
| `patchProjections()` | Used after inline edit in Phase C flow |

---

# Expected End State

- No global diffs — all reconciliation is block-scoped.
- No HTML round-trip on the iOS rendering path.
- No DTCoreText dependency.
- No offset mapping layer (`discardableText`, `htmlChars`, `htmlRange ↔ attributedRange`).
- No iOS version–specific auto-correction workarounds.
- Deterministic, block-scoped reconciliation.
- Stable cursor behaviour across autocorrect, IME, and rapid typing.
- Rust owns all offset arithmetic.
- CRDT-ready: block projection is the natural unit of synchronisation.

---

# Implementation Checklist

## Rust Core

- [x] Create `crates/wysiwyg/src/block_projection.rs` with `AttributeSet`, `InlineRunKind`,
      `InlineRun`, `BlockKind`, `BlockProjection` types
- [x] Implement `Dom::get_block_projections()` — iterates top-level block children of root Generic
      node, flattens nested formatting containers into `AttributeSet`
- [x] Implement `Dom::block_at_offset(offset_utf16)` — uses `find_range()` + handle ancestor walk
- [x] Implement `ComposerModel::apply_inline_edit(block_id, start, end, replacement)` — delegates
      to existing `Dom::replace_text_in()`
- [x] Merge adjacent `InlineRun`s with identical `AttributeSet` in projection builder
- [x] Handle `ContainerNodeKind::Link` — populate `link_url` in `AttributeSet`
- [x] Handle `MentionNode` → `InlineRunKind::Mention` with `url` + `display_text`
- [x] Handle `LineBreakNode` → `InlineRunKind::LineBreak`
- [x] Handle `ListItem` depth computation (walk ancestor `List` containers)
- [x] Export `block_projection` module from `crates/wysiwyg/src/lib.rs`
- [x] Unit test: `get_block_projections()` on paragraph + list + code block + quote document
- [x] Unit test: projection offsets are contiguous and match `dom.text_len()`
- [x] Unit test: nested `<em><strong>` → both flags set in `AttributeSet`
- [x] Unit test: `block_at_offset` resolves correctly at start / middle / end / boundary
- [x] Unit test: `apply_inline_edit` inside bold run
- [x] Unit test: `apply_inline_edit` at attribute boundary
- [x] Unit test: `apply_inline_edit` covers partial deletion across formatting containers
- [ ] Unit test: `apply_inline_edit` with mention adjacent — mention stays atomic
- [ ] Unit test: Unicode surrogate pairs and grapheme clusters at block boundaries
- [x] Unit test: projection after structural edit (Enter) — block IDs and offsets correct

## FFI Bindings (UniFFI)

- [x] Create `bindings/wysiwyg-ffi/src/ffi_block_projection.rs` with `FfiAttributeSet`,
      `FfiInlineRunKind`, `FfiInlineRun`, `FfiBlockKind`, `FfiBlockProjection`
- [x] Add `get_block_projections() -> Vec<FfiBlockProjection>` to `ffi_composer_model.rs`
- [x] Add `block_at_offset(offset_utf16: u32) -> Option<String>` to `ffi_composer_model.rs`
- [x] Add `apply_inline_edit(block_id, start, end, replacement) -> Arc<ComposerUpdate>` to
      `ffi_composer_model.rs`
- [x] Register new types in `lib.rs` / update `into_ffi.rs` conversions
- [x] Verify UniFFI code-gen produces correct Swift types (build XCFramework, inspect generated
      Swift)

## iOS — Rendering

- [x] Create `ProjectionRenderer.swift` — builds `NSAttributedString` from `[FfiBlockProjection]`
- [x] Map `FfiAttributeSet` → `UIFont` (bold/italic font combinations using `UIFontDescriptor`)
- [x] Map `FfiAttributeSet.link_url` → `.link` attributed string attribute
- [x] Map `FfiBlockKind.CodeBlock` → monospace font + code background colour attribute
- [x] Map `FfiBlockKind.Quote` → paragraph left-indent + accent border style
- [x] Map `FfiBlockKind.ListItem` → paragraph indent + prefix string (replaces `.discardableText`)
- [x] Map `InlineRunKind.Mention` → invoke `MentionReplacer` to produce pill attachment
- [ ] Add snapshot tests comparing `ProjectionRenderer` output vs current DTCoreText output
- [x] Feature-flag `useProjectionRenderer` to allow A/B comparison (flag added then removed —
      projection is now the sole path)
- [x] Replace `applyReplaceAll()` in `WysiwygComposerViewModel` to call `ProjectionRenderer`
- [x] Remove DTCoreText from `Package.swift` dependencies
- [ ] Delete `HTMLParser/` SPM target (Phase D — shared files must be moved first)
- [x] Delete `NSMutableAttributedString.swift` post-processing
- [ ] Delete `NSAttributedString+Range.swift` offset mapping (Phase C — still used by
      `reconciliateIfNeeded()` and `hasUncommitedText`)

## iOS — Model Wrapper Integration (NEW — required before any pipeline switch)

- [x] Add `getBlockProjections() -> [FfiBlockProjection]` to `ComposerModelWrapperProtocol` and
      `ComposerModelWrapper`
- [x] Add `blockAtOffset(offsetUtf16: UInt32) -> String?` to `ComposerModelWrapperProtocol` and
      `ComposerModelWrapper`
- [x] Add `applyInlineEdit(blockId:replaceStartUtf16:replaceEndUtf16:replacementText:)
      -> ComposerUpdate` to `ComposerModelWrapperProtocol` and `ComposerModelWrapper`

## iOS — ViewModel State & Wiring (NEW — connects new code to the pipeline)

- [x] Add `committedProjections: [FfiBlockProjection]` property to `WysiwygComposerViewModel`
- [x] Add `useProjectionRenderer: Bool` feature flag to `WysiwygComposerViewModel`
      (added in Phase B step 7, then **removed** when projection became the sole path)
- [x] Add `pendingEditHint: EditHint?` property to `WysiwygComposerViewModel`
- [x] Replace `applyReplaceAll()` body: call `model.getBlockProjections()` →
      `ProjectionRenderer.render()` → store in `committedProjections` → use direct UTF-16 offsets
      for selection (no `htmlRange` / `attributedRange` mapping)
- [x] Simplify `applySelect()`: remove `attributedRange(from:)` mapping, use direct `NSRange`
      from UTF-16 offsets
- [x] Simplify `select(range:)`: remove `htmlRange(from:)` mapping, pass offsets directly to
      `model.select()`
- [x] Simplify `replaceText(range:replacementText:)`: add block-aware inline vs structural routing
      via `model.blockAtOffset()` in the projection path
- [ ] Update `parserStyle` didSet / `mentionReplacer` to force a full re-render from projections
      (currently works via the existing `setContentFromHtml` path which calls `applyReplaceAll`
      and picks up the new style automatically)

## iOS — Edit Reconciliation

- [x] Add `struct EditHint { blockId: String, range: NSRange, replacement: String }` to ViewModel
- [x] Add `struct InlineDiff { replaceStart, replaceEnd, replacement }` helper
- [x] Implement `computePrefixSuffixDiff(old: String, new: String) -> InlineDiff`
- [x] ~~Replace `replaceText(range:replacementText:)` routing logic with block-aware routing~~
      Replaced with clean path: `replaceText` always returns `true`, `reconcileNative()` handles all
- [x] ~~Implement `reconcileInlineEdit(hint: EditHint)` — block-scoped diff → `applyInlineEdit()`~~
      Removed: `reconcileNative()` handles all cases via full-document prefix/suffix diff
- [x] Implement `patchProjections(blockId:newBlock:)` — O(blocks) offset shift without full
      `get_block_projections()` call
- [x] Wire `reconcileInlineEdit()` into `didUpdateText()` when `pendingEditHint` is set
- [x] Replace `reconciliateIfNeeded()` with `reconcileInlineEdit()` for the inline case
- [x] ~~Keep `reconciliateIfNeeded()` (or equivalent) only as fallback~~ — replaced entirely
      by `reconcileNative()` which handles all cases
- [x] Replace `StringDiffer.swift` global Myers diff — deleted file, replaced by
      `computePrefixSuffixDiff()` in `reconcileNative()`

## iOS — IME Composition Guard

- [x] Add `isComposing: Bool` state to `Coordinator` in `WysiwygComposerView.swift`
- [x] In `textViewDidChange`, check `textView.markedTextRange != nil` → set `isComposing = true`,
      return without calling Rust
- [x] On composition end (`isComposing == true`, `markedTextRange == nil`), call
      `reconcileBlockAfterComposition()`
- [x] Implement `reconcileBlockAfterComposition()` on `WysiwygComposerViewModel` — block-scoped
      full re-diff for the affected block
- [x] ~~Remove `hasUncommitedText` / `committedAttributedText` CJK workaround~~ —
      `hasUncommitedText` deleted; `committedAttributedText` KEPT as "last known good" state
      for `reconcileNative()`

## iOS — Unit Tests for New Components (NEW)

- [x] Test `ProjectionRenderer.render()` — paragraph with bold/italic produces correct attributes
- [x] Test `ProjectionRenderer.render()` — code block produces monospace font
- [x] Test `ProjectionRenderer.render()` — list items produce correct paragraph indent
- [x] Test `ProjectionRenderer.render()` — mention produces pill or link fallback
- [x] Test `ProjectionRenderer.render()` — multi-block document produces `\n`-separated blocks
- [x] Test `computePrefixSuffixDiff()` — single character insertion
- [x] Test `computePrefixSuffixDiff()` — deletion
- [x] Test `computePrefixSuffixDiff()` — replacement (different lengths)
- [x] Test `computePrefixSuffixDiff()` — empty old string
- [x] Test `computePrefixSuffixDiff()` — empty new string
- [x] Test `computePrefixSuffixDiff()` — identical strings (no change)
- [x] Test `patchProjections()` — offset shift propagates to subsequent blocks
- [x] Test `ComposerModelWrapper.getBlockProjections()` returns expected types
- [x] Test `ComposerModelWrapper.blockAtOffset()` returns correct block ID
- [x] Test `ComposerModelWrapper.applyInlineEdit()` returns valid `ComposerUpdate`

## iOS — Workaround Removal (deferred to step 10)

- [x] Remove `updateForDoubleSpaceToDotConversionIfNeeded()` (deleted in Phase B)
- [x] Remove `updateDotAfterInlineTextPredicationIfNeeded()` (deleted in Phase B)
- [x] Remove `isReplacingWordWithSuggestion` special-casing (deleted with Option B code path)
- [x] Remove `isExitingPredictiveText` special-casing (already absent from codebase)
- [x] Remove NBSP normalisation in diff path (`.withNBSP` deleted from `String.swift`,
      `StringDiffer` deleted)
- [ ] Remove `selectedRange = .zero` autocapitalisation workaround (verify it is no longer needed
      with the new renderer)

## Missing FFI Methods (add when needed)

- [ ] Add `replace_html(html: String, external_source: bool)` to `ffi_composer_model.rs`
      (currently WASM-only — needed for multiline paste on iOS)

---

## iOS — Old Pipeline Deletion Assessment

This section catalogues every file/type involved in the old HTML→DTCoreText→NSAttributedString
pipeline and classifies it as **deletable now**, **shared** (still needed by the new
ProjectionRenderer path), or **deletable once old path is removed**.

### Classification key

| Label | Meaning |
|-------|---------|
| **OLD-ONLY** | Used exclusively by the DTCoreText/HTMLParser rendering path. Can be deleted once `useProjectionRenderer = true` is the sole code path. |
| **SHARED** | Referenced by both old and new pipelines — must be kept (or migrated to `WysiwygComposer` target) before the `HTMLParser` module can be deleted. |
| **NEW-ONLY** | Only used by the new projection pipeline. |

---

### `Sources/DTCoreTextExtended/` (entire module)

| File | Classification | Notes |
|------|---------------|-------|
| `include/UIFont+AttributedStringBuilder.h` | OLD-ONLY | ObjC header for DTCoreText font swizzle fix |
| `UIFont+AttributedStringBuilder.m` | OLD-ONLY | Patches `fontWithCTFont:` for iOS 13+; irrelevant to ProjectionRenderer |

**Verdict**: Delete entire directory + remove SPM target once old path is removed.

---

### `Sources/HTMLParser/` — root files

| File | Classification | Notes |
|------|---------------|-------|
| `HTMLParser.swift` | OLD-ONLY | The `HTMLParser.parse(html:…)` class — calls DTCoreText. Only invoked from `applyReplaceAll` old-path branch. |
| `HTMLParserStyle.swift` | **SHARED** | Struct used by `ProjectionRenderer.style` **and** `WysiwygComposerViewModel.parserStyle`. Must be kept or moved. |
| `HTMLMentionReplacer.swift` | **SHARED** | Protocol used by `ProjectionRenderer.mentionReplacer` and `MentionReplacer` (in `Tools/`). Must be kept or moved. |
| `BlockStyle.swift` | **SHARED** | `.blockStyle` NSAttributedString attribute is read by `drawBackgroundStyleLayers()` in both pipelines. `BlockStyle.paragraphStyle` sets indent/spacing. |
| `HTMLParserHelpers.swift` | OLD-ONLY | `TempColor` enum — sentinel colours for DTCoreText CSS post-processing. |
| `MentionContent.swift` | OLD-ONLY* | Used by `NSAttributedString+Range.swift` (`.htmlChars`) which is still called by `hasUncommitedText` and `reconciliateIfNeeded()` in the projection fallback. *Becomes deletable once `.htmlChars` dependency is removed (see below).* |
| `MentionReplacement.swift` | OLD-ONLY* | Same as `MentionContent` — only referenced by range-translation code. |
| `BuildHTMLAttributedError.swift` | OLD-ONLY | Error type thrown only by `HTMLParser.parse()`. |

---

### `Sources/HTMLParser/Extensions/`

| File | Classification | Notes |
|------|---------------|-------|
| `NSAttributedString+Range.swift` | **SHARED** (transitional) | `.htmlChars` is called in: `hasUncommitedText`, `reconciliateIfNeeded()`, `updateForDoubleSpaceToDot…` (old-only). The projection pipeline still falls back to `reconciliateIfNeeded()` which uses `.htmlChars` and `.htmlRange(from:)`. **Becomes deletable** once `reconciliateIfNeeded()` is replaced with a projection-native fallback and `hasUncommitedText` is simplified to a plain string comparison. |
| `NSAttributedString+Attributes.swift` | **SHARED** | `enumerateTypedAttribute` used by `drawBackgroundStyleLayers()`. |
| `NSAttributedString.Key.swift` | **SHARED** (partially) | `.blockStyle` — used by both pipelines. `.discardableText`, `.mention` — only used by old range translation (`.htmlChars`). `.DTTextBlocks`, `.DTField`, `.DTTextLists` — old-only DTCoreText keys. |
| `NSMutableAttributedString.swift` | OLD-ONLY | `applyPostParsingCustomAttributes`, `replaceMentions` — called from `HTMLParser.parse()`. Contains `import DTCoreText`. |
| `NSRange.swift` | OLD-ONLY | `.excludingLast` — only used in `applyBackgroundStyles`. |
| `UITextView.swift` | **SHARED** | `drawBackgroundStyleLayers()` — called from `WysiwygTextView.apply()` and `.draw()`. Both pipelines produce `.blockStyle` attributes. |
| `NSParagraphStyle.swift` | **SHARED** | `mut()` helper used by `BlockStyle.paragraphStyle`. |
| `String+Character.swift` | **SHARED** | `.nbsp`, `.zwsp`, `.slash`, `.lineSeparator` etc. Used broadly. |
| `CGRect.swift` | **SHARED** | `.extendHorizontally(in:withVerticalPadding:)` used by `drawBackgroundStyleLayers()`. |
| `UIColor.swift` | OLD-ONLY | `.toHexString()` — only used by `TempColor` in `HTMLParser.defaultCSS`. |

---

### `Sources/HTMLParser/Extensions/DTCoreText/`

| File | Classification | Notes |
|------|---------------|-------|
| `DTHTMLElement.swift` | OLD-ONLY | `DTHTMLElement.sanitize()` — mention node handling during DTCoreText parse. |
| `PlaceholderTextHTMLElement.swift` | OLD-ONLY | DTCoreText subclasses for discardable/mention text. |

**Verdict**: Delete entire subdirectory once old path is removed.

---

### `Sources/WysiwygComposer/Tools/`

| File | Classification | Notes |
|------|---------------|-------|
| `StringDiffer.swift` | **SHARED** (transitional) | Myers diff used in `reconciliateIfNeeded()`. The projection pipeline falls back to this for structural edits. **Deletable** once `reconciliateIfNeeded()` is replaced. |
| `MentionReplacer.swift` | **SHARED** | Protocol extends `HTMLMentionReplacer`. Must be kept or the dependency chain refactored. |
| `ProjectionRenderer.swift` | NEW-ONLY | — |
| `InlineReconciliation.swift` | NEW-ONLY | — |

---

### `Sources/WysiwygComposer/Extensions/`

| File | Classification | Notes |
|------|---------------|-------|
| `CollectionDifference.swift` | **SHARED** (transitional) | Used by `StringDiffer`. Deletable when `StringDiffer` is removed. |

---

### `WysiwygComposerViewModel.swift` — old-path–only code blocks

These sections of the ViewModel are **only reached** when `useProjectionRenderer == false`:

| Symbol / block | Lines (approx) | Notes |
|----------------|-----------------|-------|
| `HTMLParser.parse(html:…)` call | `applyReplaceAll` else branch | DTCoreText rendering |
| `.attributedRange(from:)` calls | `applyReplaceAll`, `applySelect` else branches | Offset translation |
| `.htmlRange(from:)` calls | `select()` else branch, `reconciliateIfNeeded()` resync | Offset translation |
| `updateForDoubleSpaceToDotConversionIfNeeded()` | `didUpdateText` else branch | iOS keyboard workaround |
| `updateDotAfterInlineTextPredicationIfNeeded()` | `didUpdateText` else branch | iOS keyboard workaround |

---

### Test files

| File | Classification |
|------|---------------|
| `Tests/HTMLParserTests/HTMLParserTests.swift` | OLD-ONLY — tests DTCoreText output |
| `Tests/HTMLParserTests/HTMLParserTests+PermalinkReplacer.swift` | OLD-ONLY |
| `Tests/HTMLParserTests/Extensions/NSAttributedStringRangeTests.swift` | OLD-ONLY — tests `htmlRange`/`attributedRange` |
| `Tests/HTMLParserTests/Extensions/NSAttributedStringAttributesTests.swift` | SHARED — tests `enumerateTypedAttribute` |
| `Tests/HTMLParserTests/Extensions/UIColorExtensionsTests.swift` | OLD-ONLY |
| `Tests/WysiwygComposerTests/UITextViewTests.swift` | SHARED — tests `drawBackgroundStyleLayers` |
| `Tests/WysiwygComposerTests/Tools/StringDifferTests.swift` | SHARED (transitional) |

---

### `Package.swift` dependency chain

```
DTCoreText (external, exact 1.6.26)
  └── DTCoreTextExtended (target)
        └── HTMLParser (target, depends on DTCoreText + DTCoreTextExtended)
              └── WysiwygComposer (target, depends on HTMLParser + WysiwygComposerFFI)
```

The `DTCoreText` external dependency + `DTCoreTextExtended` target can be removed entirely
once `HTMLParser` no longer contains `import DTCoreText` in any file.

---

## iOS — Deletion Plan (phased)

### Phase A — Delete immediately (pure old-pipeline, zero shared usage)

These can be deleted **right now** without affecting either pipeline, because they are only
imported/called from files that are themselves OLD-ONLY:

- [x] Delete `Sources/HTMLParser/HTMLParserHelpers.swift` (`TempColor`)
      → Inlined as `HTMLParserSentinelColor` private enum in `HTMLParser.swift`
- [x] Delete `Sources/HTMLParser/BuildHTMLAttributedError.swift`
      → Replaced with private `HTMLParserError` in `HTMLParser.swift`
- [x] Delete `Sources/HTMLParser/Extensions/UIColor.swift` (`toHexString()`)
      → Hardcoded hex strings in `HTMLParser.swift` CSS
- [x] Delete `Sources/HTMLParser/Extensions/NSRange.swift` (`.excludingLast`)
      → Inlined as `NSRange(location: range.location, length: max(0, range.length - 1))` in
      `NSMutableAttributedString.swift`
- [x] Delete corresponding reference cleanup in `HTMLParser.swift` and
      `NSMutableAttributedString.swift`
- [x] Delete `Tests/HTMLParserTests/Extensions/UIColorExtensionsTests.swift`
      → Old test for `toHexString()`; updated `testInvalidEncodingString` to use
      `XCTAssertThrowsError` instead of specific error type check

> **Note**: `Sources/HTMLParser/Extensions/DTCoreText/DTHTMLElement.swift` and
> `PlaceholderTextHTMLElement.swift` were originally listed in Phase A but must
> stay until Phase B — the old `HTMLParser.parse()` `willFlushCallback` still
> calls `element.sanitize()` which is defined there, and the old pipeline tests
> depend on mention/NBSP handling. Those files move to Phase B below.

### Phase B — Delete after setting `useProjectionRenderer = true` permanently

Once the feature flag is flipped to `true` by default and the old `else` branches in the
ViewModel are deleted:

- [x] Delete `Sources/HTMLParser/Extensions/DTCoreText/DTHTMLElement.swift`
- [x] Delete `Sources/HTMLParser/Extensions/DTCoreText/PlaceholderTextHTMLElement.swift`
- [x] Delete `Sources/HTMLParser/Extensions/DTCoreText/` directory
- [x] Delete `Sources/HTMLParser/HTMLParser.swift` (the class)
- [x] Delete `Sources/HTMLParser/Extensions/NSMutableAttributedString.swift`
  (contains `import DTCoreText`)
- [x] Delete `Sources/DTCoreTextExtended/` entire directory
- [x] Remove `DTCoreText` external package dependency from `Package.swift`
- [x] Remove `DTCoreTextExtended` target from `Package.swift`
- [x] Remove DTCoreText attribute keys (`.DTTextBlocks`, `.DTField`, `.DTTextLists`) from
      `NSAttributedString.Key.swift`
- [x] Delete old-path ViewModel code: `HTMLParser.parse()` call, `.attributedRange(from:)`,
      `.htmlRange(from:)` in `select()`, `updateForDoubleSpaceToDot…`,
      `updateDotAfterInlineTextPredication…` (also removed `useProjectionRenderer` flag entirely)
- [x] Delete `Tests/HTMLParserTests/HTMLParserTests.swift`
- [x] Delete `Tests/HTMLParserTests/HTMLParserTests+PermalinkReplacer.swift`
- [x] Delete `Tests/HTMLParserTests/Extensions/NSAttributedStringRangeTests.swift`

### Phase C — Replace shared transitional dependencies — **COMPLETE (2026-02-24)**

Phase C replaced the old reconciliation pipeline with a clean projection-native path.

#### What was done

1. **New `reconcileNative()` method** — uses `computePrefixSuffixDiff(old:new:)` on
   `committedAttributedText.string` vs `textView.attributedText.string`, then
   `model.replaceTextIn()` with direct UTF-16 offsets. No `.htmlChars`, no `.htmlRange`,
   no Myers diff, no NBSP normalisation.

2. **Simplified `replaceText()`** — always returns `true` (UIKit owns all text mutations).
   Removed: Option B per-keystroke Rust path, IME language detection, debounce tracking,
   backspace handling, link boundary detection, `pendingEditHint` assignment. Kept: typing
   attribute reset on empty text view.

3. **Simplified `didUpdateText()`** — removed `alwaysReconcile`/IME branching, now just calls
   `reconcileNative()` + `applyPendingFormatsIfNeeded()`.

4. **Deleted dead properties**: `alwaysReconcile`, `lastReplaceTextUpdate`, `pendingEditHint`,
   `hasUncommitedText`, `ReplaceTextUpdate` struct.

5. **Deleted dead methods**: `reconciliateIfNeeded()`, `reconcileInlineEdit()`,
   `reconcileBlockAfterComposition()`.

6. **Deleted files**:
   - `StringDiffer.swift` + `StringDifferTests.swift`
   - `CollectionDifference.swift` + `CollectionDifferenceTests.swift`
   - `String+LatinLangugesTests.swift`

7. **Cleaned up**:
   - Removed `.withNBSP` and `.containsLatinAndCommonCharactersOnly` from `String.swift`
   - Removed `EditHint` struct from `InlineReconciliation.swift`
   - Updated stale NBSP/ZWSP comment in `createEnterUpdate()`

8. **Updated tests** — all tests now use `simulateTyping()` helper that simulates the full
   UIKit cycle: `replaceText` → text view update → `didUpdateText`. Link boundary tests
   rewritten for the new flow. 113 tests pass (1 pre-existing failure unrelated to this work).

#### Items kept (not dead code)

- `committedAttributedText` — used by `reconcileNative()` as "last known good" state
- `committedProjections` — stored in `applyReplaceAll()`, useful for future optimization
- `computePrefixSuffixDiff()` and `patchProjections()` in `InlineReconciliation.swift`
- `.htmlChars` definition in `NSAttributedString+Range.swift` (HTMLParser module, may have external consumers)

#### Step C.1 — Remove Option B dead code

These are dead code under `alwaysReconcile = true` and can be deleted without replacement:

- [x] Delete `hasUncommitedText` computed property (only caller is `isReplacingWordWithSuggestion`)
- [x] Delete `isReplacingWordWithSuggestion` and surrounding logic in `replaceText()`
- [x] Delete the entire Option B code path in `replaceText()` (lines ~340-458) — everything
      after the `alwaysReconcile` early return. This includes: IME language detection, debounce
      tracking, backspace handling, `model.replaceText()` direct calls, link state check,
      `pendingEditHint` assignment
- [x] ~~Delete `committedAttributedText` property~~ — KEPT: used by `reconcileNative()` as
      "last known good" state for diffing
- [x] Delete `updateForDoubleSpaceToDotConversionIfNeeded()` if still present
- [x] Delete `updateDotAfterInlineTextPredicationIfNeeded()` if still present
- [x] Update stale ZWSP/NBSP comments (e.g. `createEnterUpdate()` line ~832 references
      "representation chars" that no longer exist)

#### Step C.2 — Replace `reconciliateIfNeeded()` with projection-native reconciliation

Replace the body of `reconciliateIfNeeded()` (or rename it) to use direct UTF-16 offsets and
`committedProjections` instead of `StringDiffer` + `.htmlChars`:

```swift
// Conceptual replacement — projection-native reconciliation
func reconciliateIfNeeded() {
    guard !isDictating else { return }

    // Under projection rendering, offsets are 1:1 — no .htmlChars needed.
    // Use committedProjections to scope the diff to the affected block.
    let oldText = committedAttributedString.string  // or build from committedProjections
    let newText = textView.attributedText.string

    guard oldText != newText else { return }  // fast no-op for matching strings

    // Option A: full-document prefix/suffix diff (simple, same perf as StringDiffer)
    let diff = computePrefixSuffixDiff(old: oldText, new: newText)
    let update = model.replaceTextIn(
        newText: diff.replacement,
        start: UInt32(diff.replaceStart),
        end: UInt32(diff.replaceEnd)
    )
    applyUpdate(update)

    // Cursor resync: with 1:1 offsets, use textView.selectedRange directly
    let rustSelection = textView.selectedRange
    model.select(startUtf16Codeunit: UInt32(rustSelection.location),
                 endUtf16Codeunit: UInt32(rustSelection.location + rustSelection.length))
}
```

Key changes vs the old implementation:
- No `.htmlChars` extraction (`.removeDiscardableContent()` + `.addPlaceholderForReplacements()`)
- No `.withNBSP` normalisation (not needed when both sides use the same offset space)
- No `.htmlRange(from:)` for cursor resync (1:1 offsets)
- Uses `computePrefixSuffixDiff()` (already implemented in `InlineReconciliation.swift`)
  instead of `StringDiffer` Myers diff

**Note on NBSP:** With projection rendering, Rust emits NBSP in specific positions (empty
paragraphs, post-mention). `ProjectionRenderer` passes these through verbatim into the
NSAttributedString. UIKit may or may not preserve them when the user edits adjacent text.
If NBSP false-positive diffs reappear, a targeted normalisation can be added to
`computePrefixSuffixDiff()` — but the `.withNBSP` blanket normalisation should not be needed
since both `oldText` and `newText` come from NSAttributedString (same representation).

**Note on mentions:** Under projection rendering, mentions are rendered via
`MentionReplacer.replacementForMention()` which produces a pill NSAttributedString. The pill's
string length in UIKit may differ from Rust's 1-code-unit model. If mention offset mismatches
cause problems, `computePrefixSuffixDiff()` will need mention-aware handling. However, since
`committedProjections` carries mention positions and display text, this can be handled at the
projection level rather than the attributed-string level.

- [x] Replace `reconciliateIfNeeded()` body with projection-native implementation
- [x] Verify NBSP handling — confirmed no false-positive diffs with the new approach
- [ ] Verify mention handling — confirm mention offset integrity
- [ ] Add unit tests for the new reconciliation path

#### Step C.3 — Delete old reconciliation dependencies

Once `reconciliateIfNeeded()` no longer calls `StringDiffer` or `.htmlChars`:

- [x] Delete `StringDiffer.swift`
- [x] Delete `CollectionDifference.swift` extension (only consumer is `StringDiffer`)
- [x] Delete `Tests/WysiwygComposerTests/Tools/StringDifferTests.swift`
- [ ] Delete `MentionContent.swift` (only used by `.htmlChars` → `.addPlaceholderForReplacements()`)
- [ ] Delete `MentionReplacement.swift` (same)
- [x] Remove `.withNBSP` property from `String.swift` (no longer needed for diff normalisation)
- [ ] Remove `.discardableText` and `.mention` attribute keys from `NSAttributedString.Key.swift`
- [ ] Simplify or delete `NSAttributedString+Range.swift` — only keep any helpers still used
      by the projection path (likely none)

#### Step C.4 — Clean up `alwaysReconcile` flag

Once Phase C reconciliation is working and the Option B code path has been deleted:

- [x] Remove `alwaysReconcile` property entirely (it will always be `true` conceptually —
      the projection-native reconciliation handles all cases)
- [x] Simplify `didUpdateText()` to always call the (now projection-native) reconciliation
- [x] Remove `reconcileInlineEdit()` and `reconcileBlockAfterComposition()` if they are
      subsumed by the new `reconciliateIfNeeded()` (or refactor them into the single path)

### Phase D — Extract remaining shared types out of HTMLParser module

After Phases A–C, the only files still needed from `Sources/HTMLParser/` will be:

| File | Needed by |
|------|-----------|
| `HTMLParserStyle.swift` | `ProjectionRenderer`, ViewModel |
| `HTMLMentionReplacer.swift` | `ProjectionRenderer`, `MentionReplacer` protocol |
| `BlockStyle.swift` | `drawBackgroundStyleLayers()`, `ProjectionRenderer` (via `.blockStyle` attr) |
| `Extensions/UITextView.swift` | `WysiwygTextView` |
| `Extensions/NSAttributedString+Attributes.swift` | `drawBackgroundStyleLayers()` |
| `Extensions/NSAttributedString.Key.swift` | `.blockStyle` attribute |
| `Extensions/NSParagraphStyle.swift` | `BlockStyle.paragraphStyle` |
| `Extensions/String+Character.swift` | Broadly used |
| `Extensions/CGRect.swift` | `drawBackgroundStyleLayers()` |

Steps:

- [ ] Move the files listed above into `Sources/WysiwygComposer/` (e.g. a new
      `Extensions/Shared/` or `Styling/` subdirectory)
- [ ] Update `import HTMLParser` → no import needed (same module) in `ProjectionRenderer.swift`,
      `MentionReplacer.swift`, `WysiwygComposerViewModel.swift`
- [ ] Delete the now-empty `Sources/HTMLParser/` directory
- [ ] Remove `HTMLParser` target from `Package.swift`
- [ ] Remove `HTMLParserTests` test target from `Package.swift`
- [ ] Update `WysiwygComposer` target dependencies to remove `HTMLParser`

### Summary table

| Phase | Deletes | Prerequisite |
|-------|---------|-------------|
| **A** | 8 old-only files + 1 test | None — safe now |
| **B** | `HTMLParser` class, DTCoreText module, old ViewModel branches, 3 test files | `useProjectionRenderer` default → `true`, old branches removed |
| **C** | `StringDiffer`, `CollectionDifference`, `MentionContent`, `MentionReplacement`, old range extensions | Write projection-native `hasUncommitedText` and fallback reconciliation |
| **D** | `HTMLParser` SPM module entirely | Move ~9 shared files into `WysiwygComposer` target |
- [ ] Add `backspace_word()` and `delete_word()` to FFI if required