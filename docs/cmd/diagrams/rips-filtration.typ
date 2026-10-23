#import "@preview/cetz:0.4.2": canvas, draw
#import "style.typ": *
#show: configure

#canvas(length: 1cm, {
  import draw: *
  for offset in (0, 5.6) {
    let a = (offset, 0)
    let b = (offset, 2)
    let c = (offset + 2, 2)
    let d = (offset + 2, 0)
    let points = (a, b, c, d)
    rect((offset - 0.6, -0.75), (offset + 2.6, 3.25),
      radius: 0.16, fill: white, stroke: 0.6pt + rule)
    if offset != 0 {
      for face in ((a, b, c), (a, b, d), (a, c, d), (b, c, d)) {
        line(..face, close: true, fill: teal.transparentize(90%), stroke: none)
      }
      line(a, c, stroke: 1.1pt + amber)
      line(b, d, stroke: 1.1pt + amber)
    }
    line(a, b, c, d, close: true, stroke: 1.2pt + teal)
    for (point, label) in points.zip(("00", "01", "11", "10")) {
      circle(point, radius: 0.24, fill: white, stroke: 1pt + teal)
      content(point, text(font: "DejaVu Sans Mono", size: 9pt, label))
    }
    let heading = if offset == 0 { $epsilon = 1/2$ } else { $epsilon = 1$ }
    let betti = if offset == 0 { $beta_1 = 1$ } else { $beta_1 = 0$ }
    content((offset + 1, 2.85), heading)
    content((offset + 1, -0.46), betti)
  }
  line((3.35, 1), (4.65, 1), stroke: 0.9pt + quiet, mark: (end: ">"))
  content((4, 1.42), text(size: 8.5pt, fill: quiet)[add faces])
  content((3.8, -1.35), text(weight: "semibold")[One persistent cycle])
  line((0, -2.1), (7.7, -2.1), stroke: 0.7pt + quiet, mark: (end: ">"))
  for (x, label) in ((0, $0$), (3.6, $1/2$), (7.2, $1$)) {
    line((x, -2.02), (x, -2.18), stroke: 0.7pt + quiet)
    content((x, -2.48), label)
  }
  line((3.6, -1.75), (7.2, -1.75), stroke: 2.4pt + teal)
  circle((3.6, -1.75), radius: 0.06, fill: teal, stroke: teal)
  circle((7.2, -1.75), radius: 0.06, fill: white, stroke: 1pt + teal)
  content((-0.4, -1.75), $H_1$)
  content((8.05, -2.1), $epsilon$)
  content((3.8, -3), [birth at $1/2$; death at $1$])
})
