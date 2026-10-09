use std::mem::MaybeUninit;
use std::sync::Arc;

use crate::Value;

use super::{Bucket, FlList};

impl FlList {
    pub fn from_value(value: Value) -> FlList {
        let bucket = Self::fill_bucket(std::iter::once(value));

        let mut u_buckets: Arc<[MaybeUninit<Bucket>]> = Arc::new_uninit_slice(1);
        let m_buckets = Arc::get_mut(&mut u_buckets).unwrap();
        m_buckets[0].write(bucket);
        FlList {
            buckets: unsafe { u_buckets.assume_init() },
        }
    }

    pub fn from_pair(left: Value, right: Value) -> FlList {
        let bucket = Self::fill_bucket([left, right].into_iter());

        let mut u_buckets: Arc<[MaybeUninit<Bucket>]> = Arc::new_uninit_slice(1);
        let m_buckets = Arc::get_mut(&mut u_buckets).unwrap();
        m_buckets[0].write(bucket);
        FlList {
            buckets: unsafe { u_buckets.assume_init() },
        }
    }

    pub fn from_vec_values(elements: Vec<Value>) -> FlList {
        if elements.len() == 0 {
            Self::empty()
        } else {
            let bucket_sizes = FlList::compute_bucket_sizes(elements.len());
            let mut iter = elements.into_iter();
            Self::from_buckets(bucket_sizes.into_iter().map(move |size| {
                Self::fill_bucket(iter.by_ref().take(size))
            }))
        }
    }

    fn from_iterator_values(mut items: impl ExactSizeIterator<Item = Value>) -> FlList {
        let bucket_sizes = FlList::compute_bucket_sizes(items.len());
        Self::from_buckets(bucket_sizes.into_iter().map(move |size| {
            Self::fill_bucket(items.by_ref().take(size))
        }))
    }

    pub fn from_values_with_shape(values: Vec<Value>, bucket_sizes: &[usize]) -> FlList {
        let sum: usize = bucket_sizes.iter().sum();
        assert_eq!(values.len(), sum, "values.len() != sum of bucket_sizes");

        let mut iter = values.into_iter();
        Self::from_buckets(bucket_sizes.iter().map(move |&size| {
            Self::fill_bucket(iter.by_ref().take(size))
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::*;
    use super::*;

    #[test]
    fn test_from_value_has_size_1() {
        let list = FlList::from_value(Value::Integer(42));
        assert_eq!(1, list.len());
        match list.buckets[0].values[0].clone() {
            Value::Integer(i) => assert_eq!(42, i),
            _ => panic!("should be Integer"),
        }
    }

    #[test]
    fn test_from_pair() {
        let list = FlList::from_pair(Value::Integer(1), Value::Integer(2));
        verify(&list, 1, 3);
    }

    #[test]
    fn test_from_value_vec() {
        let list = FlList::from_vec_values(create_vec(0, 10));
        assert_eq!(list.len(), 10);
        verify(&list, 0, 10);
    }

    #[test]
    fn test_clone() {
        let list = FlList::from_value(Value::Integer(42));
        assert_eq!(1, Arc::strong_count(&(list.buckets)));
        {
            let cloned = list.clone();
            assert_eq!(list.len(), cloned.len());
            assert_eq!(2, Arc::strong_count(&(list.buckets)));
            assert!(Arc::ptr_eq(&list.buckets, &cloned.buckets));
        }
        assert_eq!(1, Arc::strong_count(&(list.buckets)));
    }
}
