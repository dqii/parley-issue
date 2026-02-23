#[cfg(test)]
#[derive(Clone)]
enum RangedStyle {
    None,
    Bold(std::ops::Range<usize>),
    Underline(std::ops::Range<usize>),
    Size(std::ops::Range<usize>, f32),
}

#[cfg(test)]
const TEXT: &str = "Hello world!!!!";

#[cfg(test)]
fn first_word() -> std::ops::Range<usize> {
    0..5 // "Hello"
}

#[cfg(test)]
fn second_word() -> std::ops::Range<usize> {
    6..15 // "world!!!!"
}

#[cfg(test)]
fn build_layout_width(text: &str, style: RangedStyle) -> f32 {
    use parley::style::{FontStack, StyleProperty};
    use parley::{FontContext, LayoutContext};

    let mut font_cx = FontContext::new();
    let mut layout_cx = LayoutContext::<peniko::Brush>::new();
    let mut builder = layout_cx.ranged_builder(&mut font_cx, text, 1.0, false);

    builder.push_default(StyleProperty::FontStack(FontStack::from("Inter")));
    builder.push_default(StyleProperty::FontSize(16.0));
    builder.push_default(StyleProperty::FontWeight(parley::style::FontWeight::new(
        400.0,
    )));
    builder.push_default(StyleProperty::Brush(peniko::Brush::Solid(
        peniko::color::AlphaColor::new([0.0, 0.0, 0.0, 1.0]),
    )));

    match style {
        RangedStyle::None => {}
        RangedStyle::Bold(range) => builder.push(
            StyleProperty::FontWeight(parley::style::FontWeight::new(700.0)),
            range,
        ),
        RangedStyle::Underline(range) => builder.push(StyleProperty::Underline(true), range),
        RangedStyle::Size(range, size) => builder.push(StyleProperty::FontSize(size), range),
    }

    let mut layout = builder.build(text);
    layout.break_all_lines(None);
    layout.width()
}

/// Returns (has_underlined_run, has_sized_run) from glyph runs.
#[cfg(test)]
fn check_run_properties(text: &str, style: RangedStyle) -> (bool, bool) {
    use parley::style::{FontStack, StyleProperty};
    use parley::{FontContext, LayoutContext};

    let mut font_cx = FontContext::new();
    let mut layout_cx = LayoutContext::<peniko::Brush>::new();
    let mut builder = layout_cx.ranged_builder(&mut font_cx, text, 1.0, false);

    builder.push_default(StyleProperty::FontStack(FontStack::from("Inter")));
    builder.push_default(StyleProperty::FontSize(16.0));
    builder.push_default(StyleProperty::FontWeight(parley::style::FontWeight::new(
        400.0,
    )));
    builder.push_default(StyleProperty::Brush(peniko::Brush::Solid(
        peniko::color::AlphaColor::new([0.0, 0.0, 0.0, 1.0]),
    )));

    let target_size = match &style {
        RangedStyle::Size(_, s) => Some(*s),
        _ => None,
    };

    match style {
        RangedStyle::None => {}
        RangedStyle::Bold(range) => builder.push(
            StyleProperty::FontWeight(parley::style::FontWeight::new(700.0)),
            range,
        ),
        RangedStyle::Underline(range) => builder.push(StyleProperty::Underline(true), range),
        RangedStyle::Size(range, size) => builder.push(StyleProperty::FontSize(size), range),
    }

    let mut layout = builder.build(text);
    layout.break_all_lines(None);

    let mut has_underline = false;
    let mut has_sized = false;
    for line in layout.lines() {
        for item in line.items() {
            if let parley::layout::PositionedLayoutItem::GlyphRun(gr) = item {
                if gr.style().underline.is_some() {
                    has_underline = true;
                }
                if let Some(target) = target_size {
                    if (gr.run().font_size() - target).abs() < 0.01 {
                        has_sized = true;
                    }
                }
            }
        }
    }
    (has_underline, has_sized)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn regular_width() -> f32 {
        build_layout_width(TEXT, RangedStyle::None)
    }

    // ── Bold ────────────────────────────────────────────────────────

    #[test]
    fn test_bold_first_word_changes_width() {
        let width = build_layout_width(TEXT, RangedStyle::Bold(first_word()));
        let delta = width - regular_width();
        assert!(
            delta > 0.1,
            "Bold on first word should widen layout (delta={delta})"
        );
    }

    /// BUG: bold on the second word does NOT widen the layout on stock
    /// parley 0.7.0 because fontique's `fallback_families` cache is not
    /// invalidated when the `FontWeight` attribute changes between runs.
    #[test]
    fn test_bold_second_word_changes_width() {
        let width = build_layout_width(TEXT, RangedStyle::Bold(second_word()));
        let delta = width - regular_width();
        assert!(
            delta > 0.1,
            "Bold on second word should widen layout (delta={delta})"
        );
    }

    // ── Control cases (these always pass) ───────────────────────────

    #[test]
    fn test_underline_works_on_both_ranges() {
        let (ul, _) = check_run_properties(TEXT, RangedStyle::Underline(first_word()));
        assert!(ul, "Underline on first word should produce an underlined run");

        let (ul, _) = check_run_properties(TEXT, RangedStyle::Underline(second_word()));
        assert!(
            ul,
            "Underline on second word should produce an underlined run"
        );
    }

    #[test]
    fn test_font_size_works_on_both_ranges() {
        let (_, sized) = check_run_properties(TEXT, RangedStyle::Size(first_word(), 32.0));
        assert!(sized, "FontSize on first word should produce a 32px run");

        let (_, sized) = check_run_properties(TEXT, RangedStyle::Size(second_word(), 32.0));
        assert!(sized, "FontSize on second word should produce a 32px run");
    }
}
