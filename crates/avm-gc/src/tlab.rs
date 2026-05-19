//! Thread-local allocation buffer (TLAB).

use std::cell::RefCell;

thread_local! {
    static TLAB: RefCell<Tlab> = RefCell::new(Tlab::new(4096));
}

#[derive(Debug)]
pub struct Tlab {
    buf: Vec<u8>,
    pos: usize,
    capacity: usize,
}

impl Tlab {
    pub fn new(capacity: usize) -> Self {
        Self {
            buf: vec![0; capacity],
            pos: 0,
            capacity,
        }
    }

    pub fn allocate(&mut self, size: usize) -> Option<usize> {
        if self.pos + size > self.capacity {
            return None;
        }
        let off = self.pos;
        self.pos += size;
        Some(off)
    }

    pub fn reset(&mut self) {
        self.pos = 0;
    }
}

pub fn with_tlab<F, R>(size: usize, f: F) -> R
where
    F: FnOnce(&mut Tlab) -> R,
{
    TLAB.with(|t| {
        let mut tlab = t.borrow_mut();
        if tlab.capacity < size {
            *tlab = Tlab::new(size);
        }
        f(&mut tlab)
    })
}
