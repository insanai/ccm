// Finite sanity checks for CMD 0002; these do not replace its proofs.
use std::collections::BTreeSet;

fn closure(seed: u32, edges: u32, n: u32) -> u32 {
    let mut reached = seed;
    loop {
        let before = reached;
        for u in 0..n {
            for v in 0..n {
                if reached & (1 << u) != 0 && edges & (1 << (u * n + v)) != 0 {
                    reached |= 1 << v;
                }
            }
        }
        if before == reached {
            return reached;
        }
    }
}

fn check_closure() -> u64 {
    let mut checks = 0;
    for edges in 0..(1 << 9) {
        for s in 0..8 {
            let ds = closure(s, edges, 3);
            assert_eq!(ds & s, s);
            assert_eq!(closure(ds, edges, 3), ds);
            for t in 0..8 {
                let dt = closure(t, edges, 3);
                let du = closure(s | t, edges, 3);
                assert_eq!(du, ds | dt);
                assert_eq!(
                    (du.count_ones() + (ds & dt).count_ones()),
                    ds.count_ones() + dt.count_ones()
                );
                if s & t == s {
                    assert_eq!(ds & dt, ds);
                }
                checks += 1;
            }
        }
    }
    checks
}

fn coverage(selected: u32, routes: &[u32; 3]) -> u32 {
    let mut out = 0;
    for (i, set) in routes.iter().enumerate() {
        if selected & (1 << i) != 0 {
            out |= set;
        }
    }
    out.count_ones()
}

fn check_coverage() -> u64 {
    let mut checks = 0;
    for a in 0..8 {
        for b in 0..8 {
            for c in 0..8 {
                let routes = [a, b, c];
                for s in 0..8 {
                    for t in 0..8 {
                        if s & t != s {
                            continue;
                        }
                        for e in 0..3 {
                            let ds = coverage(s | (1 << e), &routes) - coverage(s, &routes);
                            let dt = coverage(t | (1 << e), &routes) - coverage(t, &routes);
                            assert!(ds >= dt);
                            checks += 1;
                        }
                    }
                }
            }
        }
    }
    checks
}

fn check_inversion() -> u64 {
    let mut checks = 0;
    for encoded in 0..2187_u32 {
        let mut f = [0_i32; 8];
        let mut number = encoded;
        for value in f.iter_mut().skip(1) {
            *value = (number % 3) as i32;
            number /= 3;
        }
        let mut mu = [0_i32; 8];
        for t in 0_u32..8 {
            for u in 0_u32..8 {
                if u & t == u {
                    let sign = if (t.count_ones() - u.count_ones()) % 2 == 0 {
                        1
                    } else {
                        -1
                    };
                    mu[t as usize] += sign * f[u as usize];
                }
            }
        }
        for (s, expected) in f.iter().enumerate() {
            let recovered: i32 = (0..8).filter(|t| t & s == *t).map(|t| mu[t]).sum();
            assert_eq!(recovered, *expected);
            checks += 1;
        }
    }
    checks
}

fn check_observers() -> u64 {
    let mut checks = 0;
    for a in 0_u32..81 {
        for b in 0_u32..81 {
            let mut oa = a;
            let mut ob = b;
            let mut left = BTreeSet::new();
            let mut right = BTreeSet::new();
            let mut joint = BTreeSet::new();
            for _ in 0..4 {
                left.insert(oa % 3);
                right.insert(ob % 3);
                joint.insert((oa % 3, ob % 3));
                oa /= 3;
                ob /= 3;
            }
            assert!(joint.len() >= left.len().max(right.len()));
            assert!(joint.len() <= left.len() * right.len());
            checks += 1;
        }
    }
    checks
}

fn residual(seed: u32, edges: u32, supports: &[u32]) -> u32 {
    let reached = closure(seed, edges, 3);
    u32::from(supports.iter().all(|support| support & reached != 0))
}

