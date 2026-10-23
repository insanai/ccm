//! The development estimate section of the terminal report, drawn as a budget sketch:
//! one stacked bar of the cost composition, a legend that names every slice by glyph,
//! amount, and share, the total, the sensitivity range, and the drivers in one line.
//!
//! ```text
//!   ██████████████████████▒░░░░░░░░░░░░░░░░░░░░░░░░
//!   █ AI implementation             $1,346.10   46%
//!   ▒ Hosted tools                      $6.54   <1%
//!   ░ Human review                  $1,550.00   53%
//!     ─────────────────────────────────────────────
//!     Total                         $2,902.64
//!   Range  $2,744.90 ├────────●──────────┤ $3,325.00
//! ```
use crate::report::Buffer;
use crate::style::{self, Style};
use crate::{Args, Bundle, budget, economics, text};
use std::fmt::Write;

/// Indentation of the section body.
const INDENT: usize = 4;
/// The widest sketch; wider bars do not show composition more clearly.
const SKETCH_WIDTH: usize = 60;

/// A fill pattern that tells cost components apart without color.
struct Pattern {
    unicode: &'static str,
    ascii: &'static str,
    /// Whether the component is AI or tool spend, drawn in the accent color.
    machine: bool,
}

fn pattern(label: &str) -> Pattern {
    let (unicode, ascii, machine) = match label {
        "AI acceptance checks" => ("\u{2593}", "%", true),
        "Hosted tools" => ("\u{2592}", "=", true),
        "Human review" => ("\u{2591}", ":", false),
        "Human acceptance checks" => ("\u{2571}", "/", false),
        "Human implementation" => ("\u{2261}", "~", false),
        _ => ("\u{2588}", "#", true),
    };
    Pattern {
        unicode,
        ascii,
        machine,
    }
}

/// Writes the estimate section: the sketch, the range, the drivers, and the caveats;
/// `--verbose` adds the priced scope and `--technical` the full assumptions.
pub(crate) fn render(bundle: &Bundle, args: &Args, out: &mut Buffer, style: &Style) {
    let note = format!("{} profile{}USD", args.ai.year, style.dot());
    style.heading(out, "DEVELOPMENT ESTIMATE", &note);
    let width = style.width.saturating_sub(INDENT).min(SKETCH_WIDTH);
    let components = budget::components(bundle, args);
    let total = economics::total(bundle);
    sketch(out, style, &components, width);
    legend(out, style, &components, total, width);
    if let Some((low, high)) = range(bundle, args, &components) {
        range_line(out, style, (low, total, high), width);
    }
    out.push('\n');
    if !bundle.development.scope_known {
        style.item(
            out,
            &style.warn("!"),
            "Some implementation work is not mapped to source files; it is unpriced.",
            INDENT,
        );
    }
    note_parts(out, style, &drivers(bundle, args), style.dot());
    if bundle.change.is_none() {
        let parts = [
            "Project-specific acceptance work is not modeled;".to_owned(),
            "--model adds it.".to_owned(),
        ];
        note_parts(out, style, &parts, " ");
    }
    let source = format!("Assumptions: docs/economics.rst ({}).", args.ai.year);
    note_parts(
        out,
        style,
        &["Planning estimate, not a quote.".to_owned(), source],
        " ",
    );
    if args.verbose {
        verbose(bundle, args, out, style);
    }
    if args.technical {
        assumptions(bundle, args, out, style);
    }
}

/// Draws the stacked bar of the cost composition, `width` cells wide.
fn sketch(out: &mut Buffer, style: &Style, components: &[(&str, f64)], width: usize) {
    let amounts: Vec<f64> = components.iter().map(|(_, value)| *value).collect();
    let cells = allocate(&amounts, width);
    if cells.iter().all(|&count| count == 0) {
        return;
    }
    let mut bar = String::new();
    for ((label, _), count) in components.iter().zip(cells) {
        let fill = pattern(label);
        let segment = style.glyph(fill.unicode, fill.ascii).repeat(count);
        bar.push_str(&if fill.machine {
            style.accent(&segment)
        } else {
            segment
        });
    }
    let _ = writeln!(out, "{:INDENT$}{bar}", "");
}

