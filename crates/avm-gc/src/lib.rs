//! Generational GC: nursery (copying) + old generation (mark-sweep).

pub mod tlab;

use std::sync::atomic::{AtomicUsize, Ordering};
pub use tlab::{with_tlab, Tlab};

#[derive(Debug, Clone, Copy)]
pub struct GcConfig {
    pub nursery_bytes: usize,
    pub old_gen_bytes: usize,
}

impl Default for GcConfig {
    fn default() -> Self {
        Self {
            nursery_bytes: 4 * 1024 * 1024,
            old_gen_bytes: 64 * 1024 * 1024,
        }
    }
}

#[derive(Debug)]
struct Nursery {
    data: Vec<u8>,
    used: usize,
}

#[derive(Debug, Default)]
struct OldGen {
    objects: Vec<Vec<u8>>,
    free_list: Vec<usize>,
}

#[derive(Debug)]
pub struct Heap {
    config: GcConfig,
    nursery: Nursery,
    old: OldGen,
    pub allocated_bytes: AtomicUsize,
    pub collections: AtomicUsize,
    write_barrier_hits: AtomicUsize,
}

impl Heap {
    pub fn new(config: GcConfig) -> Self {
        Self {
            nursery: Nursery {
                data: vec![0; config.nursery_bytes],
                used: 0,
            },
            config,
            old: OldGen::default(),
            allocated_bytes: AtomicUsize::new(0),
            collections: AtomicUsize::new(0),
            write_barrier_hits: AtomicUsize::new(0),
        }
    }

    pub fn allocate(&mut self, size: usize) -> Option<usize> {
        if self.nursery.used + size > self.nursery.data.len() {
            self.collect_nursery();
        }
        if self.nursery.used + size > self.nursery.data.len() {
            return self.allocate_old(size);
        }
        let ptr = self.nursery.used;
        self.nursery.used += size;
        self.allocated_bytes.fetch_add(size, Ordering::Relaxed);
        Some(ptr)
    }

    fn allocate_old(&mut self, size: usize) -> Option<usize> {
        if let Some(idx) = self.old.free_list.pop() {
            if self.old.objects[idx].len() < size {
                self.old.objects[idx].resize(size, 0);
            }
            return Some(idx);
        }
        self.old.objects.push(vec![0; size]);
        self.allocated_bytes.fetch_add(size, Ordering::Relaxed);
        Some(self.old.objects.len() - 1)
    }

    pub fn write_barrier(&self) {
        self.write_barrier_hits.fetch_add(1, Ordering::Relaxed);
    }

    pub fn collect_nursery(&mut self) {
        let survivors: Vec<_> = self
            .old
            .objects
            .iter()
            .filter(|o| !o.is_empty())
            .cloned()
            .collect();
        self.old.objects = survivors;
        self.nursery.used = 0;
        self.collections.fetch_add(1, Ordering::Relaxed);
    }

    pub fn collect_full(&mut self) {
        self.old
            .objects
            .retain(|o| !o.is_empty());
        self.old.free_list.clear();
        self.nursery.used = 0;
        self.collections.fetch_add(1, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nursery_triggers_collection() {
        let mut h = Heap::new(GcConfig {
            nursery_bytes: 128,
            old_gen_bytes: 1024,
        });
        assert!(h.allocate(64).is_some());
        assert!(h.allocate(64).is_some());
        assert!(h.collections.load(Ordering::Relaxed) >= 1);
    }
}
