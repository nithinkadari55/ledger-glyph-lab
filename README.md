# ledger-glyph-lab

Compiles Typst documents and reports layout defects as JSON lines, so an
automated author can repair its source without looking at rendered pages.

## Layout

- `vendor/typst`: Typst at commit `84cca406aafe7f830ed3f37f1cc6f0c8b6b65a03`
  (Apache-2.0). See `vendor/typst/LICENSE` and `vendor/typst/NOTICE`.
- `crates/lgl`: compiler driver, geometry extraction, rules, CLI.
- `corpus`: regression documents. Each starts with `// expect: <rule>` lines or
  `// expect-none`.

## Usage

```sh
cargo run --release -p lgl -- check doc.typ --pdf doc.pdf --geometry doc.json
```

Each stdout line is one diagnostic:

```json
{"v":1,"rule":"L001.page-overflow","severity":"error","message":"content extends 412.3pt beyond the right edge of page 1","loc":{"file":"doc.typ","line":3,"col":1,"end_line":3,"end_col":42},"elem":"box","page":1,"measured":412.3,"limit":0.5,"unit":"pt","hints":["..."]}
```

Exit code: `0` no errors, `1` errors found, `2` invalid invocation.

## License

Code in `crates/` is dual-licensed under MIT or Apache-2.0.
