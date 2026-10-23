#import "@preview/cetz:0.3.4": canvas, draw
#import "@preview/fletcher:0.5.8" as fletcher: diagram, node, edge

// A deterministic graphite-style line drawing. No bitmap, color, or random seed.
#let cover-sketch() = canvas(length: 1cm, {
  import draw: *
  set-style(stroke: (paint: black, thickness: 0.55pt), fill: none)
  // Slightly offset strokes and restrained hatching suggest pencil on paper.
  let sketch(..points) = {
    line(..points.pos(), stroke: 0.55pt + black)
    scope({
      translate((0.025, -0.018))
      line(..points.pos(), stroke: 0.22pt + luma(65%))
    })
  }
  // An open notebook, seen from above and a little to the side.
  sketch((1, 1.2), (6.5, 0.5), (13, 1.8), (13.7, 7), (7, 6), (1.7, 6.7), (1, 1.2))
  sketch((1.2, 1.5), (6.5, 0.85), (12.7, 2.1))
  sketch((1.7, 6.7), (6.9, 5.8), (6.5, 0.5))
  sketch((6.9, 5.8), (13.7, 7))
  sketch((1.6, 6.45), (6.72, 5.6), (6.36, 0.9))
  for y in (2.1, 2.7, 3.3, 3.9, 4.5) {
    sketch((2.3, y), (5.5, y - 0.42))
  }
  for x in range(0, 14) {
    let a = 7.2 + x * 0.38
    line((a, 1.0 + (a - 6.5) * .2), (a + .12, 1.2 + (a - 6.5) * .2),
      stroke: .22pt + luma(62%))
  }
  // Fletcher supplies the small dependency-and-evidence drawing on the right page.
  content((10.2, 3.8), anchor: "center", diagram(
    cell-size: (11mm, 10mm), node-stroke: 0.5pt, edge-stroke: 0.5pt,
    node((0, 0), [], radius: 1.4mm),
    node((1, -1), [], radius: 1.4mm),
    node((1, 1), [], radius: 1.4mm),
    node((2, 0), [], radius: 1.4mm),
    edge((0, 0), (1, -1), "-"), edge((0, 0), (1, 1), "-"),
    edge((1, -1), (2, 0), "-"), edge((1, 1), (2, 0), "-"),
  ))
  // A pencil and its graphite tip, entirely in monochrome.
  sketch((11.5, .1), (14.8, 5.3), (15.15, 5.05), (11.9, -.15), (11.5, .1))
  sketch((11.5, .1), (11.45, -.72), (11.9, -.15))
  line((11.45, -.72), (11.65, -.4), stroke: 1.15pt + black)
  sketch((14.55, 4.9), (14.9, 4.65))
  sketch((11.72, -.02), (14.96, 5.17))
  // A short handwritten check, rather than a decorative filled symbol.
  sketch((8.7, 2.45), (9.05, 2.1), (9.8, 2.95))
})
