use std::mem::MaybeUninit;
use std::sync::Arc;

use crate::{FlError, Value};

mod access;
mod add_and_join;
mod construct;
mod destruct;

#[derive(Debug)]
pub struct FlList {
    buckets: Arc<[Bucket]>,
}

const BASE_SIZE: usize = 8;
const FACTOR: usize = 4;

#[derive(Debug)]
struct Bucket {
    values: Arc<[Value]>,
}

impl Clone for FlList {
    fn clone(&self) -> Self {
        Self {
            buckets: Arc::clone(&self.buckets),
        }
    }
}

impl FlList {
    fn check_not_empty(&self, message: &str) -> Result<(), FlError> {
        if self.buckets.len() > 0 {
            Ok(())
        } else {
            Err(FlError::new(message))
        }
    }

    fn needs_reshaping(number_of_buckets: usize, len: usize) -> bool {
        (1 << number_of_buckets) > len
    }

    /// Create a vec with bucket sizes for a list of `size`. Starting at both ends
    /// with size 3/4 * BASE_SIZE and increasing by FACTOR to the middle.
    fn compute_bucket_sizes(size: usize) -> Vec<usize> {
        let mut num_buckets: usize = 2;
        let mut bucket_size: usize = 3 * BASE_SIZE / 4;
        let mut size_in_buckets: usize = 2 * bucket_size;
        while size_in_buckets < size {
            bucket_size *= FACTOR;
            size_in_buckets += 2 * bucket_size;
            num_buckets += 2;
        }
        num_buckets -= 1;
        let mut bucket_sizes: Vec<usize> = vec![0; num_buckets];
        bucket_size = BASE_SIZE;
        let mut rest = size;
        let mut i: usize = 0;
        let mut j: usize = num_buckets - 1;
        while i < j {
            bucket_sizes[i] = bucket_size / 2;
            bucket_sizes[j] = bucket_size / 2;
            rest -= bucket_sizes[i] + bucket_sizes[j];
            bucket_size *= FACTOR;
            i += 1;
            j -= 1;
        }
        bucket_sizes[i] = rest;

        bucket_sizes
    }

    /// Build a [`Bucket`] from `items`, allocating its backing `Arc<[Value]>` directly
    /// via [`Arc::new_uninit_slice`] so no intermediate `Vec` is needed.
    ///
    /// If `items` yields fewer
    /// values than its reported length, remaining slots are filled with [`Value::Nil`]
    /// rather than panicking.
    fn fill_bucket(mut items: impl ExactSizeIterator<Item = Value>) -> Bucket {
        let mut u_values: Arc<[MaybeUninit<Value>]> = Arc::new_uninit_slice(items.len());
        let slots = Arc::get_mut(&mut u_values).unwrap();
        for slot in slots.iter_mut() {
            // The "or" case should never happen, but this way we avoid a panic,
            // which on unwind will not free the the uninitialized values.
            slot.write(items.next().unwrap_or(Value::Nil));
        }
        Bucket {
            values: unsafe { u_values.assume_init() },
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn create_vec(from: usize, to: usize) -> Vec<Value> {
        let mut values: Vec<Value> = Vec::new();
        for i in from..to {
            values.push(Value::Integer(i as i64));
        }
        values
    }

    pub fn create(from: usize, to: usize) -> FlList {
        FlList::from_values(create_vec(from, to))
    }

    pub fn verify(list: &FlList, from: usize, to: usize) {
        assert_eq!(to - from, list.len(), "unexpected list length");
        for (i, v) in list.iter().enumerate() {
            match v {
                Value::Integer(n) => assert_eq!((from + i) as i64, n, "at index {i}"),
                _ => panic!("expected Value::Integer at index {i}, got {:?}", v),
            }
        }
    }
}
