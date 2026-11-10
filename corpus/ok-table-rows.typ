// expect-none
#set page(paper: "a6", margin: 1.2cm)
#set text(size: 10pt)
The sessions of the day, with the room for each.

#table(columns: (auto, 1fr, auto), stroke: (x, y) => (bottom: 0.5pt + luma(180)),
  table.header([*Time*], [*Session*], [*Room*]),
  [9:00], [Session 1], [Room A],
  [10:00], [Session 2], [Room B],
  [11:00], [Session 3], [Room C],
  [12:00], [Session 4], [Room D],
  [13:00], [Session 5], [Room A],
  [14:00], [Session 6], [Room B],
  [15:00], [Session 7], [Room C],
  [16:00], [Session 8], [Room D],
  [17:00], [Session 9], [Room A],
  [18:00], [Session 10], [Room B],
  [19:00], [Session 11], [Room C],
  [20:00], [Session 12], [Room D],
  [21:00], [Session 13], [Room A],
  [22:00], [Session 14], [Room B],
  [23:00], [Session 15], [Room C],
  [24:00], [Session 16], [Room D],
  [25:00], [Session 17], [Room A],
  [26:00], [Session 18], [Room B],
  [27:00], [Session 19], [Room C],
  [28:00], [Session 20], [Room D],
  [29:00], [Session 21], [Room A],
)

Rooms can change at short notice; the board at the entrance shows the current plan for every session.
