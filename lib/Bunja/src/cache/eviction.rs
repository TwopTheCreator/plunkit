use std::collections::{HashMap, VecDeque};

pub trait EvictionPolicy {
    fn record_access(&mut self, key: &str, size: u64);
    fn get_eviction_candidates(&self, space_needed: u64) -> Vec<String>;
    fn remove(&mut self, key: &str);
    fn clear(&mut self);
    fn len(&self) -> usize;
    fn total_size(&self) -> u64;
}

#[derive(Debug)]
pub struct LruEviction {
    access_order: VecDeque<String>,
    sizes: HashMap<String, u64>,
}

impl LruEviction {
    pub fn new() -> Self {
        Self {
            access_order: VecDeque::new(),
            sizes: HashMap::new(),
        }
    }
}

impl EvictionPolicy for LruEviction {
    fn record_access(&mut self, key: &str, size: u64) {
        self.access_order.retain(|k| k != key);

        self.access_order.push_back(key.to_string());
        self.sizes.insert(key.to_string(), size);
    }

    fn get_eviction_candidates(&self, space_needed: u64) -> Vec<String> {
        let mut candidates = vec![];
        let mut freed_space = 0u64;

        for key in self.access_order.iter() {
            if freed_space >= space_needed {
                break;
            }

            if let Some(&size) = self.sizes.get(key) {
                candidates.push(key.clone());
                freed_space += size;
            }
        }

        candidates
    }

    fn remove(&mut self, key: &str) {
        self.access_order.retain(|k| k != key);
        self.sizes.remove(key);
    }

    fn clear(&mut self) {
        self.access_order.clear();
        self.sizes.clear();
    }

    fn len(&self) -> usize {
        self.sizes.len()
    }

    fn total_size(&self) -> u64 {
        self.sizes.values().sum()
    }
}
