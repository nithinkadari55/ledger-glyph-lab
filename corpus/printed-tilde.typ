// expect: L053.markup-in-text
#set page(paper: "a5", margin: 1.5cm)
#set text(size: 10pt)
#let feature(title, body) = block(below: 1em)[*#title* \ #body]
#feature("Quiet sleep mode", "Runs at 24~dB(A), quieter than a rustle of leaves.")
#feature("Design", "Drawn by S.~Petrova and built in Leeds.")
