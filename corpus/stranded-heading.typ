// expect: L014.stranded-heading
// expect: S004.disabled-protection
#set page(paper: "a5", margin: 2cm)
#show heading: set block(sticky: false)
#block(width: 100%, height: 440pt, fill: luma(240))
= Results
#lorem(60)
