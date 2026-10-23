#import "@preview/fletcher:0.5.8": diagram, node, edge
#import "style.typ": *
#show: configure

#let vertex(pos, label, name) = node(pos, label, name: name,
  width: 22mm, inset: 6pt, corner-radius: 4pt)
#let layer(title, subtitle, color, body) = block(
  width: 148mm, inset: 10pt, radius: 5pt, stroke: 0.6pt + rule,
  grid(columns: (22mm, auto), column-gutter: 10pt,
    align(left + horizon)[
      #text(fill: color, weight: "bold")[#title] \
      #text(8pt, fill: quiet)[#subtitle]
    ], body))

#stack(dir: ttb, spacing: 9pt,
  layer("AST", "syntax", quiet, diagram(
    spacing: 9mm, node-stroke: 0.7pt + quiet, edge-stroke: 0.8pt + quiet,
    node-fill: rgb("#f6f7f9"),
    vertex((0, 0), [block], <b>),
    vertex((1, 0), [read], <r>),
    vertex((2, 0), [guard], <g>),
    vertex((3, 0), [sink], <s>),
    edge(<b>, <r>, "->"), edge(<b>, <g>, "->", bend: -30deg),
    edge(<g>, <s>, "->"),
  )),
  layer("CFG", "execution", blue, diagram(
    spacing: 9mm, node-stroke: 0.7pt + blue, edge-stroke: 0.9pt + blue,
    node-fill: blue-wash,
    vertex((0, 0), [read], <r>),
    vertex((1, 0), [guard], <g>),
    vertex((2, 0), [sink], <s>),
    vertex((3, 0), [exit], <e>),
    edge(<r>, <g>, "->"), edge(<g>, <s>, [true], "->"),
    edge(<s>, <e>, "->"), edge(<g>, <e>, [false], "->", bend: 35deg),
  )),
  layer("DATA", "value x", amber, diagram(
    spacing: 9mm, node-stroke: 0.7pt + amber, edge-stroke: 0.9pt + amber,
    node-fill: rgb("#fff5e6"),
    vertex((0, 0), [read], <r>),
    vertex((1, 0), [guard], <g>),
    vertex((2, 0), [sink], <s>),
    edge(<r>, <g>, $x$, "->"), edge(<r>, <s>, $x$, "->", bend: -30deg),
  )),
)
#v(8pt)
#text(8.5pt, fill: quiet)[
  read: #raw("x = read()") #h(8pt) guard: #raw("x > 0") #h(8pt) sink: #raw("sink(x)") \
  Repeated names denote the same program vertices in different edge layers.
]