/// Splits `width` cells among nonnegative `amounts` by largest remainder. Every nonzero
/// amount gets at least one cell so that the legend can point at it.
fn allocate(amounts: &[f64], width: usize) -> Vec<usize> {
    let sum: f64 = amounts.iter().filter(|a| **a > 0.0).sum();
    if sum <= 0.0 || width == 0 {
        return vec![0; amounts.len()];
    }
    let ideal: Vec<f64> = amounts
        .iter()
        .map(|a| a.max(0.0) / sum * width as f64)
        .collect();
    let mut cells: Vec<usize> = ideal
        .iter()
        .map(|&i| {
            if i > 0.0 {
                (i.floor() as usize).max(1)
            } else {
                0
            }
        })
        .collect();
    while cells.iter().sum::<usize>() < width {
        let next = (0..cells.len())
            .filter(|&k| ideal[k] > 0.0)
            .max_by(|&a, &b| {
                let gap = |k: usize| ideal[k] - cells[k] as f64;
                gap(a).total_cmp(&gap(b)).then(b.cmp(&a))
            });
        let Some(next) = next else { break };
        cells[next] += 1;
    }
    while cells.iter().sum::<usize>() > width {
        let Some(largest) = (0..cells.len()).max_by_key(|&k| (cells[k], usize::MAX - k)) else {
            break;
        };
        if cells[largest] <= 1 {
            break;
        }
        cells[largest] -= 1;
    }
    cells
}

/// Writes one legend row per component and the emphasized total, all right-aligned to
/// the sketch width.
fn legend(out: &mut Buffer, style: &Style, components: &[(&str, f64)], total: f64, width: usize) {
    let sum: f64 = components.iter().map(|(_, v)| v.max(0.0)).sum();
    let amount_width = components
        .iter()
        .map(|(_, value)| text::money(*value).len())
        .chain(std::iter::once(text::money(total).len()))
        .max()
        .unwrap_or(0);
    // glyph, space, label, two spaces, amount, two spaces, percent
    let label_width = width.saturating_sub(2 + 2 + amount_width + 2 + 4).max(8);
    for (label, value) in components {
        let fill = pattern(label);
        let glyph = style.glyph(fill.unicode, fill.ascii);
        let glyph = if fill.machine {
            style.accent(glyph)
        } else {
            glyph.to_owned()
        };
        let share = if sum > 0.0 { value / sum } else { 0.0 };
        let _ = writeln!(
            out,
            "{:INDENT$}{glyph} {:<label_width$}  {:>amount_width$}  {}",
            "",
            text::truncate(label, label_width, style.unicode),
            text::money(*value),
            style.dim(&format!("{:>4}", style::percent(share)))
        );
    }
    let rule = width.saturating_sub(2);
    let _ = writeln!(out, "{:INDENT$}  {}", "", style.dim(&style.line(rule)));
    let total_row = format!(
        "{:<label_width$}  {:>amount_width$}",
        "Total",
        text::money(total)
    );
    let _ = writeln!(
        out,
        "{:INDENT$}  {}",
        "",
        style.bold(&style.accent(&total_row))
    );
}

/// Returns the total when AI implementation costs per line sit at the low and the high
/// end of the default calibration study. A custom or manual profile has no such band.
fn range(bundle: &Bundle, args: &Args, components: &[(&str, f64)]) -> Option<(f64, f64)> {
    if args.custom_profile || args.agent_override {
        return None;
    }
    let implementation = bundle.development.estimate.implementation_usd;
    if implementation <= 0.0 || components.is_empty() {
        return None;
    }
    let implementation = implementation * bundle.cost.expected_attempts;
    let total = economics::total(bundle);
    let low = total - implementation * (1.0 - budget::SPAN_LOW / budget::SPAN_MIDPOINT);
    let high = total + implementation * (budget::SPAN_HIGH / budget::SPAN_MIDPOINT - 1.0);
    Some((low, high))
}

/// Draws `Range  $low ├────●──┤ $high` with the dot at the central estimate, and names
/// what varies on the line below.
fn range_line(
    out: &mut Buffer,
    style: &Style,
    (low, middle, high): (f64, f64, f64),
    width: usize,
) {
    let low_text = text::money(low);
    let high_text = text::money(high);
    let label = "Range  ";
    let fixed = label.len() + low_text.len() + high_text.len() + 4;
    let track = width.saturating_sub(fixed).max(5);
    let position = if high > low {
        ((middle - low) / (high - low) * (track - 1) as f64).round() as usize
    } else {
        track / 2
    };
    let dash = style.glyph("\u{2500}", "-");
    let track_left = dash.repeat(position);
    let track_right = dash.repeat(track - 1 - position.min(track - 1));
    out.push('\n');
    let _ = writeln!(
        out,
        "{:INDENT$}{}{} {}{}{}{}{} {}",
        "",
        style.dim(label),
        low_text,
        style.glyph("\u{251c}", "|"),
        style.dim(&track_left),
        style.bold(&style.accent(style.glyph("\u{25cf}", "o"))),
        style.dim(&track_right),
        style.glyph("\u{2524}", "|"),
        high_text
    );
    let explain = format!(
        "AI ${} to ${} per line (Span 2026)",
        budget::SPAN_LOW,
        budget::SPAN_HIGH
    );
    let indent = INDENT + label.len();
    for part in text::wrap_words(&explain, style.width.saturating_sub(indent)) {
        let _ = writeln!(out, "{:indent$}{}", "", style.dim(&part));
    }
}

