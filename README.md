# Matrix Rich Text Editor
[![codecov](https://codecov.io/gh/element-hq/matrix-rich-text-editor/branch/main/graph/badge.svg?token=UFBR3KtPdg)](https://codecov.io/gh/matrix-org/matrix-rich-text-editor)
![GitHub](https://img.shields.io/github/license/element-hq/matrix-rich-text-editor)

A cross-platform rich text editor intended for use in Matrix clients including
the Element clients.

Works on Web, Android and iOS using a cross-platform core written in Rust,
and platform-specific wrappers.

__Important note:__ This project is still in an early stage. Minor versions could bring 
breaking API changes, see [CHANGELOG.md](CHANGELOG.md) for details.
Bugs and crashes may occur, please report them [here](https://github.com/element-hq/matrix-rich-text-editor/issues/new).

## Live demo

Try it out at
[element-hq.github.io/matrix-rich-text-editor](https://element-hq.github.io/matrix-rich-text-editor/).

## Building the code

Get the prerequisites for each platform by reading the READMEs for them:

* WASM/JavaScript:
  [bindings/wysiwyg-wasm/README.md](bindings/wysiwyg-wasm/README.md)

* Android/Kotlin or iOS/Swift:
  [bindings/wysiwyg-ffi/README.md](bindings/wysiwyg-ffi/README.md)

Now, to build all the bindings, try:

```bash
make
```

To build for a single platform, or to learn more, see the individual README
files above.

## Release the code

See [RELEASE.md](RELEASE.md).

## More info

For more detailed explanations and examples of platform-specific code to use
Rust bindings like those generated here, see
[Building cross-platform Rust for Web, Android and iOS – a minimal example](https://www.artificialworlds.net/blog/2022/07/06/building-cross-platform-rust-for-web-android-and-ios-a-minimal-example/).

## See also

* The [Browser Selections Inventory](https://gitlab.com/andybalaam/browser-selections)
  - used while writing tests, to persuade the browser to select text in the
  same way as if it had been done manually.

## Copyright & License

Copyright (c) 2022-2024 The Matrix.org Foundation C.I.C.

Copyright (c) 2024-2025 New Vector Ltd

This software is multi licensed by New Vector Ltd (Element). It can be used either:

(1) for free under the terms of the GNU Affero General Public License (as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version); OR

(2) under the terms of a paid-for Element Commercial License agreement between you and Element (the terms of which may vary depending on what you and Element have agreed to). Unless required by applicable law or agreed to in writing, software distributed under the Licenses is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the Licenses for the specific language governing permissions and limitations under the Licenses.