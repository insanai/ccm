#import "@preview/cetz:0.3.4": canvas, draw

// The cover lattice: a square grid of modules seen in isometric projection.
// One module changes. Its change follows directed dependency edges, and the
// modules it reaches form the closure. Height and color show the distance from
// the change. Everything is computed from the constants below; there is no
// randomness, so every build draws the same picture.

#let palette = (
  ink: rgb("#26202e"),
  purple: rgb("#7341aa"),
  pink: rgb("#ee77ad"),
  line: rgb("#cfc6d8"),
  floor: rgb("#f3edf8"),
)

#let lattice-size = 9        // Modules along each side of the grid.
#let pitch = 1.0             // Distance between neighboring modules.
#let module-size = 0.46      // Edge length of a module's footprint.
#let rest-height = 0.07      // Height of a module the change does not reach.
#let peak-height = 2.1       // Height of the changed module.
#let changed = (4, 3)        // Grid position of the changed module.
#let link-seed = 16          // Selects one of many equally valid lattices.
#let edge-share = 64         // Percent of candidate edges that exist.

// Isometric projection: x runs down to the right, y down to the left, z up.
#let iso((x, y, z)) = {
  let c = calc.cos(30deg)
  ((x - y) * c, -(x + y) / 2 + z)
}

// A small integer hash of a grid link, in 0..100. The link joins (i, j) to
// its neighbor one step further along `axis` (0 for x, 1 for y). Knuth's
// multiplicative constant spreads consecutive link numbers apart.
#let link-hash(i, j, axis) = {
  let n = (i * lattice-size + j) * 2 + axis + link-seed
  calc.rem(calc.quo(calc.rem(n * 2654435761, 4294967296), 65536), 100)
}

// A link exists when its hash falls below the edge share. Its direction comes
// from the parity of the hash: even links point forward, odd links point back.
#let link(i, j, axis) = {
  let h = link-hash(i, j, axis)
  if h >= edge-share { return none }
  calc.even(h)
}

// Successors of a module: the neighbors that depend on it.
#let successors(i, j) = {
  let out = ()
  for (axis, (di, dj)) in ((0, (1, 0)), (1, (0, 1))) {
    let (k, l) = (i + di, j + dj)
    if k < lattice-size and l < lattice-size and link(i, j, axis) == true {
      out.push((k, l))
    }
    let (k, l) = (i - di, j - dj)
    if k >= 0 and l >= 0 and link(k, l, axis) == false { out.push((k, l)) }
  }
  out
}

// Breadth-first distances from the changed module. Unreached modules map to none.
#let distances() = {
  let key((i, j)) = str(i) + "," + str(j)
  let seen = ((key(changed)): 0)
  let frontier = (changed,)
  while frontier.len() > 0 {
    let next = ()
    for (i, j) in frontier {
      for s in successors(i, j) {
        if key(s) not in seen {
          seen.insert(key(s), seen.at(key((i, j))) + 1)
          next.push(s)
        }
      }
    }
    frontier = next
  }
  (i, j) => seen.at(key((i, j)), default: none)
}

// Mixes color `a` towards color `b` by the fraction `t`, in sRGB like the
// Rust logo generator, so the cover and the mark share exact tones.
#let mix(a, b, t) = color.mix((a, 100% - t), (b, t), space: rgb)

// Face colors (top, left, right) of a module at closure distance d. Light comes
// from the upper left, as in the logo. Deeper modules fade towards white.
#let tones(d, deepest) = {
  if d == none { return (white, white, white) }
  let (pink, purple, ink) = (palette.pink, palette.purple, palette.ink)
  let faces = if d == 0 {
    (mix(pink, white, 60%), pink, mix(pink, purple, 55%))
  } else {
    (mix(mix(purple, pink, 30%), white, 42%), purple, mix(purple, ink, 60%))
  }
  let fade = if d == 0 { 0% } else { (d - 1) / deepest * 62% }
  faces.map(c => mix(c, white, fade))
}

#let cover-art() = canvas(length: 0.9cm, {
  import draw: *
  let dist = distances()
  let cells = range(lattice-size).map(i => range(lattice-size).map(j => (i, j))).join()
  let deepest = calc.max(..cells.map(((i, j)) => {
    let d = dist(i, j)
    if d == none { 0 } else { d }
  }))
  let height(d) = if d == none { rest-height } else {
    rest-height + (peak-height - rest-height) * calc.pow(1 - d / (deepest + 1), 3)
  }
  let half = pitch / 2
  // The closure region, tinted on the floor.
  for (i, j) in cells {
    if dist(i, j) != none {
      let (x, y) = (i * pitch, j * pitch)
      line(iso((x - half, y - half, 0)), iso((x + half, y - half, 0)),
        iso((x + half, y + half, 0)), iso((x - half, y + half, 0)), close: true,
        fill: palette.floor, stroke: none)
    }
  }
  // The boundary of the closure: every cell side whose neighbor lies outside.
  for (i, j) in cells.filter(((i, j)) => dist(i, j) != none) {
    let (x, y) = (i * pitch, j * pitch)
    for ((di, dj), (a, b), (c, e)) in (
      ((1, 0), (1, -1), (1, 1)), ((-1, 0), (-1, -1), (-1, 1)),
      ((0, 1), (-1, 1), (1, 1)), ((0, -1), (-1, -1), (1, -1)),
    ) {
      let (k, l) = (i + di, j + dj)
      let outside = k < 0 or l < 0 or k >= lattice-size or l >= lattice-size
      if outside or dist(k, l) == none {
        line(iso((x + a * half, y + b * half, 0)), iso((x + c * half, y + e * half, 0)),
          stroke: 0.6pt + mix(palette.purple, white, 35%))
      }
    }
  }
  // Dependency edges on the floor, from each module to its successors.
  for (i, j) in cells {
    for (k, l) in successors(i, j) {
      let inside = dist(i, j) != none
      line(iso((i * pitch, j * pitch, 0)), iso((k * pitch, l * pitch, 0)),
        stroke: if inside { 0.7pt + palette.purple } else { 0.45pt + palette.line })
    }
  }
  // Modules, painted from the back corner to the front corner.
  let r = module-size / 2
  for (i, j) in cells.sorted(key: ((i, j)) => i + j) {
    let d = dist(i, j)
    let (top, left, right) = tones(d, deepest)
    let h = height(d)
    let (x, y) = (i * pitch, j * pitch)
    let edge = if d == none { 0.45pt + palette.line.darken(25%) } else { 0.35pt + white }
    let p(a, b, z) = iso((x + a * r, y + b * r, z))
    line(p(1, -1, 0), p(1, 1, 0), p(1, 1, h), p(1, -1, h), close: true,
      fill: right, stroke: edge)
    line(p(-1, 1, 0), p(1, 1, 0), p(1, 1, h), p(-1, 1, h), close: true,
      fill: left, stroke: edge)
    line(p(-1, -1, h), p(1, -1, h), p(1, 1, h), p(-1, 1, h), close: true,
      fill: top, stroke: edge)
  }
})
