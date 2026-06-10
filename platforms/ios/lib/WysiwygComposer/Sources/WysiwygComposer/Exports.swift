//
// Copyright 2024 New Vector Ltd.
// Copyright 2023 The Matrix.org Foundation C.I.C
//
// SPDX-License-Identifier: AGPL-3.0-only OR LicenseRef-Element-Commercial
// Please see LICENSE in the repository root for full details.
//

// The UniFFI-generated bindings live in their own target so they can opt out of default
// MainActor isolation. Re-export them so that `import WysiwygComposer` continues to expose the
// composer FFI types (`ComposerModel`, `ComposerUpdate`, …) as before the target split.
@_exported import WysiwygComposerBindings
