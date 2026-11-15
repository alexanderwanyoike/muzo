use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;

use super::Paragraph;

struct Cluster {
    bytes: Range<usize>,
    left: f32,
    right: f32,
}

pub(super) fn visible_text(paragraph: &Paragraph, text: &str, width: f32) -> (String, f32) {
    let mut clusters = clusters(paragraph, text);
    clusters.sort_unstable_by(|a, b| a.left.total_cmp(&b.left));
    let mut visible: Vec<_> = clusters
        .into_iter()
        .take_while(|cluster| cluster.right <= width)
        .collect();
    let offset = visible
        .iter()
        .map(|cluster| cluster.right)
        .fold(0.0, f32::max);
    visible.sort_unstable_by_key(|cluster| cluster.bytes.start);
    let label = visible
        .into_iter()
        .map(|cluster| &text[cluster.bytes])
        .collect();
    (label, offset)
}

fn clusters(paragraph: &Paragraph, text: &str) -> Vec<Cluster> {
    let boundaries: Vec<_> = text
        .grapheme_indices(true)
        .map(|(start, _)| start)
        .chain([text.len()])
        .collect();
    let mut glyphs: Vec<_> = paragraph
        .buffer()
        .layout_runs()
        .flat_map(|line| line.glyphs)
        .filter(|glyph| glyph.start < glyph.end)
        .map(|glyph| {
            let first = boundaries.partition_point(|&start| start <= glyph.start) - 1;
            let last = boundaries.partition_point(|&start| start < glyph.end);
            Cluster {
                bytes: boundaries[first]..boundaries[last],
                left: glyph.x,
                right: glyph.x + glyph.w,
            }
        })
        .collect();
    glyphs.sort_unstable_by_key(|glyph| glyph.bytes.start);

    // Marks can share a grapheme, and ligatures can cover several.
    let mut clusters: Vec<Cluster> = Vec::new();
    for glyph in glyphs {
        if let Some(previous) = clusters
            .last_mut()
            .filter(|previous| glyph.bytes.start < previous.bytes.end)
        {
            previous.bytes.end = previous.bytes.end.max(glyph.bytes.end);
            previous.left = previous.left.min(glyph.left);
            previous.right = previous.right.max(glyph.right);
        } else {
            clusters.push(glyph);
        }
    }
    clusters
}
