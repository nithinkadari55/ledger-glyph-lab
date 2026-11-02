// expect: L035.row-baselines
#set page(paper: "a4", margin: 2cm)
#set text(size: 10pt)
#show raw.where(block: false): it => box(
  fill: luma(238), inset: (x: 3pt), radius: 2pt, text(size: 0.88em, it),
)
#table(
  columns: (auto, auto, 1fr),
  inset: 5pt,
  [*Variable*], [*Default*], [*Description*],
  [`API_HOST`], [`0.0.0.0`], [Network interface the server binds to. Use `127.0.0.1` locally.],
  [`API_PORT`], [`8000`], [TCP port for incoming requests.],
)
