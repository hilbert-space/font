use std::collections::{BTreeMap, BTreeSet};

use opentype::layout::{Feature, Language, Script};
use opentype::truetype::GlyphID;

use super::{Features, Value};
use crate::formats::opentype::characters::{Character, ReverseMapping as Mapping};
use crate::formats::opentype::features::glyphs::Glyph;

pub trait Characters<'l> {
    type Target;
    type Parameter: 'l;

    fn characters(self, _: &Mapping, _: Self::Parameter) -> Self::Target;
}

type Glyphs = BTreeSet<Vec<Glyph>>;

impl<'l> Characters<'l> for &BTreeMap<Feature, BTreeMap<Script, BTreeMap<Language, Glyphs>>> {
    type Target = Features;
    type Parameter = ();

    fn characters(self, mapping: &Mapping, _: Self::Parameter) -> Self::Target {
        self.iter()
            .map(|(key, value)| (*key, value.characters(mapping, ())))
            .collect()
    }
}

impl<'l> Characters<'l> for &BTreeMap<Script, BTreeMap<Language, Glyphs>> {
    type Target = Value;
    type Parameter = ();

    fn characters(self, mapping: &Mapping, _: Self::Parameter) -> Self::Target {
        self.iter()
            .map(|(key, value)| (*key, value.characters(mapping, ())))
            .collect()
    }
}

impl<'l> Characters<'l> for &BTreeMap<Language, Glyphs> {
    type Target = BTreeMap<Language, BTreeSet<Vec<Character>>>;
    type Parameter = ();

    fn characters(self, mapping: &Mapping, _: Self::Parameter) -> Self::Target {
        self.iter()
            .map(|(key, value)| (*key, value.characters(mapping, ())))
            .collect()
    }
}

impl<'l> Characters<'l> for &Glyphs {
    type Target = BTreeSet<Vec<Character>>;
    type Parameter = ();

    fn characters(self, mapping: &Mapping, _: Self::Parameter) -> Self::Target {
        self.iter()
            .filter_map(|value| value.characters(mapping, self))
            .collect()
    }
}

impl<'l> Characters<'l> for &[Glyph] {
    type Target = Option<Vec<Character>>;
    type Parameter = &'l Glyphs;

    fn characters(self, mapping: &Mapping, glyphs: Self::Parameter) -> Self::Target {
        self.iter()
            .map(|value| value.characters(mapping, glyphs))
            .collect()
    }
}

impl<'l> Characters<'l> for &Glyph {
    type Target = Option<Character>;
    type Parameter = &'l Glyphs;

    fn characters(self, mapping: &Mapping, glyphs: Self::Parameter) -> Self::Target {
        match self {
            Glyph::Scalar(value) => mapping.get(*value).map(Character::Scalar),
            Glyph::Range(start, end) => compress(*start..=*end, mapping, glyphs),
            Glyph::Ranges(value) => compress(
                value.iter().flat_map(|value| value.0..=value.1),
                mapping,
                glyphs,
            ),
            Glyph::List(value) => compress(value.iter().cloned(), mapping, glyphs),
        }
    }
}

fn compress<T>(values: T, mapping: &Mapping, _: &Glyphs) -> Option<Character>
where
    T: Iterator<Item = GlyphID>,
{
    let mut values = values
        .filter_map(|glyph_id| mapping.get(glyph_id))
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    if values.is_empty() {
        return None;
    }
    if values.len() == 1 {
        return Some(Character::Scalar(values[0]));
    }
    let mut ranges = Vec::new();
    let (mut start, mut end) = (values[0], values[0]);
    let mut iterator = values.iter().skip(1).cloned();
    loop {
        match iterator.next() {
            Some(next) => {
                if end as usize + 1 == next as usize {
                    end = next;
                    continue;
                }
                ranges.push((start, end));
                start = next;
                end = next;
            }
            _ => {
                ranges.push((start, end));
                break;
            }
        }
    }
    if ranges.len() == 1 {
        if ranges[0].0 == ranges[0].1 {
            return Some(Character::Scalar(ranges[0].0));
        }
        return Some(Character::Range(ranges[0].0, ranges[0].1));
    }
    if 2 * ranges.len() < values.len() {
        Some(Character::Ranges(ranges))
    } else {
        Some(Character::List(values))
    }
}
