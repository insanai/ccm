#let ink = rgb("#1b2330")
#let quiet = rgb("#5b6574")
#let teal = rgb("#0f766e")
#let teal-wash = rgb("#e7f3f1")
#let blue = rgb("#0c4a6e")
#let blue-wash = rgb("#eef4f8")
#let amber = rgb("#b45309")
#let rule = rgb("#cbd5e1")
#let configure(body) = {
  set page(width: auto, height: auto, margin: 12pt, fill: white)
  set text(font: "Libertinus Sans", size: 10pt, fill: ink)
  set par(justify: false)
  body
}