fn check_cuts() -> u64 {
    let mut checks = 0;
    for edges in 0..512 {
        for encoded in 0..512 {
            let supports = [encoded & 7, (encoded >> 3) & 7, (encoded >> 6) & 7];
            let mut minimum = None;
            let mut h = 0;
            for seed in 0_u32..8 {
                let reached = closure(seed, edges, 3);
                let product: u32 = supports
                    .iter()
                    .map(|support| {
                        let inner: u32 = (0..3)
                            .filter(|v| support & (1 << v) != 0)
                            .map(|v| 1 - ((reached >> v) & 1))
                            .product();
                        1 - inner
                    })
                    .product();
                assert_eq!(product, residual(seed, edges, &supports));
                if product != 0 {
                    minimum =
                        Some(minimum.map_or(seed.count_ones(), |x: u32| x.min(seed.count_ones())));
                }
                for other in 0..8 {
                    if seed & other == seed {
                        assert!(product <= residual(other, edges, &supports));
                    }
                }
                checks += 1;
            }
            for v in 0..3 {
                let reached = closure(1 << v, edges, 3);
                h = h.max(supports.iter().filter(|set| *set & reached != 0).count() as u32);
            }
            if supports.iter().all(|s| *s != 0) {
                let k = minimum.unwrap();
                assert!(k >= 3_u32.div_ceil(h));
                assert!(k <= 3);
            } else {
                assert!(minimum.is_none());
            }
        }
    }
    assert_eq!(residual(0, 0, &[]), 1);
    checks
}

fn activation(family: u32, changed: u32) -> bool {
    (0..8).any(|trigger| family & (1 << trigger) != 0 && trigger & changed == trigger)
}

fn check_triggers() -> u64 {
    let mut checks = 0;
    let mut antichains = 0;
    for family in 0..256 {
        let is_antichain = (0..8).all(|a| {
            (0..8)
                .all(|b| a == b || family & (1 << a) == 0 || family & (1 << b) == 0 || a & b != a)
        });
        if !is_antichain {
            continue;
        }
        antichains += 1;
        for edges in 0..512 {
            for s in 0..8 {
                for t in 0..8 {
                    if s & t == s {
                        let left = activation(family, closure(s, edges, 3));
                        let right = activation(family, closure(t, edges, 3));
                        assert!(!left || right);
                        checks += 1;
                    }
                }
            }
        }
    }
    assert_eq!(antichains, 20);
    checks
}

fn check_envelopes() -> u64 {
    let mut checks = 0;
    for encoded in 0..19683 {
        let mut lo = 0;
        let mut hi = 0;
        let mut value = encoded;
        for bit in 0..9 {
            match value % 3 {
                0 => {}
                1 => hi |= 1 << bit,
                2 => {
                    lo |= 1 << bit;
                    hi |= 1 << bit;
                }
                _ => unreachable!(),
            }
            value /= 3;
        }
        for seed in 0..8 {
            let low = closure(seed, lo, 3);
            let high = closure(seed, hi, 3);
            assert_eq!(low & high, low);
            checks += 1;
        }
    }
    checks
}

fn check_impact() -> u64 {
    let control = [2_i64, 5, 8];
    let agent = [4_i64, 3, 12];
    let probability_numerators = [1_i64, 1, 2];
    let probability_denominators = [3_i64, 2, 3];
    let mut weighted_sum = 0;
    for assignment in 0..8 {
        let mut probability_weight = 1;
        let mut twice_estimate_numerator = 0;
        for i in 0..3 {
            let p = probability_numerators[i];
            let d = probability_denominators[i];
            if assignment & (1 << i) != 0 {
                probability_weight *= p;
                twice_estimate_numerator += 2 * d * agent[i] / p;
            } else {
                probability_weight *= d - p;
                twice_estimate_numerator -= 2 * d * control[i] / (d - p);
            }
        }
        weighted_sum += probability_weight * twice_estimate_numerator;
    }
    let effect_sum: i64 = agent.iter().zip(control).map(|(a, b)| a - b).sum();
    assert_eq!(weighted_sum, 18 * 2 * effect_sum);
    8
}

fn observation_count(cases: u32, observer: u32) -> u32 {
    let mut image = 0_u32;
    for point in 0..4 {
        if cases & (1 << point) != 0 {
            image |= 1 << ((observer >> point) & 1);
        }
    }
    image.count_ones()
}

