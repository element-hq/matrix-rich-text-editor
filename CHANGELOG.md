# Changelog

# [2.38.1]
- [Android] Workaround bug that caused the next focus view search loop when the editor is used in a Compose context, causing ANRs.
- [Android] Fix crashes when either the editor or the rendering views contain a code block and their size changes, making the previously calculated and cached coordinates for rendering the code block invalid.
- [Common] Change licensing to AGPLv3 + Element Commercial license. 

# [2.38.0]
- [Rust] Bump Rust version to `1.76`.
- [Web] Bump wasm-pack version to `0.13.1`.
- [Web] Switch bindings/wysiwyg-wasm from npm to yarn.
- [Web] Add typescript checking to bindings/wysiwyg-wasm.
- [Web] Update vite and related packages.
- [Web] Simplify build scripts.
- [Web] Publish a dual CJS/ESM package with platform-specific loaders.

# [2.37.14]
- [Android] Have separate modes for parsing HTML for 'editor mode' and 'message mode' using `isEditor: Boolean` parameter.
- [Android] Try auto-selecting the latest NDK version available instead of using a hardcoded one.
- [Web] Update example app to use the new npm package.

# [2.37.13] - 2024-10-04
- [Android] Make `RichTextEditorStyle` and other data classes' constructors public to match their `copy` methods in Kotlin 2.0. This would cause build errors on Kotlin 2.1.

# [2.37.12] - 2024-10-02

- [Common] Dependencies update
- [iOS] fixed a release bug that prevented the XCFramework to be downloaded from SwiftPM.

# [2.37.11] - 2024-10-01

- [Common] matrix-rich-text-editor is being forked by Element at https://github.com/element-hq/matrix-rich-text-editor. Contributions are licensed to Element under a CLA and made available under an AGPLv3.0 license.
  You can read more about this here:
  - https://matrix.org/blog/2024/08/heart-of-matrix/
  - https://element.io/blog/sustainable-licensing-at-element-with-agpl/
- [Web] The matrix-wysiwyg npm package will now be published at https://www.npmjs.com/package/@vector-im/matrix-wysiwyg

# [2.37.9] - 2024-08-28
- [Rust] Escape text passed to `ComposerModel::set_link_with_text` and `insert_mention*` methods of the WASM bindings.

# [2.37.8] - 2024-07-31
- [Web] Fix a bug that caused the composer to be reinitialised a second time.

# [2.37.7] - 2024-07-18
- [Common] Add auto completion for emojis.
- [Android] Fix issues with new lines by replacing the HTML parser with Jsoup.
- [iOS] Added support for Xcode 15.4, improved XCFramework generation script. 

# [2.37.6] - 2024-07-15
- [iOS] Re-enabled inline prediction and replaced the behaviour of reconciliate with a commmited text system for latin and common only characters, to allow for a better user experience with inline prediction.

# [2.37.5] - 2024-07-08
- [Rust] Fixed a crash that happened when entering a new line after setting through html a blockquote containing a single line.
- [Common] Updated dependencies.

# [2.37.4] - 2024-06-20
- [Android] Make `CustomMentionSpan` public.
- [Rust] Escape text passed to `ComposerModel::set_link_with_text` and `insert_mention*` methods.

# [2.37.3] - 2024-05-21
- [Android] Add `RichTextEditorState.setMarkdown(text)` function, which allows the usage of MD as the initial text of the rich text editor Compose wrapper.

# [2.37.2] - 2024-05-06
- [Android] Fix selection mismatch on Android 14+ after using word completion.

# [2.37.1] - 2024-05-03
- [Android] Return pure Markdown in the `RichTextEditorState.messageMarkdown` property instead of MD + HTML for mention pills.
- [Android] Fix selection indexes not being up to date when adding a punctuation mark using GBoard keyboard in Android < 13.

# [2.37.0] - 2024-04-12
- [Android] Discard mentions with no associated text.
- [iOS] iOS tests: Use xcresultparser instead of slather.

