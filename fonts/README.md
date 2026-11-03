# Bundled fonts

These font families are embedded into `lgl` at build time (see
`crates/lgl/build.rs`), compressed, so documents can use them by name wherever
lgl runs, without installed fonts. A font is decompressed only when a document
uses it. Typst's own fonts (Libertinus Serif, New Computer Modern, New Computer
Modern Math, DejaVu Sans Mono) are available as well.

The files come unmodified from [google/fonts](https://github.com/google/fonts),
at commit `2fe1e8c6b137fb13041a7cfca6fdcd8a910ea4dc` for the families added on
2026-11-03. Every family is licensed under the SIL Open Font License 1.1, with
its licence in `OFL.txt` next to the font files, except Special Elite, which is
licensed under the Apache License 2.0 (`specialelite/LICENSE.txt`). Variable
fonts provide every weight; the other families ship static files.

`index.json` holds the metadata of every bundled file so that lgl can start
without decompressing the fonts. After adding or changing a font, run the
tests: `bundled_font_index_is_current` writes a fresh index to
`target/font-index.json` (CI uploads it as the `font-index` artifact); copy it
over `fonts/index.json`.

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
- DM Sans (`dmsans/`)
- Figtree (`figtree/`)
- Outfit (`outfit/`)
- Space Grotesk (`spacegrotesk/`)
- Red Hat Text (`redhattext/`)
- Red Hat Display (`redhatdisplay/`)
- Public Sans (`publicsans/`)
- Libre Franklin (`librefranklin/`)
- Cabin (`cabin/`)
- Jost (`jost/`)
- Nunito (`nunito/`)
- Quicksand (`quicksand/`)
- Belleza (`belleza/`)

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
- Newsreader (`newsreader/`)
- Libre Caslon Text (`librecaslontext/`)
- Spectral (`spectral/`, Regular, Italic, Bold and Bold Italic)
- Alegreya (`alegreya/`)
- Bodoni Moda (`bodonimoda/`)
- STIX Two Text (`stixtwotext/`)

## Slab serif

- Bitter (`bitter/`)
- Arvo (`arvo/`)

## Metric-compatible with common office fonts

Same character widths, so text breaks into the same lines.

- Tinos (`tinos/`): Times New Roman
- Arimo (`arimo/`): Arial, Helvetica
- Cousine (`cousine/`): Courier New
- Carlito (`carlito/`): Calibri
- Caladea (`caladea/`): Cambria
- Gelasio (`gelasio/`): Georgia

## Readability

- Atkinson Hyperlegible Next (`atkinsonhyperlegiblenext/`)
- Lexend (`lexend/`)
- Andika (`andika/`): early reading
- Edu NSW ACT Foundation (`edunswactfoundation/`): handwriting worksheets

## Display (headlines only)

- Oswald (`oswald/`)
- Bebas Neue (`bebasneue/`, capitals only)
- Anton (`anton/`)
- Abril Fatface (`abrilfatface/`)
- DM Serif Display (`dmserifdisplay/`)
- Archivo Black (`archivoblack/`; Typst names it "Archivo", weight 900)
- Bricolage Grotesque (`bricolagegrotesque/`)
- Alfa Slab One (`alfaslabone/`)
- Saira Stencil One (`sairastencilone/`)
- Orbitron (`orbitron/`)
- Poiret One (`poiretone/`)
- Fredericka the Great (`frederickathegreat/`)
- Cinzel (`cinzel/`): classical capitals
- UnifrakturCook (`unifrakturcook/`): blackletter

## Script and handwriting

- Dancing Script (`dancingscript/`)
- Great Vibes (`greatvibes/`)
- Pinyon Script (`pinyonscript/`)
- Caveat (`caveat/`)
- Patrick Hand (`patrickhand/`)
- Comic Neue (`comicneue/`)

## Monospace

- JetBrains Mono (`jetbrainsmono/`)
- Fira Code (`firacode/`)
- Source Code Pro (`sourcecodepro/`)
- IBM Plex Mono (`ibmplexmono/`)
- Roboto Mono (`robotomono/`)
- Courier Prime (`courierprime/`): screenplays
- Special Elite (`specialelite/`): typewriter

## Math

- Libertinus Math (`libertinusmath/`)
- STIX Two Math (`stixtwomath/`)
- Noto Sans Math (`notosansmath/`)

## Symbols, emoji, music and barcodes

- Noto Sans Symbols (`notosanssymbols/`)
- Noto Sans Symbols 2 (`notosanssymbols2/`)
- Noto Emoji (`notoemoji/`, monochrome)
- Noto Music (`notomusic/`)
- Libre Barcode 39 (`librebarcode39/`)
- Libre Barcode 128 (`librebarcode128/`)

Manrope and Fira Code have no italic styles.

Variable fonts with an optical-size axis (DM Sans, Newsreader, Bricolage
Grotesque) name their default instance after its size, like "DM Sans 9pt"; lgl
lists them under their plain names, and Typst picks the optical size from the
text size.
