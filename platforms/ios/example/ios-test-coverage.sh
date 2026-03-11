#!/bin/bash

set -eo pipefail

xcodebuild \
  -project Wysiwyg.xcodeproj \
  -scheme WysiwygComposerTests \
  -sdk iphonesimulator \
  -destination 'platform=iOS Simulator,name=iPhone 17,OS=26.1' \
  -derivedDataPath ./DerivedData \
  -resultBundlePath tests.xcresult \
  -enableCodeCoverage YES \
  test
