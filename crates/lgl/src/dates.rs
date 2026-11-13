//! A printed weekday that does not match its date ("Tuesday, 14 May 2025"
//! when 14 May 2025 is a Wednesday).

use std::collections::{BTreeMap, BTreeSet};

use crate::diag::{self, Diagnostic, Severity};
use crate::geom::{Geometry, ItemKind};
use crate::world::LabWorld;

pub const WRONG_WEEKDAY: &str = "L095.wrong-weekday";

const DAYS: &[(&str, u32)] = &[
    ("monday", 0),
    ("tuesday", 1),
    ("wednesday", 2),
    ("thursday", 3),
    ("friday", 4),
    ("saturday", 5),
    ("sunday", 6),
    ("mon", 0),
    ("tue", 1),
    ("tues", 1),
    ("wed", 2),
    ("thu", 3),
    ("thur", 3),
    ("thurs", 3),
    ("fri", 4),
    ("sat", 5),
    ("sun", 6),
];

const WEEKDAY_NAMES: [&str; 7] =
    ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];

const MONTHS: [&str; 12] = [
    "january",
    "february",
    "march",
    "april",
    "may",
    "june",
    "july",
    "august",
    "september",
    "october",
    "november",
    "december",
];

/// A word of a printed line, with the text item it comes from.
struct Word {
    text: String,
    start: usize,
    end: usize,
    item: usize,
}

/// A weekday printed with a date: the weekday, the date and the year if one
/// is printed.
struct Mention {
    shown: String,
    weekday: u32,
    day: u32,
    month: u32,
    year: Option<i32>,
    item: usize,
}

pub fn run(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mentions = mentions(geometry);
    // A date printed without a year takes the document's year when every
    // date that names a year names the same one.
    let years: BTreeSet<i32> = mentions.iter().filter_map(|m| m.year).collect();
    let only_year = (years.len() == 1).then(|| *years.first().unwrap());
    let mut out = Vec::new();
    for mention in &mentions {
        let Some(year) = mention.year.or(only_year) else { continue };
        let Some(actual) = weekday(year, mention.month, mention.day) else { continue };
        if actual == mention.weekday {
            continue;
        }
        let item = &geometry.items[mention.item];
        let month = MONTHS[mention.month as usize - 1];
        let month = format!("{}{}", month[..1].to_uppercase(), &month[1..]);
        let mut diagnostic = Diagnostic::new(
            WRONG_WEEKDAY,
            Severity::Warning,
            format!(
                "page {} prints \"{}\", but {} {month} {year} is a {}",
                item.page + 1,
                mention.shown,
                mention.day,
                WEEKDAY_NAMES[actual as usize]
            ),
        );
        diagnostic.loc =
            diag::resolve_span(world, geometry.span_for(item.span, item.elem));
        diagnostic.page = Some(item.page + 1);
        diagnostic.bbox = item.visible().map(|b| b.rounded());
        diagnostic.hints.push(
            "correct the weekday or the date so that they agree, and check the other \
             dates of the document against the same calendar"
                .into(),
        );
        out.push(diagnostic);
    }
    out
}

