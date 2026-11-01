// expect: L028.footnote-on-other-page
#set page(paper: "a5", margin: 2cm)
#block(width: 100%, height: 455pt, fill: luma(240))
Last line on the page.#footnote[#lorem(80)]
