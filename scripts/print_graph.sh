#!/usr/bin/env bash
set -euo pipefail

ccomps -x "$1" | dot -Grankdir=TB | gvpack -array3 | neato -Tpng -n2 -o "$2"
