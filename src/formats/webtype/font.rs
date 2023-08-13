use std::cell::RefCell;
use std::rc::Rc;

use typeface::Tape;

use crate::formats::opentype::cache::Cache;
use crate::formats::opentype::font::{
    read_axes, read_characters, read_features, read_metrics, read_names,
};
use crate::Result;

pub struct Font<T> {
    cache: Rc<RefCell<Cache<T>>>,
}

impl<T: Tape> crate::font::Case for Font<T> {
    #[inline]
    fn axes(&mut self) -> Result<crate::Axes> {
        read_axes(&mut self.cache.borrow_mut())
    }

    #[inline]
    fn characters(&mut self) -> Result<crate::Characters> {
        read_characters(&mut self.cache.borrow_mut())
    }

    #[inline]
    fn features(&mut self) -> Result<crate::Features> {
        read_features(&mut self.cache.borrow_mut())
    }

    #[inline]
    fn metrics(&mut self) -> Result<crate::Metrics> {
        read_metrics(&mut self.cache.borrow_mut())
    }

    #[inline]
    fn names(&mut self) -> Result<crate::Names> {
        read_names(&mut self.cache.borrow_mut())
    }

    #[inline]
    fn draw(&mut self, _: char) -> Result<Option<crate::Glyph>> {
        error!("working with glyphs is not supported yet")
    }
}

pub fn read<T>(tape: Rc<RefCell<T>>, backend: webtype::Font) -> Result<Vec<Font<T>>>
where
    T: Tape,
{
    let cache = Rc::new(RefCell::new(Cache::new(tape, backend)));
    Ok(vec![Font { cache }])
}