fn check_ambiguity() -> u64 {
    let mut checks = 0;
    let mut positive = 0;
    let mut negative = 0;
    for universe in 1..16 {
        for observer in 0..16 {
            for e in 0..16 {
                for f in 0..16 {
                    let both = universe & e & f;
                    if both == 0 {
                        continue;
                    }
                    let n = observation_count(universe, observer);
                    let ne = observation_count(universe & e, observer);
                    let nf = observation_count(universe & f, observer);
                    let nef = observation_count(both, observer);
                    assert!(n >= ne && n >= nf && ne >= nef && nf >= nef);
                    let direct = (n as f64 / nef as f64).log2();
                    let staged = (n as f64 / ne as f64).log2() + (ne as f64 / nef as f64).log2();
                    assert!((direct - staged).abs() < 1e-12);
                    positive += u64::from(ne * nf > n * nef);
                    negative += u64::from(ne * nf < n * nef);
                    checks += 1;
                }
            }
        }
    }
    assert!(positive > 0 && negative > 0);
    assert_eq!(observation_count(15 & 5, 12), 2);
    assert_eq!(observation_count(15 & 9, 12), 2);
    assert_eq!(observation_count(15 & 5 & 9, 12), 1);
    checks
}

fn check_persistence_area() -> u64 {
    let mut checks = 0;
    for a in 1_u32..16 {
        for b in 1_u32..16 {
            for c in 1_u32..16 {
                for d in 1_u32..16 {
                    if b & a != b || d & c != d || a & c != a || b & d != b {
                        continue;
                    }
                    let left =
                        (a.count_ones() as f64).log2() + 2.0 * (b.count_ones() as f64).log2();
                    let right =
                        (c.count_ones() as f64).log2() + 2.0 * (d.count_ones() as f64).log2();
                    assert!(left >= 0.0 && right <= 6.0);
                    assert!(left <= right + 1e-12);
                    checks += 1;
                }
            }
        }
    }
    checks
}

fn compatible(map: &[usize; 3], source: &[usize; 3], target: &[usize; 3]) -> bool {
    (0..3).all(|x| (0..3).all(|y| source[x] != source[y] || target[map[x]] == target[map[y]]))
}

fn ternary_map(mut code: usize) -> [usize; 3] {
    let mut map = [0; 3];
    for value in &mut map {
        *value = code % 3;
        code /= 3;
    }
    map
}

fn check_quotients() -> u64 {
    let partitions = [[0, 0, 0], [0, 0, 1], [0, 1, 0], [0, 1, 1], [0, 1, 2]];
    let mut checks = 0;
    for x in &partitions {
        assert!(compatible(&[0, 1, 2], x, x));
        for y in &partitions {
            for z in &partitions {
                for f_code in 0..27 {
                    let f = ternary_map(f_code);
                    if !compatible(&f, x, y) {
                        continue;
                    }
                    for g_code in 0..27 {
                        let g = ternary_map(g_code);
                        if !compatible(&g, y, z) {
                            continue;
                        }
                        let composed = f.map(|v| g[v]);
                        assert!(compatible(&composed, x, z));
                        for a in 0..3 {
                            for b in 0..3 {
                                if x[a] == x[b] {
                                    assert_eq!(z[composed[a]], z[composed[b]]);
                                }
                            }
                        }
                        checks += 1;
                    }
                }
            }
        }
    }
    checks
}

fn check_words() -> u64 {
    let mut checks = 0;
    for code in 0..81 {
        let mut transitions = [0_usize; 4];
        let mut remaining = code;
        for destination in &mut transitions {
            *destination = remaining % 3;
            remaining /= 3;
        }
        for start in 0..2 {
            for accepting in 0..4 {
                let mut counts = [0_u64; 2];
                counts[start] = 1;
                for horizon in 0..6 {
                    let mut brute = 0;
                    for word in 0..(1 << horizon) {
                        let mut state = start;
                        for offset in 0..horizon {
                            state = transitions[2 * state + ((word >> offset) & 1)];
                            if state == 2 {
                                break;
                            }
                        }
                        brute += u64::from(state < 2 && accepting & (1 << state) != 0);
                    }
                    let recurrence: u64 = counts
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| accepting & (1 << i) != 0)
                        .map(|(_, n)| n)
                        .sum();
                    assert_eq!(recurrence, brute);
                    let mut next = [0; 2];
                    for (state, count) in counts.iter().enumerate() {
                        for symbol in 0..2 {
                            let to = transitions[2 * state + symbol];
                            if to < 2 {
                                next[to] += count;
                            }
                        }
                    }
                    counts = next;
                    checks += 1;
                }
            }
        }
    }
    checks
}

fn binary_rank(columns: &[u64]) -> u32 {
    let mut pivots = [0_u64; 64];
    let mut rank = 0;
    for &column in columns {
        let mut value = column;
        for bit in (0..64).rev() {
            if value & (1_u64 << bit) == 0 {
                continue;
            }
            if pivots[bit] != 0 {
                value ^= pivots[bit];
            } else {
                pivots[bit] = value;
                rank += 1;
                break;
            }
        }
    }
    rank
}

