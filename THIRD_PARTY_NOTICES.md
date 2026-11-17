# Third-party notices

This project bundles a small number of icon assets. They remain under their
original licenses, reproduced below. No modifications to the license terms are
made; only the icon geometry/color was adapted (recolored to white, rasterized
to an alpha coverage mask) for use in the UI.

---

## Material Symbols (Material Icons)

Icons under `assets/icons/` (and the derived 48×48 alpha masks in
`assets/icons/rgba/`) are from **Material Symbols** by Google, obtained via
<https://fonts.google.com/icons>.

- Files: `menu`, `pause`, `play_arrow`, `undo`, `redo`, `edit`, `ink_eraser`,
  `timer`, `lightbulb`, `celebration`, `filter_9_plus`.
- License: Apache License 2.0.

```
Copyright 2024 Google LLC

Licensed under the Apache License, Version 2.0 (the "License");
you may not use these files except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```

A full copy of the Apache License 2.0 is available at
<https://www.apache.org/licenses/LICENSE-2.0>.

The Material Symbols repository is at
<https://github.com/google/material-design-icons> (the `symbols` set;
`google/material-design-icons` is also Apache-2.0 licensed).

---

## Rebuilding the embedded masks

The Rust front-end embeds `assets/icons/rgba/<name>.a.gray` (48×48 8-bit alpha
coverage masks) via `include_bytes!`. To regenerate them from the source SVGs:

```sh
bash scripts/generate-icons.sh
```

Requires `rsvg-convert` (librsvg) and ImageMagick (`magick`). The original
downloaded SVGs are the source of truth.
