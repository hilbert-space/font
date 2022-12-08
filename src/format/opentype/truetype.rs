use std::rc::Rc;
use truetype::glyph_data::{self, CompositeDescription, GlyphData, SimpleDescription};

use super::mapping::Mapping;
use super::metrics::Metrics;
use crate::builder::Builder;
use crate::case::Case;
use crate::glyph::Glyph;
use crate::{Offset, Result};

pub struct TrueType {
    glyph_data: Rc<GlyphData>,
    metrics: Rc<Metrics>,
    mapping: Rc<Mapping>,
}

macro_rules! expect(($condition:expr) => (if !$condition { reject!(); }));
macro_rules! reject(() => (raise!("found a malformed glyph")));

impl TrueType {
    #[inline]
    pub fn new(glyph_data: Rc<GlyphData>, metrics: Rc<Metrics>, mapping: Rc<Mapping>) -> Self {
        TrueType {
            glyph_data: glyph_data,
            metrics: metrics,
            mapping: mapping,
        }
    }

    fn draw_glyph(&self, builder: &mut Builder, glyph: &glyph_data::Glyph) -> Result<()> {
        use truetype::glyph_data::Description::*;

        match &glyph.description {
            &Simple(ref description) => draw_simple(builder, description),
            &Composite(ref description) => draw_composite(self, builder, description),
        }
    }
}

impl Case for TrueType {
    fn draw(&self, glyph: char) -> Result<Option<Glyph>> {
        let mut builder = Builder::new();
        let glyph_index = match self.mapping.find(glyph) {
            Some(glyph_index) => glyph_index,
            _ => return Ok(None),
        };
        let glyph = match self.glyph_data.get(glyph_index) {
            Some(glyph) => glyph,
            _ => reject!(),
        };
        builder.set_horizontal_metrics(self.metrics.get(glyph_index));
        if let &Some(ref glyph) = glyph {
            self.draw_glyph(&mut builder, glyph)?;
            builder.set_bounding_box(glyph.min_x, glyph.min_y, glyph.max_x, glyph.max_y);
        }
        Ok(Some(builder.into()))
    }
}

fn draw_simple(builder: &mut Builder, description: &SimpleDescription) -> Result<()> {
    let &SimpleDescription {
        ref end_points,
        ref flags,
        ref x,
        ref y,
        ..
    } = description;
    let point_count = flags.len();
    expect!(point_count > 0);
    expect!(point_count == x.len());
    expect!(point_count == y.len());
    let mut i = 0;
    let mut sum = Offset::default();
    for k in end_points.iter().map(|&k| k as usize) {
        expect!(i < k);
        expect!(k < point_count);
        let start = (x[i], y[i]).into();
        let mut control = if flags[i].is_on_curve() {
            None
        } else {
            Some(start)
        };
        let mut sum_delta = start;
        let mut offset = Offset::default();
        for j in (i + 1)..=k {
            let current = (x[j], y[j]).into();
            sum_delta += current;
            if flags[j].is_on_curve() {
                match control.take() {
                    Some(control) => {
                        builder.add_quadratic(control, current);
                        offset += control + current;
                    }
                    _ => {
                        builder.add_linear(current);
                        offset += current;
                    }
                }
            } else {
                match &mut control {
                    &mut Some(ref mut control) => {
                        let current = current / 2.0;
                        builder.add_quadratic(*control, current);
                        offset += *control + current;
                        *control = current;
                    }
                    control @ &mut None => {
                        *control = Some(current);
                    }
                }
            }
        }
        let position = match (flags[i].is_on_curve(), control) {
            (false, None) => {
                let current = -offset;
                let control = -offset - start;
                builder.add_quadratic(control, current);
                offset += control + current;
                Offset::default()
            }
            (false, Some(control)) => {
                {
                    let current = -offset / 2.0;
                    builder.add_quadratic(control, current);
                    offset += control + current;
                }
                {
                    let current = -offset;
                    let control = -offset - start;
                    builder.add_quadratic(control, current);
                    offset += control + current;
                }
                Offset::default()
            }
            (true, None) => {
                let current = -offset;
                builder.add_linear(current);
                offset += current;
                sum + start
            }
            (true, Some(control)) => {
                let current = -offset - control;
                builder.add_quadratic(control, current);
                offset += control + current;
                sum + start
            }
        };
        debug_assert_eq!(offset, Offset::default());
        builder.move_absolute(position);
        builder.flush();
        sum += sum_delta;
        i = k + 1;
    }
    Ok(())
}

fn draw_composite(
    case: &TrueType,
    builder: &mut Builder,
    description: &CompositeDescription,
) -> Result<()> {
    use truetype::glyph_data::{Arguments, Options};

    for component in description.components.iter() {
        let glyph_index = component.glyph_index as usize;
        let offset = match &component.arguments {
            &Arguments::Offsets(x, y) => Offset::from((x, y)),
            &Arguments::Indices(..) => unimplemented!(),
        };
        match &component.options {
            &Options::None => {}
            &Options::Scalar(..) => unimplemented!(),
            &Options::Vector(..) => unimplemented!(),
            &Options::Matrix(..) => unimplemented!(),
        }
        let glyph = match case.glyph_data.get(glyph_index) {
            Some(&Some(ref glyph)) => glyph,
            Some(&None) => continue,
            _ => reject!(),
        };
        if component.flags.should_use_metrics() {
            builder.set_horizontal_metrics(case.metrics.get(glyph_index));
        }
        builder.nest(offset, |builder| case.draw_glyph(builder, glyph))?;
    }
    Ok(())
}
