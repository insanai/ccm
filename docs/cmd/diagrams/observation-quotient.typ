#import "@preview/fletcher:0.5.8": diagram, node, edge
#import "style.typ": *
#show: configure

#diagram(
  spacing: (25mm, 20mm),
  node-stroke: 0.8pt + blue,
  edge-stroke: 0.9pt + blue,
  node-fill: blue-wash,
  node-inset: 10pt,
  node((0, 0), align(center)[$X$ \ #text(9pt, fill: quiet)[concrete cases]],
    name: <x>, width: 36mm, corner-radius: 5pt),
  node((1, 0), align(center)[$Y$ \ #text(9pt, fill: quiet)[concrete cases]],
    name: <y>, width: 36mm, corner-radius: 5pt),
  node((0, 1), align(center)[$Q(X)$ \ #text(9pt, fill: quiet)[observation classes]],
    name: <qx>, width: 36mm, fill: teal-wash, stroke: teal, corner-radius: 5pt),
  node((1, 1), align(center)[$Q(Y)$ \ #text(9pt, fill: quiet)[observation classes]],
    name: <qy>, width: 36mm, fill: teal-wash, stroke: teal, corner-radius: 5pt),
  edge(<x>, <y>, $f$, "->"),
  edge(<x>, <qx>, $q_X$, "->"),
  edge(<y>, <qy>, $q_Y$, "->"),
  edge(<qx>, <qy>, $Q(f)$, "->", stroke: teal),
)
#v(10pt)
#align(center)[$Q(f) compose q_X = q_Y compose f$]
