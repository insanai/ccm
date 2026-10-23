#import "cover-art.typ": cover-sketch

#let book(title: "CCM", author: "", version: "", date: "", lang: "en", paper: "iso-b5",
  logo: none, copyright: "", accent: rgb("#7341aa"), numbering: false,
  version-label: "Version", body) = {
  // The CCM print edition uses a fixed B5 trim.
  let paper = "iso-b5"
  let ink = rgb("#26202e")
  let quiet = rgb("#6d6577")
  let rule = rgb("#ded4e5")
  let sans = ("DejaVu Sans", "Libertinus Serif")
  set document(title: title, author: author)
  set text(font: ("Libertinus Serif", "New Computer Modern"), size: 10.5pt,
    fill: ink, lang: lang, hyphenate: true)
  set par(justify: true, leading: .64em, spacing: .8em, first-line-indent: 1em)
  set heading(numbering: none)
  show heading: set text(font: sans, hyphenate: false)
  show heading.where(level: 1): it => {
    if it.outlined { pagebreak(weak: true) }
    block(above: 11mm, below: 9mm, sticky: true)[
      #line(length: 18mm, stroke: 2pt + rgb("#ee77ad"))
      #v(5mm)
      #text(size: 24pt, weight: "bold", fill: accent, it.body)
    ]
  }
  show heading.where(level: 2): it => block(above: 1.7em, below: .7em, sticky: true,
    text(size: 14pt, weight: "bold", fill: ink, it.body))
  show heading.where(level: 3): it => block(above: 1.3em, below: .5em, sticky: true,
    text(size: 11pt, weight: "bold", fill: accent, it.body))
  show link: set text(fill: accent)
  show raw: set text(font: "DejaVu Sans Mono", size: .79em, hyphenate: false)
  show raw.where(block: true): it => block(width: 100%, breakable: true,
    inset: 10pt, fill: rgb("#f7f3f9"), radius: 3pt, it)
  set table(stroke: .4pt + rule, inset: 6pt)
  show table: set text(size: .86em)
  show table: set par(justify: false)
  show figure.caption: set text(size: 9pt, fill: quiet)
  set figure(gap: 3mm)
  page(paper: paper, margin: (x: 19mm, top: 17mm, bottom: 17mm),
    header: none, footer: none, numbering: none)[
    #set par(justify: false, leading: .3em, first-line-indent: 0pt)
    #if logo != none { image(logo, width: 34mm) }
    #v(7mm)
    #text(font: sans, size: 10pt, weight: "bold", tracking: .17em)[THE CCM BOOK]
    #v(6mm)
    #text(font: sans, size: 39pt, weight: "bold", tracking: -.025em)[
      Code, change,\
      and confidence.
    ]
    #v(5mm)
    #text(size: 13pt, fill: quiet)[Algorithms, evidence, and the economics \ of software development]
    #v(10mm)
    #align(center, scale(x: 75%, y: 75%, reflow: true, cover-sketch()))
    #v(7mm)
    #line(length: 100%, stroke: .6pt + ink)
    #v(4mm)
    #text(font: sans, size: 11pt, weight: "bold", author)
    #h(1fr)
    #text(font: sans, size: 9pt)[EDITION #version / 2026]
  ]
  set page(paper: paper, margin: (inside: 21mm, outside: 19mm, top: 22mm, bottom: 20mm),
    header: context align(end, text(font: sans, size: 8pt, fill: quiet)[CCM / CODE, CHANGE, CONFIDENCE]),
    footer: context {
      line(length: 100%, stroke: .4pt + rule)
      v(2mm)
      text(font: sans, size: 8pt, fill: quiet)[#version-label #version]
      h(1fr)
      text(font: sans, size: 9pt, fill: accent, counter(page).display("1"))
    })
  counter(page).update(1)
  outline(title: [Contents], depth: 2, indent: 1.2em)
  pagebreak()
  body
  context [#metadata(counter(page).final().first()) <ccm-pages>]
}