/// Summarizes the inputs that move the estimate most, in one line.
fn drivers(bundle: &Bundle, args: &Args) -> Vec<String> {
    let mut parts = Vec::new();
    let lines = bundle.development.estimate.lines;
    parts.push(format!(
        "{} program {} priced",
        text::number(lines),
        if lines == 1 { "line" } else { "lines" }
    ));
    if args.agent_override {
        parts.push(format!(
            "AI and tools set to {}/attempt",
            text::money(args.cost.agent_cost)
        ));
    } else {
        parts.push(format!(
            "{}/{} per M tokens",
            price(args.ai.input_price),
            price(args.ai.output_price)
        ));
    }
    parts.push(format!("{}/h human", price(args.cost.hourly_cost)));
    let attempts = bundle.cost.expected_attempts;
    if (attempts - 1.0).abs() > f64::EPSILON {
        parts.push(format!("x{attempts:.2} expected attempts"));
    }
    parts
}

/// Formats a price without cents when it is a whole number of dollars.
fn price(amount: f64) -> String {
    if amount.fract() == 0.0 && amount.abs() < 1e12 {
        format!("${}", text::number(amount as u64))
    } else {
        format!("${amount:.2}")
    }
}

/// Writes dim secondary phrases packed into lines, breaking only between phrases.
fn note_parts(out: &mut Buffer, style: &Style, parts: &[String], separator: &str) {
    for line in text::pack(parts, separator, style.width.saturating_sub(INDENT)) {
        note_line(out, style, &line);
    }
}

/// Writes dim secondary text wrapped at the section indentation.
fn note_line(out: &mut Buffer, style: &Style, body: &str) {
    for part in text::wrap_words(body, style.width.saturating_sub(INDENT)) {
        let _ = writeln!(out, "{:INDENT$}{}", "", style.dim(&part));
    }
}

fn verbose(bundle: &Bundle, args: &Args, out: &mut Buffer, style: &Style) {
    let p = &args.ai;
    note_line(
        out,
        style,
        &format!(
            "Priced scope: {} program lines; {}.",
            text::number(bundle.development.estimate.lines),
            bundle.development.scope
        ),
    );
    note_line(
        out,
        style,
        &format!(
            "AI prices ${:.2} input / ${:.2} output per M tokens.",
            p.input_price, p.output_price
        ),
    );
}

/// Writes the full price card, priors, sources, and caveats under a subheading.
fn assumptions(bundle: &Bundle, args: &Args, out: &mut Buffer, style: &Style) {
    let mut plain = Buffer::new(1 << 20);
    budget::details(bundle, args, &mut plain);
    budget::uncertainty(bundle, &mut plain);
    let _ = writeln!(out, "\n{:INDENT$}{}", "", style.bold("Assumptions"));
    let marker = style.dim(style.glyph("\u{b7}", "-"));
    for line in plain.lines() {
        if out.failed() {
            break;
        }
        style.item_painted(out, &marker, line.trim(), INDENT, Style::dim);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocation_fills_the_width_and_shows_every_nonzero_component() {
        let cells = allocate(&[1346.10, 6.54, 1550.0], 48);
        assert_eq!(cells.iter().sum::<usize>(), 48);
        assert!(cells.iter().all(|&count| count > 0), "{cells:?}");
        assert_eq!(allocate(&[0.0, 0.0], 10), vec![0, 0]);
        assert_eq!(allocate(&[5.0, 0.0, 5.0], 10), vec![5, 0, 5]);
    }

    #[test]
    fn whole_dollar_prices_drop_their_cents() {
        assert_eq!(price(10.0), "$10");
        assert_eq!(price(2.5), "$2.50");
        assert_eq!(price(1500.0), "$1,500");
    }
}
