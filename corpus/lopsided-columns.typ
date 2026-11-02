// expect: L046.lopsided-columns
#set page(paper: "a4", margin: 2cm)
#set text(size: 10pt)
#table(
  columns: (1fr, 4cm),
  inset: 5pt,
  [*Risk*], [*Mitigation*],
  [Data loss during transfer], [Checksums on every batch, parallel running for each wave, and a tested rollback plan.],
  [Disruption at cutover], [Cutover on a quiet weekend, printed downtime procedures, and helpers on every ward.],
  [Costs above forecast], [Monthly cost reviews, budget alerts, and a contingency held by the committee.],
)
