// expect: L070.pair-spacing
#set page(paper: "a4", margin: 2cm)
#set text(size: 10pt)
= Set up
#let cmd(c, d) = {
  block(above: 0pt, below: 16pt, block(fill: luma(240), inset: 3pt, raw(c)))
  block(above: 0pt, below: 7pt, d)
}
#cmd("git init", [Turn the current folder into a new repository.])
#cmd("git clone <url>", [Download a repository with its full history.])
#cmd("git status", [Show changed, staged and untracked files.])
#cmd("git add <file>", [Stage one file for the next commit.])
