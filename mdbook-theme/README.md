# mdbook theme (source for CI)

This directory is the **tracked** copy of the mdbook theme. `docsbookgen` copies it into `docs/mdbook/theme/` before running `mdbook build`, so CI has `mermaid.js` and `custom.css` even when `docs/` is gitignored.

Edit files here; they are deployed with the book. Local `docs/mdbook/theme/` is overwritten at build time.
