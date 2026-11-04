// expect: L068.drop-cap-hole
#import "@preview/droplet:0.3.1": dropcap
#set page(paper: "a4", margin: 2cm)
#set text(size: 10pt)
#set par(justify: true)
#dropcap(height: 3, gap: 6pt)[T][#smallcaps[he first thing you notice is the quiet.]] Eleven storeys above a busy avenue the traffic has thinned to a murmur, and in its place there is the rustle of kale leaves, the hum of a bee working through borage and the soft clink of a trowel against a terracotta pot.

#lorem(60)
