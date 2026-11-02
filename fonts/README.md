# Bundled fonts

These font families are embedded into `lgl` at build time (see
`crates/lgl/build.rs`), so documents can use them by name wherever lgl runs,
without installed fonts. Typst's own fonts (Libertinus Serif, New Computer
Modern, New Computer Modern Math, DejaVu Sans Mono) are available as well.

Every family is licensed under the SIL Open Font License 1.1; its licence is in
`OFL.txt` next to the font files. The files come unmodified from
[google/fonts](https://github.com/google/fonts). Variable fonts provide every
weight; a few families ship static Regular, Italic, Bold and Bold Italic files
(some also Medium and SemiBold).

## Sans serif

- Inter (`inter/`)
- Source Sans 3 (`sourcesans3/`)
- IBM Plex Sans (`ibmplexsans/`)
- Open Sans (`opensans/`)
- Lato (`lato/`)
- Montserrat (`montserrat/`)
- Roboto (`roboto/`)
- Work Sans (`worksans/`)
- Fira Sans (`firasans/`)
- Manrope (`manrope/`)
- Plus Jakarta Sans (`plusjakartasans/`)
- Raleway (`raleway/`)
- Poppins (`poppins/`)
- Noto Sans (`notosans/`)

## Serif

- Source Serif 4 (`sourceserif4/`)
- Lora (`lora/`)
- Playfair Display (`playfairdisplay/`)
- EB Garamond (`ebgaramond/`)
- Crimson Pro (`crimsonpro/`)
- Libre Baskerville (`librebaskerville/`)
- Noto Serif (`notoserif/`)
- IBM Plex Serif (`ibmplexserif/`)
- PT Serif (`ptserif/`)
- Literata (`literata/`)
- Fraunces (`fraunces/`)

## Monospace

- JetBrains Mono (`jetbrainsmono/`)
- Fira Code (`firacode/`)
- Source Code Pro (`sourcecodepro/`)
- IBM Plex Mono (`ibmplexmono/`)
- Roboto Mono (`robotomono/`)

Manrope and Fira Code have no italic styles.
