#!/usr/bin/env bash
# Regenerate the embedded icon alpha masks from the source SVGs.
#
# The Rust code embeds `assets/icons/rgba/<name>.a.gray`: 48x48 8-bit coverage
# masks (white-on-transparent rendered to alpha). Edit/replace the SVGs in
# `assets/icons/`, then run this script. Requires `rsvg-convert` + ImageMagick.
set -euo pipefail

cd "$(dirname "$0")/.."
out="assets/icons/rgba"
mkdir -p "$out"

for svg in assets/icons/*.svg; do
  name="$(basename "$svg" .svg)"
  # Render at 48x48, keep only the alpha (coverage) channel as 8-bit gray.
  magick -background none "$svg" -resize 48x48 -alpha extract -colorspace gray -depth 8 \
    "$out/$name.a.gray"
  echo "wrote $out/$name.a.gray"
done