fn check_square_homology() -> u64 {
    for threshold in 0..3 {
        let mut edges = Vec::new();
        let mut boundaries = Vec::new();
        for u in 0_u32..4 {
            for v in u + 1..4 {
                if (u ^ v).count_ones() <= threshold {
                    edges.push((u, v));
                    boundaries.push((1 << u) | (1 << v));
                }
            }
        }
        let mut triangles = Vec::new();
        for a in 0..4 {
            for b in a + 1..4 {
                for c in b + 1..4 {
                    let column = edges.iter().enumerate().fold(0_u64, |out, (i, &(u, v))| {
                        out | if [a, b, c].contains(&u) && [a, b, c].contains(&v) {
                            1 << i
                        } else {
                            0
                        }
                    });
                    if column.count_ones() == 3 {
                        let boundary = boundaries
                            .iter()
                            .enumerate()
                            .filter(|(i, _)| column & (1 << i) != 0)
                            .fold(0, |out, (_, face)| out ^ face);
                        assert_eq!(boundary, 0);
                        triangles.push(column);
                    }
                }
            }
        }
        let rank_one = binary_rank(&boundaries);
        let rank_two = binary_rank(&triangles);
        let beta_one = edges.len() as u32 - rank_one - rank_two;
        assert_eq!(beta_one, u32::from(threshold == 1));
        assert_eq!(4 - rank_one, if threshold == 0 { 4 } else { 1 });
    }
    3
}

fn check_spectral_energy() -> u64 {
    let pairs = [(0, 1), (0, 2), (1, 2)];
    let mut checks = 0;
    for code in 0..27 {
        let weights = ternary_map(code);
        let mut laplacian = [[0_i64; 3]; 3];
        for (&(a, b), &weight) in pairs.iter().zip(&weights) {
            let w = weight as i64;
            laplacian[a][a] += w;
            laplacian[b][b] += w;
            laplacian[a][b] -= w;
            laplacian[b][a] -= w;
        }
        for vector in 0..27 {
            let f = ternary_map(vector).map(|value| value as i64 - 1);
            let energy: i64 = pairs
                .iter()
                .zip(&weights)
                .map(|(&(a, b), &w)| w as i64 * (f[a] - f[b]).pow(2))
                .sum();
            let mut quadratic = 0;
            for (i, row) in laplacian.iter().enumerate() {
                for (j, value) in row.iter().enumerate() {
                    quadratic += f[i] * value * f[j];
                }
            }
            assert_eq!(quadratic, energy);
            assert!(energy >= 0);
            checks += 1;
        }
    }
    checks
}

fn check_separator_counts() -> u64 {
    let mut checks = 0;
    for left in 0..16 {
        for right in 0..16 {
            let mut brute = 0;
            for x in 0..2 {
                for y in 0..2 {
                    for z in 0..2 {
                        brute += ((left >> (2 * x + y)) & 1) * ((right >> (2 * y + z)) & 1);
                    }
                }
            }
            let mut messages = 0;
            for y in 0..2 {
                let a: u32 = (0..2).map(|x| (left >> (2 * x + y)) & 1).sum();
                let b: u32 = (0..2).map(|z| (right >> (2 * y + z)) & 1).sum();
                messages += a * b;
            }
            assert_eq!(messages, brute);
            checks += 1;
        }
    }
    // Unconstrained hidden y doubles full counts without adding x,z observations.
    assert_eq!(2_u32.pow(3), 2 * 2_u32.pow(2));
    checks
}

fn main() {
    println!("ambiguity cases: {}", check_ambiguity());
    println!("persistence area cases: {}", check_persistence_area());
    println!("quotients cases: {}", check_quotients());
    println!("words cases: {}", check_words());
    println!("square homology cases: {}", check_square_homology());
    println!("spectral energy cases: {}", check_spectral_energy());
    println!("separator counts cases: {}", check_separator_counts());
    println!("trigger monotonicity pairs: {}", check_triggers());
    println!("nested-graph seed envelopes: {}", check_envelopes());
    println!("randomized assignments: {}", check_impact());
    println!("closure pairs: {}", check_closure());
    println!("evidence marginal inequalities: {}", check_coverage());
    println!("interaction reconstructions: {}", check_inversion());
    println!("observer pairs: {}", check_observers());
    println!("certificate seed queries: {}", check_cuts());
}
