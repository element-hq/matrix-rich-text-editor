#!/bin/bash

set -eo pipefail

xcodebuild \
  -project Wysiwyg.xcodeproj \
  -scheme Wysiwyg \
  -sdk iphonesimulator \
  -destination 'platform=iOS Simulator,name=Snapshots,OS=26.1' \
  -derivedDataPath ./DerivedData \
  -resultBundlePath ui-tests.xcresult \
  -enableCodeCoverage YES \
  test
  
