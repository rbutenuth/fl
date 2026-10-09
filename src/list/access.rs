
//use std::fmt;
use std::sync::Arc;

use crate::{FlError, Value};

use super::{Bucket, FlList};


impl FlList {
    pub fn len(&self) -> usize {
        self.buckets.iter().map(|b| b.values.len()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.buckets.len() == 0
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

    pub fn first(&self) -> Result<Value, FlError> {
        self.check_not_empty("first on empty list")?;
        Ok(self.buckets[0].values[0].clone())
    }

    pub fn last(&self) -> Result<Value, FlError> {
        self.check_not_empty("last on empty list")?;
        Ok(self.buckets.last().unwrap().values.last().unwrap().clone())
    }

}

pub struct FlListIter {
    buckets: Arc<[Bucket]>,
    bucket_idx: usize,
    in_bucket_idx: usize,
    back_bucket_idx: usize,
    back_in_bucket_idx: usize,
    remaining: usize,
    len: usize,
}

impl Iterator for FlListIter {
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining > 0 {
            self.remaining -= 1;
            let current_values = &self.buckets[self.bucket_idx].values;
            let result = current_values[self.in_bucket_idx].clone();
            if self.in_bucket_idx < current_values.len() - 1 {
                self.in_bucket_idx += 1;
            } else {
                self.bucket_idx += 1;
                self.in_bucket_idx = 0;
            }
            Some(result)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl ExactSizeIterator for FlListIter {
    fn len(&self) -> usize {
        self.len
    }
}

impl DoubleEndedIterator for FlListIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.remaining > 0 {
            self.remaining -= 1;
            let current_values = &self.buckets[self.back_bucket_idx].values;
            let result = current_values[self.back_in_bucket_idx].clone();
            if self.back_in_bucket_idx > 0 {
                self.back_in_bucket_idx -= 1;
            } else {
                self.back_bucket_idx -= 1;
                self.back_in_bucket_idx = self.buckets[self.back_bucket_idx].values.len() - 1;
            }
            Some(result)
        } else {
            None
        }
    }
}

impl IntoIterator for &FlList {
    type Item = Value;
    type IntoIter = FlListIter;

    fn into_iter(self) -> Self::IntoIter {
        let len = self.len();
        let last_bucket_idx = self.buckets.len() - 1;

        FlListIter {
            buckets: Arc::clone(&self.buckets),
            bucket_idx: 0,
            in_bucket_idx: 0,
            back_bucket_idx: last_bucket_idx,
            back_in_bucket_idx: if last_bucket_idx > 0 { self.buckets[last_bucket_idx].values.len() - 1 } else { 0 },
            remaining: len,
            len,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::super::tests::{create, verify};

    #[test]
    fn test_empty_has_len_0_and_get_fails() {
        let list = FlList::empty();
        assert_eq!(0, list.len());
        let at_zero = list.get(0);
        assert!(at_zero.is_err());
        let first = list.first();
        assert!(first.is_err());
        let last = list.last();
        assert!(last.is_err());
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

    #[test]
    fn test_first_and_last() {
        let list = create(1, 6);
        verify(&list, 1, 6);
        assert_eq!(Value::Integer(1), list.first().unwrap());
        assert_eq!(Value::Integer(5), list.last().unwrap());
    }

    #[test]
    fn test_is_empty() {
        let list = FlList::empty();
        assert_eq!(0, list.len());
        let list = FlList::from_value(Value::Integer(42));
        assert!(!list.is_empty());
    }

}