/// The weekday (0 is Monday) of a date, if the date exists.
fn weekday(year: i32, month: u32, day: u32) -> Option<u32> {
    if !(1900..=2200).contains(&year) || !(1..=12).contains(&month) {
        return None;
    }
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    let lengths =
        [31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    if day == 0 || day > lengths[month as usize - 1] {
        return None;
    }
    // Sakamoto's method, with 0 for Sunday.
    const OFFSETS: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let y = if month < 3 { year - 1 } else { year };
    let sunday_based =
        (y + y / 4 - y / 100 + y / 400 + OFFSETS[month as usize - 1] + day as i32) % 7;
    Some(((sunday_based + 6) % 7) as u32)
}

/// Every weekday printed next to a date, line by line.
fn mentions(geometry: &Geometry) -> Vec<Mention> {
    let mut rows: BTreeMap<(usize, i64), Vec<usize>> = BTreeMap::new();
    for (index, item) in geometry.items.iter().enumerate() {
        if item.kind == ItemKind::Text
            && item.text.as_deref().is_some_and(|t| !t.trim().is_empty())
        {
            rows.entry((item.page, (item.bbox.y1 * 2.0).round() as i64))
                .or_default()
                .push(index);
        }
    }
    let mut out = Vec::new();
    for mut row in rows.into_values() {
        row.sort_by(|&a, &b| {
            geometry.items[a].bbox.x0.total_cmp(&geometry.items[b].bbox.x0)
        });
        let mut line = String::new();
        let mut words = Vec::new();
        for index in row {
            if !line.is_empty() {
                line.push(' ');
            }
            let base = line.len();
            let text = geometry.items[index].text.as_deref().unwrap_or_default();
            line.push_str(&text.replace('\u{a0}', " "));
            let mut start = None;
            for (offset, c) in
                line[base..].char_indices().chain([(line.len() - base, ' ')])
            {
                match (c.is_alphanumeric(), start) {
                    (true, None) => start = Some(base + offset),
                    (false, Some(s)) => {
                        words.push(Word {
                            text: line[s..base + offset].to_lowercase(),
                            start: s,
                            end: base + offset,
                            item: index,
                        });
                        start = None;
                    }
                    _ => {}
                }
            }
        }
        scan(&line, &words, &mut out);
    }
    out
}

/// Finds "Tuesday, 14 May 2025", "Tue 14 May", "Tuesday, May 14, 2025" and
/// the like among the words of a line.
fn scan(line: &str, words: &[Word], out: &mut Vec<Mention>) {
    // Only spaces, commas and full stops may stand between the parts.
    let joined = |a: &Word, b: &Word| {
        line[a.end..b.start]
            .chars()
            .all(|c| c.is_whitespace() || c == ',' || c == '.')
    };
    let day_of = |word: &Word| {
        let digits = word.text.trim_end_matches(['s', 't', 'n', 'd', 'r', 'h']);
        (digits.len() <= 2
            && !digits.is_empty()
            && digits.chars().all(|c| c.is_ascii_digit()))
        .then(|| digits.parse::<u32>().ok())
        .flatten()
    };
    let month_of = |word: &Word| {
        MONTHS.iter().position(|m| {
            word.text == *m || (word.text.len() >= 3 && m.starts_with(word.text.as_str()))
        })
    };
    let year_of = |word: &Word| {
        (word.text.len() == 4 && word.text.chars().all(|c| c.is_ascii_digit()))
            .then(|| word.text.parse::<i32>().ok())
            .flatten()
    };
    for i in 0..words.len() {
        let Some(&(_, weekday)) = DAYS.iter().find(|(name, _)| *name == words[i].text)
        else {
            continue;
        };
        let next = |k: usize| words.get(k).filter(|w| joined(&words[k - 1], w));
        let parsed = match (next(i + 1), next(i + 2)) {
            (Some(a), Some(b)) if day_of(a).is_some() && month_of(b).is_some() => {
                Some((day_of(a).unwrap(), month_of(b).unwrap(), i + 3))
            }
            (Some(a), Some(b)) if month_of(a).is_some() && day_of(b).is_some() => {
                Some((day_of(b).unwrap(), month_of(a).unwrap(), i + 3))
            }
            _ => None,
        };
        let Some((day, month, after)) = parsed else { continue };
        let year = next(after).and_then(year_of);
        let last = if year.is_some() { after } else { after - 1 };
        out.push(Mention {
            shown: line[words[i].start..words[last].end].to_string(),
            weekday,
            day,
            month: month as u32 + 1,
            year,
            item: words[i].item,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::weekday;

    #[test]
    fn weekdays_are_computed() {
        assert_eq!(weekday(2025, 5, 14), Some(2));
        assert_eq!(weekday(2025, 5, 13), Some(1));
        assert_eq!(weekday(2026, 6, 14), Some(6));
        assert_eq!(weekday(2024, 2, 29), Some(3));
        assert_eq!(weekday(2025, 2, 29), None);
        assert_eq!(weekday(2000, 1, 1), Some(5));
    }
}
