
//use std::fmt;
//use std::sync::Arc;

use crate::{FlError, Value};

use super::{FlList};
//use super::{Bucket, FplList};


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

    fn check_not_empty(&self, message: &str) -> Result<(), FlError> {
        if self.buckets.len() > 0 {
            Ok(())
        } else {
            Err(FlError::new(message))
        }
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