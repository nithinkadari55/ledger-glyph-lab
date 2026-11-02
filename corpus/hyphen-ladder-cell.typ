// expect: L012.hyphen-ladder
#set page(paper: "a4", margin: (x: 22mm, y: 22mm))
#set text(font: "Libertinus Serif", size: 11pt, lang: "en", hyphenate: true)
#show table: set par(justify: false)
#table(
  columns: (28%, 1fr),
  [*Chair*], [Helena Marsh, Chief Operating Officer],
  [*Attendees*], [Helena Marsh (Chair); David Okafor (Project Director); Priya Nair (Project Office, minutes); Tomás Rivera (Finance); Sarah Lindqvist (IT Infrastructure); Marcus Webb (Operations); Aisha Rahman (Change Management)],
)
