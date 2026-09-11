use std::mem::MaybeUninit;
use std::sync::Arc;

use crate::Value;

use super::{Bucket, FlList};

const BASE_SIZE: usize = 8;
const FACTOR: usize = 4;

impl FlList {
    pub fn empty() -> FlList {
        FlList {
            buckets: unsafe { Arc::new_uninit_slice(0).assume_init() },
        }
    }

    pub fn from_value(value: Value) -> FlList {
        let mut u_values: Arc<[MaybeUninit<Value>]> = Arc::new_uninit_slice(1);

        let values = Arc::get_mut(&mut u_values).unwrap();
        values[0].write(value);
        let bucket = Bucket {
            values: unsafe { u_values.assume_init() },
        };

        let mut u_buckets: Arc<[MaybeUninit<Bucket>]> = Arc::new_uninit_slice(1);
        let m_buckets = Arc::get_mut(&mut u_buckets).unwrap();
        m_buckets[0].write(bucket);
        FlList {
            buckets: unsafe { u_buckets.assume_init() },
        }
    }

    pub fn from_pair(left: Value, right: Value) -> FlList {
        let mut u_values: Arc<[MaybeUninit<Value>]> = Arc::new_uninit_slice(2);

        let values = Arc::get_mut(&mut u_values).unwrap();
        values[0].write(left);
        values[1].write(right);
        let bucket = Bucket {
            values: unsafe { u_values.assume_init() },
        };

        let mut u_buckets: Arc<[MaybeUninit<Bucket>]> = Arc::new_uninit_slice(1);
        let m_buckets = Arc::get_mut(&mut u_buckets).unwrap();
        m_buckets[0].write(bucket);
        FlList {
            buckets: unsafe { u_buckets.assume_init() },
        }
    }

    pub fn from_values(elements: Vec<Value>) -> FlList {
        if elements.len() == 0 {
            Self::empty()
        } else {
            let bucket_sizes = Self::compute_bucket_sizes(elements.len());
            let mut u_buckets: Arc<[MaybeUninit<Bucket>]> = Arc::new_uninit_slice(bucket_sizes.len());
            let m_buckets = Arc::get_mut(&mut u_buckets).unwrap();
            let mut src_idx = 0;
            for (bucket_idx, &size) in bucket_sizes.iter().enumerate() {
                let mut values = Self::nil_values(size);
                let slice = Arc::get_mut(&mut values).unwrap();
                for v in slice {
                    *v = elements[src_idx].clone();
                    src_idx += 1;
                }
                m_buckets[bucket_idx].write(Bucket { values: values});
            }

            FlList {
                buckets: unsafe { u_buckets.assume_init() },
            }
        }
    }

    pub fn nil_values(len: usize) -> Arc<[Value]> {
        let mut u_values: Arc<[MaybeUninit<Value>]> = Arc::new_uninit_slice(len);
        let mutable = Arc::get_mut(&mut u_values).unwrap();
        for slot in mutable.iter_mut() {
            slot.write(Value::Nil);
        }
        unsafe { u_values.assume_init() }
    }

    fn compute_bucket_sizes(size: usize) -> Vec<usize> {
        let mut num_buckets: usize = 2;
        let mut bucket_size = 3 * BASE_SIZE / 4;
        let mut size_in_buckets = 2 * bucket_size;
        while size_in_buckets < size {
            bucket_size *= FACTOR;
            size_in_buckets += 2 * bucket_size;
            num_buckets += 2;
        }
        num_buckets -= 1;
        let mut bucket_sizes = vec![0usize; num_buckets];
        bucket_size = BASE_SIZE;
        let mut rest = size;
        let mut i = 0;
        let mut j = num_buckets - 1;
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
}

#[cfg(test)]
mod tests {
    use super::super::tests::*;
    use super::*;

    #[test]
    fn test_empty_has_size_0() {
        assert_eq!(0, FlList::empty().len());
    }

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
        let list = FlList::from_values(create_vec(0, 10));
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
