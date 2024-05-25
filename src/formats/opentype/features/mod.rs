//! Layout features.

mod graph;
mod sample;
mod transform;

pub use opentype::layout::{Language, Script};

pub use sample::{Component, Sample};

use std::collections::{BTreeMap, BTreeSet};
use std::io::Result;

use opentype::layout::{Directory, Feature};

use crate::formats::opentype::cache::Cache;
use crate::formats::opentype::features::graph::{Graph, Table};
use crate::formats::opentype::features::transform::Transform;

/// Layout features.
pub type Features = BTreeMap<Type, Value>;

/// A type.
pub type Type = Feature;

/// A value.
pub type Value = BTreeMap<Script, BTreeMap<Language, BTreeSet<Sample>>>;

pub(crate) fn read<T: crate::Read>(cache: &mut Cache<T>) -> Result<Features> {
    let mut values = Default::default();
    if let Some(table) = cache.try_glyph_positioning()? {
        populate(&mut values, &table.borrow());
    }
    if let Some(table) = cache.try_glyph_substitution()? {
        populate(&mut values, &table.borrow());
    }
    let mapping = cache.reverse_mapping()?.clone();
    Ok(values.transform(&mapping, ()))
}

#[allow(clippy::type_complexity)]
fn populate<T>(
    values: &mut BTreeMap<Feature, BTreeMap<Script, BTreeMap<Language, Graph>>>,
    directory: &Directory<T>,
) where
    T: Table,
{
    for (i, header) in directory.scripts.headers.iter().enumerate() {
        let script = Script::from_tag(&header.tag);
        if let Some(record) = directory.scripts.records[i].default_language.as_ref() {
            for index in record.feature_indices.iter().cloned().map(usize::from) {
                if let (Some(header), Some(record)) = (
                    directory.features.headers.get(index),
                    directory.features.records.get(index),
                ) {
                    let feature = Feature::from_tag(&header.tag);
                    let glyphs = record
                        .lookup_indices
                        .iter()
                        .cloned()
                        .filter_map(|index| directory.lookups.records.get(index as usize))
                        .flat_map(|record| {
                            record
                                .tables
                                .iter()
                                .flat_map(|table| table.extract(directory))
                        })
                        .collect::<BTreeMap<_, _>>();
                    values
                        .entry(feature)
                        .or_default()
                        .entry(script)
                        .or_default()
                        .insert(Language::Default, glyphs);
                }
            }
        }
        for (j, header) in directory.scripts.records[i]
            .language_headers
            .iter()
            .enumerate()
        {
            let language = Language::from_tag(&header.tag);
            let record = &directory.scripts.records[i].language_records[j];
            for index in record.feature_indices.iter().cloned().map(usize::from) {
                if let (Some(header), Some(record)) = (
                    directory.features.headers.get(index),
                    directory.features.records.get(index),
                ) {
                    let feature = Feature::from_tag(&header.tag);
                    let glyphs = record
                        .lookup_indices
                        .iter()
                        .cloned()
                        .filter_map(|index| directory.lookups.records.get(index as usize))
                        .flat_map(|record| {
                            record
                                .tables
                                .iter()
                                .flat_map(|table| table.extract(directory))
                        })
                        .collect::<BTreeMap<_, _>>();
                    values
                        .entry(feature)
                        .or_default()
                        .entry(script)
                        .or_default()
                        .insert(language, glyphs);
                }
            }
        }
    }
}
