// expect: L006.overlap
// expect: S002.absolute-placement
#set page(paper: "a4", margin: 2cm)
#place(top + left, dy: 2pt)[#text(14pt)[Placed overlay text]]
This paragraph starts at the very top of the page and collides with the placed text.
