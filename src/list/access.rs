
//use std::fmt;
use std::sync::Arc;

use crate::{FlError, Value};

use super::{Bucket, FlList};


impl FlList {
    pub fn len(&self) -> usize {
        self.buckets.iter().map(|b| b.values.len()).sum()
    }

    pub fn get(&self, index: isize) -> Result<Value, FlError> {
        self.check_not_empty("get on empty list")?;
        if index < 0 {
            return Err(FlError::new(&format!("negative index: {}", index)));
        }
        let u_index = index as usize;
        let mut bucket_idx = 0;
        let mut count = 0;

        while count + self.buckets[bucket_idx].values.len() <= u_index {
            count += self.buckets[bucket_idx].values.len();
            bucket_idx += 1;
            if bucket_idx >= self.buckets.len() {
            	return Err(FlError::new("index >= size"));
            }
        }

        Ok(self.buckets[bucket_idx].values[u_index - count].clone())
    }

    pub fn iter(&self) -> FlListIter {
        FlListIter {
            buckets: Arc::clone(&self.buckets),
            bucket_idx: 0,
            in_bucket_idx: 0,
        }
    }

    fn check_not_empty(&self, message: &str) -> Result<(), FlError> {
        if self.buckets.len() > 0 {
            Ok(())
        } else {
            Err(FlError::new(message))
        }
    }
}

pub struct FlListIter {
    buckets: Arc<[Bucket]>,
    bucket_idx: usize,
    in_bucket_idx: usize,
}

impl Iterator for FlListIter {
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        while self.bucket_idx < self.buckets.len() {
            let bucket = &self.buckets[self.bucket_idx];
            if self.in_bucket_idx < bucket.values.len() {
                let v = bucket.values[self.in_bucket_idx].clone();
                self.in_bucket_idx += 1;
                return Some(v);
            }
            self.bucket_idx += 1;
            self.in_bucket_idx = 0;
        }
        None
    }
}

impl IntoIterator for &FlList {
    type Item = Value;
    type IntoIter = FlListIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_has_len_0_and_get_fails() {
        let list = FlList::empty();
        assert_eq!(0, list.len());
        let result = list.get(0);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_with_negative_index_fails() {
        let list = FlList::from_value(Value::Integer(42));
        let result = list.get(-1);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_with_index_out_of_bounds_fails() {
        let list = FlList::from_value(Value::Integer(42));
        let result = list.get(1);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_first() {
        let list = FlList::from_value(Value::Integer(42));
        let value = list.get(0).unwrap();
        assert_eq!(Value::Integer(42), value);
    }
}