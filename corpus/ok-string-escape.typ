// expect-none
#set page(paper: "a4", margin: 2cm)
#set text(size: 10pt)
#let price = "$84.6 M"
#let path = "C:\Reports\q3"
Revenue reached #price, and the costs are filed under #raw("\$cost") and #path.
#"Thirty $ and \"quoted\" words".replace(regex("\$"), "USD ")

#lorem(60)