# [2.36.0] - 2024-04-03
- [Android] Fix text composition issues in Compose wrapper.
- [Android] Simplify `InputConnection` implementation, make `OnSelectionChangedListener` more precise.
- [iOS] Fix a couple of bugs detecting whether the content is empty or not.
- [iOS] Release to [matrix-org/matrix-rich-text-editor-swift](https://github.com/matrix-org/matrix-rich-text-editor-swift), attaching the binaries to GitHub releases.

# [2.35.0] - 2024-03-27
- [iOS] Fix deployment with Xcode 15.3.

# [2.34.0] - 2024-12-04
- [Rust] Bumped UniFFi to revision `789a9023b522562a95618443cee5a0d4f111c4c7`.
- [Android] reconcile UI text with editor text.
- [Android] fix English characters for some Chinese keyboards.

# [2.33.0] - 2024-12-04
- [iOS] Re-enabled auto-correction but disabled inline predictions on iOS 17.
- [iOS] Build workflow now uses Xcode 15.2.

# [2.32.0] - 2024-11-04
- [Android] Allow customising the initial Markdown value in `RichTextEditorState`.

# [2.31.0] - 2024-08-04
- [iOS] Temporarily disabled auto correction until we fix the predictive text issue.

# [2.30.0] - 2024-01-04
- [Rust] Bumped Rust version to `1.76`.
- [Rust] Bumped UniFFi to `v0.26.1`.
- [Rust] Bumped speculoos to `0.11.0`.
- [Android] Fix touch handling for links: the listener should only fire once per click and tapping on an empty space next to a link wrapping several lines should not fire it.

# [2.29.0] - 2024-02-08
### Changed
- [iOS] Improved the textView injection process in the view model.

# [2.28.0] - 2024-02-08
### Fixed
- [iOS] Fixed an issue that broke the text view if the SwiftUI view was completely re-rendered.

# [2.27.0] - 2024-02-01

### Fixed
- [Common] Remove NBSP chars in replace_text for empty paragraphs (#926)
- [Common] Fix delete word not replacing text (#833)

# [2.26.0] - 2024-01-17

### Fixed
- [iOS] Fixed an issue that made the RTE crash when ending dictation.

# [2.25.0] - 2024-01-12

### Fixed
- [iOS] Fixed a bug that prevented dictation to work properly on iOS 17 and above.

# [2.24.0] - 2024-01-05

### Added
- [Android] Add support for handling rich content from the keyboard (images, gifs, stickers, etc.).

### Fixed
- [Common] Fix deletion of several selected paragraphs, especially those with no content.
- [Android] Fix incorrect detection of NBSP chars as extra characters while parsing HTML.
- [Android] Destroy (as in free its memory) the current `ComposerModel` when it panics, before we replace it with a new instance.
- [Android] When updating the selection, make sure it's inside the bounds of the existing text.
- [Android] When creating the `SpannableString` while parsing the HTML, make sure every span is inside the bounds of its text.

# [2.23.0] - 2023-12-26

### Changed
- [Android] `EditorStyledTextView` and `EditorStyledText` now have an `onTextLayout` lambda parameter to publish its text layout results.
- [Android] Add `releaseOnDetach` param to `EditorStyledText` so it can be reused in lazy composables.

# [2.22.0] - 2023-12-06

### Fixed
- [Android] Fix for user suggestions not updating the RTE text properly sometimes.
- [Common] Fix for the editor crashing when adding a new line just after a mention.
- [iOS] Fix for shift+enter key not working properly and refactored the Key Commands handling.
- [Rust] Fix for markdown mode now correctly parsing multilined text.

# [2.21.0] - 2023-12-06

### Fixed

- [Android] Fix line height changes in `EditorStyledText`
- [Android] Fixes needed to get `EditorSyledTextView` to work with screenshot tests

### Changed

- [Android] Refactor text view test (#896)
- [Android] Allow specifying line height for all API versions (#897)
- [Android] Add typing callback (#900)

# [2.20.0] - 2023-11-29

### Fixed

- Android: fix link click detection in `EditorStyledTextView`, as previously it consumed the touch event even when no link or mention was clicked.
- Android: fix race condition when creating a new `MentionDetector` for `EditorStyledTextView`, which caused mentions to only be detected sometimes.
- Android: handle all spans with URLs in the gesture detector of `EditorStyledTextView`.

### Changed

- Android: added `TextStyle.lineHeight` property to customise line height on `EditorStyledTextView`.
- Android: bump JNA version to `v5.13.0`.

# [2.19.0] - 2023-11-28

### Fixed

- Android: fix a bug with auto capitalisation.
- Android: fix disappearing numbers with some keyboards.
- Android: fix using emojis inside composing regions.
- iOS: fix the position of the caret.
- iOS: fix double space to full stop conversion shortcut

### Changed
- Common: remove uniffi fork from dependencies

# [2.18.0] - 2023-11-17

### Fixed

- iOS: fixed a bug that made the system speech to text functionality suddenly stop after a few seconds.

### Added

- Android: Improved reliability of UI tests running on CI emulator.
- Android: CI takes a screenshot when instrumentation tests fail.

# [2.17.0] - 2023-11-16

### Fixed

- Android: removed workaround for excessive recompositions from the Compose components.
- Android: fix indexes not working properly for mentions.
- Android: add a workaround to always get a valid `lineCount` in the Composables.
- Common: prevent trailing NBSP characters from being removed when parsing. They should only be removed if they are the whole content of a container node.

### Added

- Common: added support for parsing mentions using a different prefix from the default `https://matrix.to` one.

# [2.16.0] - 2023-11-09

### Fixed

- Android: [API breaking change] replaced the existing `mentionDisplayHandler` parameter in both `RichTextEditor` and `EditorStyledText` with a couple of `resolveMentionDisplay` and `resolveRoomMentionDisplay` lambdas.
- Android: fixed an issue that caused unnecessary recompositions in the Compose components.
- Android: `EditorEditText` was being leaked by the ViewModel on activity recreations, also causing the right styles to not be applied properly.
- Android: change several default theme colors.
- Android: add `MentionsState` to get the intentional mentions in the text.
- Android: make `LinkSpan`, `PillSpan` and `CustomMentionSpan` clickable in the TextView component.

### Added

- Common: Added `MentionDetector` to the FFI bindings so we can use the same method for detecting mentions in all platforms.
- Android: Added `replaceSuggestion` and `insertMentionAtSuggestion` methods to the Compose implementation of the RTE composer.

# [2.15.0] - 2023-10-30

### Fixed

- iOS: Use message content APIs (#842)
- Android: Fix crash due to out of bounds error (#845)
- Android: Handle and track more internal exceptions (#846)

### Added

- iOS: Add "shouldReplaceText" API to ignore keyboard input (#857)
- iOS: Add getMentionsState API (#850)
- Common: get_mentions_state() implemented on the Rust side (#849)
- iOS: Expose the set at room mention function (#843)
- iOS: Enhanced `MentionDisplayHelper` (#840) 
- Android: Add rich text styled view (#837)

### Changed

- Common: Replace paragraphs with line breaks in message output (#834)

# [2.14.1] - 2023-10-04

### Fixed

- iOS: Fix background drawing when clearing quote / code block (#829)
- iOS: Fix reconciliate breaking with leading whitespaces (#831)

# [2.14.0] - 2023-09-27

### Added

- Android: [API breaking change] Add support for subcomposition and sharing state (#822)

### Fixed

- Common: Fix crash on pressing enter after emoji (#824)
- Android Fix selection state restoration (#825)
- iOS: Fix pending format inconsistencies (#823)

# [2.13.0] - 2023-09-20

### Added

- Android: allow font size customisation in compose (#819)

### Fixed

- Fix unwanted extra paragraphs after HTML parsing (#820)

# [2.12.0] - 2023-09-18

### Added

- Android: Add customisation options for links and code (#813)

### Changed

- Android: [API breaking change] Simplifiy API for setting code background (#813)

### Fixed

- iOS: Fix code block and inline code fonts (#814)

# [2.11.0] - 2023-09-15

- iOS: Add API to change the text view placeholder color (#810)

# [2.10.2] - 2023-09-14

### Fixed

- Android: Fix missing link action updates in compose library (#804)

# [2.10.1] - 2023-09-13

No changes except to publishing workflow.

# [2.10.0] - 2023-09-12

### Added

- Android: Add link support to compose library (#801)

### Changed

- Common: Remove unnecessary escape characters in markdown output (#800)

### Fixed

- Android: Fix performance issue caused by excessive recomposition (#803)

# [2.9.0] - 2023-09-07

### Added

- Android: Add fake/test mode for compose state

### Fixed

- Android: Fix compose error callback not getting called

# [2.8.0] - 2023-08-31

### Added

- iOS: Setting to override library's log level

### Changed

- iOS: Removed `WysiwygAction` enum in favour of bindings `ComposerAction`

# [2.7.0] - 2023-08-29

### Added

- Android: Compose error callback

# [2.6.3] - 2023-08-25

### Changed

- iOS: Revert library back to be exposed as static

### Fixed

- Android: Publishing configuration for Compose library

# [2.6.2] - 2023-08-23

### Changed

- iOS: Library is now exposed as dynamic
- iOS: Now uses sizeThatFits to handle text view height on iOS 16.0 and higher

### Fixed

- iOS: Composer text view not being resized after backspacing multiline content

# [2.6.1] - 2023-08-21

### Changed

- iOS: Reworked placeholder text implementation

### Fixed

- iOS: Ideal composer height not being initially published

# [2.6.0] - 2023-08-18

### Added

- Android: More compose APIs (focus, line count, text and cursor style)
- Android: Compose preview support

### Changed

- Android: Remove default padding on the composable editor 

### Fixed

- Android: Compatibility with apps compiled with JDK 11 
- iOS: Reduce library size

# [2.5.0] - 2023-08-11

### Added

- Android: Support for a Compose version of the library
- iOS: Support for overriding behaviour for `enter` and `shift` + `enter` keyboard inputs

### Changed

- Common: Uniffi bindings build now uses library path and macros instead of UDL file
- iOS: Improved support for delegating items pasting to the hosting application 

### Fixed

- iOS: Composer focused state being published within view updates

# [2.4.1] - 2023-07-26

### Fixed

- Web: CTRL+Backspace behaviour on Windows

# [2.4.0] - 2023-07-26

### Changed

- Common: API for getting Markdown content now depends on context (composer usage or final output)
- Common: Set Rust version to 1.71 and Uniffi version to 0.24.1

### Fixed

- Common: Fix an inconsistency with code blocks when parsing back and forth between Markdown & HTML
- iOS: Fixed XCFramework hierarchy to avoid modulemap files conflicting with other C or Rust libraries
- iOS: Fixed an issue with Uniffi C symbols conflicting with other Rust libraries in a shared namespace

# [2.3.1] - 2023-06-30

### Added

- Web: Listener to handle composition events

### Changed

- Web: Enable choice of outputting rich text in message or composer format
- Web: Initialising behaviour to support initialising a composer containing mentions
- Web: Improved consistency when converting plain text composer output into a Rust model

### Fixed

- Common: Newline handng when parsing block quotes from markdown
- Common: Issue where links were being split incorrectly inside list items
- Web: Running tests with coverage no longer hangs in CI

# [2.3.0] - 2023-06-16

### Added

- Common: Functions for inserting mentions and at-room mentions
- Common: Function to output html formatted in accordance with the matrix spec
- Common: A new crate to represent matrix mentions
- Web: Expose html message output
- Web: Expose new function for adding at-room mentions

### Changed

- Web: Types for the `attributes` argument when inserting a mention

# [2.2.2] - 2023-06-08

### Added

- Common: New `MentionNode` to represent mentions in the Rust model
- Common: Functions to output html specifically formatted for sending as a message
- Common: Documentation for the example format selection writer
- Android: Allow pasting of images
- Web: New listener for `beforeinput` events

### Changed

- iOS: Run iOS coverage for unit and ui tests separately
- Android: Now uses the html output in the message format

### Fixed

- iOS: Incorrect character placement after multiple newlines

# [2.2.1] - 2023-05-23

### Fixed

- Common: Disable invariant assertions by default

# [2.2.0] - 2023-05-18

### Changed

- Android: [API breaking change] Add support for mentions
- Common: Read attributes from markdown for mentions

# [2.1.0] - 2023-05-02

### Changed

- Common: [API breaking change] Add extra HTML attributes parameter to all link creation APIs
- Common: [API breaking change] Rename `link` parameter to `url` wherever it's relevant
- iOS: Expose `set_content_from_markdown` to the hosting application

### Fixed

- iOS: Disable autocorrection if current input is a command
- iOS: Fix an issue with code blocks NBSP placeholder not being replaced on display

# [2.0.0] - 2023-04-04

### Changed

- Common: [API breaking change] Change signature of `set_link_suggestion` function to take an attributes argument

### Fixed

- Web: Fix selection issue when formatted text is inside paragraph adjacent to a list
- Web: Prevent memory issues when using `replace_text_suggestion` function

# [1.4.1] - 2023-03-28

### Fixed

- Common: Fix behaviour when deleting after the first of multiple mentions
- Web: Fix memory managment issue for mentions

# [1.4.0] - 2023-03-21

### Changed

- Common: Enable link attribute parsing
- Common: Link button disabling logic exists for immutable links
- Common: Backspace/delete behaviour now handles immutable links
- iOS: Swift-tools-version bump from 5.6 to 5.7
- Web: Selection logic now handles immutable links

# [1.3.0] - 2023-03-17

### Changed

- Common: Autocompleted links now have extra attributes when representing a custom link type
- Common: `replace_text` now defaults to writing outside of link label when on edge
- iOS: Content of the `UITextView` for the plain text mode is now published
- iOS: Updated `PermalinkReplacer` API to allow inserting/removing custom objetcs in the attributed string in plain text mode

### Fixed

- iOS: Fix link button availability around replaced links
- iOS: Fix index computation around multiple replaced links

# [1.2.2] - 2023-03-08

### Fixed

- iOS: Fix attachment views sometimes not clearing when switching to plain text mode
- iOS: Fix publishing height updates of the composer in plain text mode
- iOS: Remove `select` Rust API call in plain text mode

# [1.2.1] - 2023-03-07

### Changed

- Web: Change type of suggestion exported by hook

# [1.2.0] - 2023-03-06

### Added

- Common: Detection for at/hash/slash pattern in text
- Common: API for replacing a detected pattern with link/text
- Web & iOS: Support for inserting mentions/commands

### Changed

- Common: Set content with HTML/Markdown now returns a `Result`

### Fixed

- Common: Fixed creating a list from a selection containing quotes/code blocks
- Common: Fix leading whitespaces ignored by HTML parsing
- iOS: Fix trailing whitespace ignored after a link

# [1.1.1] - 2023-02-14

### Fixed

- iOS: Fix selection/cursor position after lists

# [1.1.0] - 2023-02-10

### Added

- Common: Add utility to get content as plain text

### Changed

- Web & iOS: Hide indent/unindent buttons outside of lists in example app
- Android: Update NDK version to r25c (LTS)
- Android: Allow subclassing `EditorStyledTextView`

### Fixed

- Common: Recover from Rust model panics
- Web: Don't rerender when testRef is not set
- iOS: Fix pending formats not reapplied in new list item after `enter`
- iOS: Improve composer content vertical spacing

# [1.0.0] - 2023-02-07

### Added

- Common: Add transaction functionality to Dom
- Web: Add cmd-e shortcut for inline code
- Web: Add soft delete line backwards implementation

### Changed

- Common: [API breaking change] Rename UnIndent as Unindent
- iOS: Set BlockStyle parameters as public
- iOS: Update padding and style of quotes and code block

### Fixed

- Common: Fix code block HTML rendering
- Common: Fix links creation panicking when empty nodes are within the range
- Common: Fix enter behaviour on empty list item with formatting
- Common: Fix menu state in empty formatted paragraphs/list items
- Common: Fix parent `li` paragraph hierarchy after backspacing an indented `li`
- Web: Handle insertReplacementText input event
- Web: Fix cursor positioning issue when splitting a node into paragraphs
- Web: Fix Ctrl/cmd-a behaviour
- Android: Fix test coverage configuration
- Android: Fix rendering of code with trailing new line
- iOS: Fix line indent after quotes & code blocks

# [0.23.0] - 2023-01-31

### Added

- Common: Add utility to build the array of ancestor handles from a handle
- Web: Add indent and unindent

### Fixed

- Android: Fix crashes in `EditorStyledTextView`

# [0.22.0] - 2023-01-30

### Added

- Common: Add `<pre>`, `<code>` and `<p>` handling to html parsers
- Common: Add clippy to CI
- Android: Add configuration options for code size in inline code and code blocks

### Changed

- Common: Paragraphs are now contained inside `<p>` tags
- Common: Further improve link behaviour when spanning block nodes
- Common: Leading and trailing empty paragraphs inside `<pre>` tags will now contain `&nbsp;` HTML entity
- iOS: Use ZWSP in place of `&nbsp;` in some places to allow block rendering
- iOS: Improve code utilities for writing Swift tests

### Fixed

- Common: Apply clippy fixes to all files
- Common: Fix parsing of HTML tags containing only `&nbsp;` HTML entity
- Common: Fix button state behaviour when creating nested lists
- Common: Fix button state behaviour when starting inline code

# [0.21.0] - 2023-01-26

### Added

- Web: Add editor HTMLElement parameter to inputEventProcessor

### Changed

- Common: Improve link behaviour when the selection contains block or structure nodes
- iOS: Integrate DTCoreText

# [0.20.0] - 2023-01-19

### Added

- Web: Add quotes

### Changed

- Common: Disable lists, inline code and links inside code blocks

# [0.19.0] - 2023-01-17

### Changed

- Common: Blank selections allow the user to create links with text

### Fixed

- Web: Fix isWysiwygReady

# [0.18.0] - 2023-01-16

### Added

- Web: Add code blocks
- iOS: Add live document tree to example app
- Android: Add code block styling
- Android: Add helpers for inline code styling

### Changed

- Common: Default to https:// or mailto: for links without schemes

# [0.17.0] - 2023-01-12

### Added

- Common: Fixed a bug that created links with generic empty nodes in some cases

# [0.16.0] - 2023-01-11

### Added

- Web: InputEventProcessor is called on keyDown event
- iOS: Add Quote/code blocks integration

# [0.15.0] - 2023-01-10

### Added

- Common: Add parsing of code blocks and quotes to the WASM HTML parser
- iOS: Add attributed string to html mapping for lists with ZWSP
- Android: Add inline code formatting
- Android: Make bullet size configurable
- Android: Add code blocks & quotes to the example app

### Fixed

- Common: Fix creating lists with a leading line break
- Common: Fix selection state after toggling off list

# [0.14.0] - 2023-01-05

### Added

- Web: Add removeLinks and getLinks method
- iOS/Android: Code block & quote bindings

### Fixed

- Common: Code block & quote fixes
- Common: List behavior improvements
- Android: Fix special character handling

# [0.13.0] - 2022-12-22

### Added

- Common: Add TS types to exports for new conversion functions
- Common: Add ability for links to wrap existing elements

### Changed

- Common: Change RELEASE.md to reflect changes to iOS build script
- iOS: Change iOS build script to increase automation

### Fixed

- Common: Fix edge case performance in `model.state.dom.insert_parent`

# [0.12.0] - 2022-12-20

### Added

- Common: Add support for ZWSP in code blocks
- Common: Add conversion functions to convert between rich and plain text
- Common: Add `insert_parent` DOM method
- iOS: Show background colour of inline code in the composer

### Changed

- Common: Change uses of ZWSP to be new DomNode type
- Common: Change `CharType` enum to remove ZWSP
- iOS: Change tapping on a link to highlight the link, not open it

### Fixed

- Common: Fix disabling inline code then typing not reordering nodes

# [0.11.0] - 2022-12-15

### Added

- Web: Add link APIs
- Common: Add DomLocationPosition helper
- Common: Add code block support

### Fixed

- Common: Fix issues with line breaks at the start and end of code blocks
- Common: Fix for a bug where attributed string keeps format style for links and inline code after deletion
- iOS: Fix link color

# [0.10.0] - 2022-12-08

### Added

- Common: Add extension to character to create ZWSP easily
- Common: Add 'remove word' functionality
- Common: Extended selection/range for DOM
- Common: Add DOM handle depth helper
- iOS: Add links implementation in the example app for testing
- Android: Add link APIs

### Changed

- Common: Change link interface to use string

### Fixed

- Common: Make `find_range_by` work symmetrically
- Common: Lookup and join ancestor on join nodes

# [0.9.0] - 2022-12-06

### Added

- Web: Add `insertText` method in available functions
- Common: Add extension to character to create ZWSP easily

### Changed

Common & iOS: Remove the Rust & Uniffi version duplication from publish.yml
Common: Change Rust panic behaviour from abort to unwind

### Fixed

- Android: Fix crashes in ElementX on config changes
- Common: Fix dom location length with nested tags
- Common: Fix crashes when `replace_text` is called

# [0.8.0] - 2022-11-28

### Added

- Common: inline code formatting now works for selections with several nodes, also, formatting states are disabled when inline code is selected.
- Common: `DomLocation` now has `kind()` method to make finding nodes of some kind easier in ranges.
- Common: `DomIterator` can now be used in a sub-tree of the DOM (from an internal node instead of the root one).

### Fixed:

- Common: when replacing text at the end of a link node, the new text is added to the next text node if exists instead of the link node.
- Android: fixed formatting disappearing from the last typed word when adding a whitespace.
- Web: Handle insertCompositionText as if it were insertText (hopefully fixing accented characters in Element Desktop)

# [0.7.0] - 2022-11-21

### Changed

- Common: update to Rust 1.65 and uniffi-rs 0.21.0.
- Common: internal refactor.
- Common: MenuState updates are now always returned when we change some content.
- Common: new API for retrieving parent nodes: `Dom.parent(&child_handle)` and `Dom.parent_mut(&child_handle)`.

### Added

- Common: `Dom` is now iterable.
- Common: links can now be added to several nodes and updated.
- Android: add Markdown support.
- Android: add `RustErrorCollector` to be able to collect and treat any Rust errors manually in the integrating clients.
- Web: added debug mode.

### Fixed

- Android: text input is now diffed so we don't rely on composition, as it sometimes broke formatting.
- iOS: Voice dictation should work now.

# [0.6.0] - 2022-11-11

### Changed

- Common: MenuState updates now contain a single Map/Dictionary with an entry for each possible action, being either `Enabled`, `Disabled` or `Reversed`.

# [0.5.0] - 2022-11-11

### Added

- Common: initial Markdown support.
- Common: added get/set methods for Markdown text (`set_content_from_markdown`, `get_content_as_markdown`). Also added a getter for HTML contents (`get_content_as_html`).
- iOS: added plain text mode with Markdown support.
- iOS: expose `maxExpandedHeight` and `maxCompressedHeight` properties in `WysiwygComposerViewModel`.
- Web: added `prettier` config to `eslint`.

### Fixed

- Common: prevent crash when deleting an emoji or other complex grapheme.
- Common: fix html5ever output when a text node contains escaped HTML entities.
- Android: fixed `TextWatcher`s being called with an empty String for every change in the composer.
- Android: fixed back system key being intercepted by the editor, preventing back navigation.
- iOS: fixed bold + italic formatting not being correctly rendered on iOS 14-15.
- iOS: fixed bug when deleting whole words with long press on backspace.
- iOS: fixed missing keystrokes when the user typed very fast.
- iOS: fixed the editor contents being cleared when plain text mode was enabled.

### Changed

- Common: `replace_all_html` is now `set_content_from_html`.
- Web: use native `DOMParser` instead of `html5ever` to parse HTML. This should decrease the WASM binary size.
- Web: reduced WASM binary size by 76%.

# [0.4.0] - 2022-10-26

### Added

- Android: Add plain text APIs

### Fixed

- Android: Fix issue with hardware backspace key

# [0.3.2] - 2022-10-24

### Added

- Web: `useWysiwyg` hook can be initialized with a content

### Fixed

- Web: Fix losing selection after Ctrl-a Ctrl-b

## [0.3.1] - 2022-10-19

### Added

- iOS: Show placeholder text

### Fixed

- Web: allow instantiating multiple composers
- Android: code improvements

## [0.3.0] - 2022-10-19

### Added

- Web: Allow pressing Enter to send message

### Fixed

- iOS: use correct fonts

## [0.2.1] - 2022-10-17

### Added

- iOS: add support for focused state.
- Android: handle cut & paste events properly.

### Changed

- Android: only crash on composer exceptions when using DEBUG mode.

### Fixed

- iOS: use right cursor color and fix blinking issue when replacing text.
- Fix inserting characters or new lines before a new line at index 0.
- Android: fix formatting not being replaced at index 0 when using hardware
  keyboard.

### Removed

- iOS: remove unneeded UIKit integration code.

## [0.2.0] - 2022-10-13

### Added

- Web: Add formatting states
- Web: Remove onChange handler and return the content instead

## [0.1.0] - 2022-10-11

### Added

- Web: support cut and paste
- Document release process
- NPM releases via a manual github workflow

## [0.0.2] - 2022-10-10

### Added

- Improve React integration

## [0.0.1] - 2022-10-10

### Added

- First attempt at packaging for NPM
- Basic text editing (newlines, bold, italic etc. formatting)
- Draft support for lists
- Draft support for links
