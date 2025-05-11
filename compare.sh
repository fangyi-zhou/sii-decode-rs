#!/bin/bash

set -euo pipefail

name=$1
expected_filename=${name/.sii/_backup.sii}
actual_filename=${name/.sii/_actual.sii}
diff_filename=${name/.sii/.diff}

cargo run "$name" > "$actual_filename"
diff -bB "$expected_filename" "$actual_filename" > "$diff_filename"
