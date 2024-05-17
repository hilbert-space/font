//! Layout features.

pub use opentype::layout::{Coverage, Language, Script};
pub use opentype::truetype::GlyphID;

use std::collections::BTreeMap;
use std::io::Result;

use opentype::layout::{Directory, Feature};

use crate::formats::opentype::cache::Cache;
use crate::formats::opentype::characters::ReverseMapping;
use crate::CharacterID;

/// Layout features.
pub type Features = BTreeMap<Type, Value>;

/// A type.
pub type Type = Feature;

/// A value.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Value {
    /// The scripts and languages.
    pub scripts: BTreeMap<Script, BTreeMap<Option<Language>, Vec<Vec<CharacterID>>>>,
}

trait Characters {
    #[inline]
    fn characters(&self, _: &ReverseMapping) -> Vec<Vec<CharacterID>> {
        Default::default()
    }
}

pub(crate) fn read<T: crate::Read>(cache: &mut Cache<T>) -> Result<Features> {
    let mut values = Features::default();
    let mapping = cache.reverse_mapping()?.clone();
    if let Some(table) = cache.try_glyph_positioning()? {
        populate(&mut values, &table.borrow(), &mapping);
    }
    if let Some(table) = cache.try_glyph_substitution()? {
        populate(&mut values, &table.borrow(), &mapping);
    }
    Ok(values)
}

fn populate<T>(values: &mut Features, table: &Directory<T>, mapping: &ReverseMapping)
where
    T: Characters,
{
    for (i, header) in table.scripts.headers.iter().enumerate() {
        let script = Script::from_tag(&header.tag);
        if let Some(record) = table.scripts.records[i].default_language.as_ref() {
            for index in record.feature_indices.iter() {
                if let (Some(header), Some(record)) = (
                    table.features.headers.get(*index as usize),
                    table.features.records.get(*index as usize),
                ) {
                    let feature = Feature::from_tag(&header.tag);
                    let mut characters = record
                        .lookup_indices
                        .iter()
                        .filter_map(|index| table.lookups.records.get(*index as usize))
                        .flat_map(|record| {
                            record
                                .tables
                                .iter()
                                .flat_map(|table| table.characters(mapping))
                        })
                        .collect::<Vec<_>>();
                    characters.sort();
                    values
                        .entry(feature)
                        .or_default()
                        .scripts
                        .entry(script)
                        .or_default()
                        .insert(None, characters);
                }
            }
        }
        for (j, header) in table.scripts.records[i].language_headers.iter().enumerate() {
            let language = Language::from_tag(&header.tag);
            let record = &table.scripts.records[i].language_records[j];
            for index in record.feature_indices.iter() {
                if let (Some(header), Some(record)) = (
                    table.features.headers.get(*index as usize),
                    table.features.records.get(*index as usize),
                ) {
                    let feature = Feature::from_tag(&header.tag);
                    let mut characters = record
                        .lookup_indices
                        .iter()
                        .filter_map(|index| table.lookups.records.get(*index as usize))
                        .flat_map(|record| {
                            record
                                .tables
                                .iter()
                                .flat_map(|table| table.characters(mapping))
                        })
                        .collect::<Vec<_>>();
                    characters.sort();
                    values
                        .entry(feature)
                        .or_default()
                        .scripts
                        .entry(script)
                        .or_default()
                        .insert(Some(language), characters);
                }
            }
        }
    }
}

impl Characters for opentype::tables::glyph_positioning::Type {}

impl Characters for opentype::tables::glyph_substitution::Type {
    fn characters(&self, mapping: &ReverseMapping) -> Vec<Vec<CharacterID>> {
        use opentype::layout::Context;
        use opentype::tables::glyph_substitution::{SingleSubstitution, Type};

        let mut values = Vec::default();
        match self {
            Type::SingleSubstitution(SingleSubstitution::Format1(value)) => {
                values.extend(
                    iterate(&value.coverage)
                        .filter_map(|glyph_id| mapping.get(glyph_id))
                        .map(expand),
                );
            }
            Type::SingleSubstitution(SingleSubstitution::Format2(value)) => {
                values.extend(
                    iterate(&value.coverage)
                        .filter_map(|glyph_id| mapping.get(glyph_id))
                        .map(expand),
                );
            }
            Type::MultipleSubstitution(value) => {
                values.extend(
                    iterate(&value.coverage)
                        .filter_map(|glyph_id| mapping.get(glyph_id))
                        .map(expand),
                );
            }
            Type::AlternateSubstitution(value) => {
                values.extend(
                    iterate(&value.coverage)
                        .filter_map(|glyph_id| mapping.get(glyph_id))
                        .map(expand),
                );
            }
            Type::LigatureSubstitution(value) => {
                values.extend(iterate(&value.coverage).zip(&value.rules).flat_map(
                    |(glyph_id, rule)| {
                        rule.records.iter().filter_map(move |record| {
                            let mut value = Vec::with_capacity(record.glyph_count as usize);
                            value.push(mapping.get(glyph_id)?);
                            for glyph_id in &record.glyph_ids {
                                value.push(mapping.get(*glyph_id)?);
                            }
                            Some(value)
                        })
                    },
                ));
            }
            Type::ContextualSubstitution(Context::Format1(value)) => {
                values.extend(iterate(&value.coverage).zip(&value.rules).flat_map(
                    |(glyph_id, rule)| {
                        rule.records.iter().filter_map(move |record| {
                            let mut value = Vec::with_capacity(record.glyph_count as usize);
                            value.push(mapping.get(glyph_id)?);
                            for glyph_id in &record.glyph_ids {
                                value.push(mapping.get(*glyph_id)?);
                            }
                            Some(value)
                        })
                    },
                ));
            }
            _ => {}
        }
        values
    }
}

#[inline]
fn expand<T>(value: T) -> Vec<T> {
    vec![value]
}

fn iterate(_: &Coverage) -> impl Iterator<Item = GlyphID> {
    vec![].into_iter()
}
