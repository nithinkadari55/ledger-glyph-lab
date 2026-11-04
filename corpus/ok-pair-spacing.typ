// expect-none
#set page(paper: "a4", margin: 2cm)
#set text(size: 10pt)
#let cmd(c, d) = {
  block(above: 0pt, below: 7pt, text(font: "DejaVu Sans Mono", size: 9pt, c))
  block(above: 0pt, below: 16pt, d)
}
#cmd("git init", [Turn the current folder into a new repository.])
#cmd("git clone <url>", [Download a repository with its full history.])
#cmd("git status", [Show changed, staged and untracked files.])
#cmd("git add <file>", [Stage one file for the next commit.])
