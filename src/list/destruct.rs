use std::{fs::TryLockError::Error, mem::MaybeUninit, sync::Arc};

use crate::{
    FlError, Value,
    list::{BASE_SIZE, FACTOR},
};

use super::{Bucket, FlList};

impl FlList {
    pub fn remove_first(&self) -> Result<FlList, FlError> {
        self.check_not_empty("remove_first on empty list")?;
        if self.buckets[0].values.len() == 1 {
            Ok(Self::from_buckets(self.buckets[1..].iter().map(|b| {
                Bucket {
                    values: Arc::clone(&b.values),
                }
            })))
        } else if self.buckets[0].values.len() <= BASE_SIZE + 1 {
            let new_first = Bucket {
                values: Arc::<[Value]>::from(&self.buckets[0].values[1..]),
            };
            Ok(Self::from_buckets_sized(
                self.buckets.len(),
                std::iter::once(new_first).chain(self.buckets[1..].iter().map(|b| Bucket {
                    values: Arc::clone(&b.values),
                })),
            ))
        } else {
            let first_values = &self.buckets[0].values;
            let count = first_values.len() - 1;
            let split_sizes = Self::split_sizes(count);
            let total_buckets = split_sizes.len() + self.buckets.len() - 1;
            let mut src_idx: usize = 1;
            Ok(Self::from_buckets_sized(
                total_buckets,
                split_sizes
                    .into_iter()
                    .map(move |size| {
                        let bucket = Bucket {
                            values: Arc::<[Value]>::from(&first_values[src_idx..src_idx + size]),
                        };
                        src_idx += size;
                        bucket
                    })
                    .chain(self.buckets[1..].iter().map(|b| Bucket {
                        values: Arc::clone(&b.values),
                    })),
            ))
        }
    }

    pub fn remove_last(&self) -> Result<FlList, FlError> {
        self.check_not_empty("remove_last on empty list")?;
        let last_idx = self.buckets.len() - 1;
        let last_values = &self.buckets[last_idx].values;
        if last_values.len() == 1 {
            Ok(Self::from_buckets(self.buckets[..last_idx].iter().map(|b| {
                Bucket {
                    values: Arc::clone(&b.values),
                }
            })))
        } else if last_values.len() <= BASE_SIZE + 1 {
            let new_last = Bucket {
                values: Arc::<[Value]>::from(&last_values[..last_values.len() - 1]),
            };
            Ok(Self::from_buckets_sized(
                self.buckets.len(),
                self.buckets[..last_idx]
                    .iter()
                    .map(|b| Bucket {
                        values: Arc::clone(&b.values),
                    })
                    .chain(std::iter::once(new_last)),
            ))
        } else {
            let count = last_values.len() - 1;
            let split_sizes = Self::split_sizes(count);
            let total_buckets = last_idx + split_sizes.len();
            let mut src_idx: usize = 0;
            Ok(Self::from_buckets_sized(
                total_buckets,
                self.buckets[..last_idx]
                    .iter()
                    .map(|b| Bucket {
                        values: Arc::clone(&b.values),
                    })
                    .chain(split_sizes.into_iter().rev().map(move |size| {
                        let bucket = Bucket {
                            values: Arc::<[Value]>::from(&last_values[src_idx..src_idx + size]),
                        };
                        src_idx += size;
                        bucket
                    })),
            ))
        }
    }

    fn split_sizes(count: usize) -> Vec<usize> {
        std::iter::successors(Some(BASE_SIZE / 2), |&s| s.checked_mul(FACTOR))
            .scan(count, |rem, size| {
                if *rem == 0 {
                    None
                } else {
                    let take = size.min(*rem);
                    *rem -= take;
                    Some(take)
                }
            })
            .collect()
    }

    pub fn lower_half(&self) -> FlList {
        let to = self.len() / 2;
        if to == 0 {
            return Self::empty();
        }
        // Walk through buckets until the running total reaches `to`.
        let mut count = 0;
        let mut from_bucket_idx = 0;
        while count < to {
            count += self.buckets[from_bucket_idx].values.len();
            from_bucket_idx += 1;
        }
        if from_bucket_idx < self.buckets.len() && count == to {
            // Lucky case: `to` falls exactly on an existing bucket boundary.
            Self::from_buckets(self.buckets[..from_bucket_idx].iter().map(|b| Bucket {
                values: Arc::clone(&b.values),
            }))
        } else {
            let bucket_sizes = Self::create_shape_for_splitting(to);
            let num_buckets = bucket_sizes.len();
            let mut iter = self.into_iter();
            Self::from_buckets_sized(
                num_buckets,
                bucket_sizes.into_iter().map(move |size| {
                    Self::fill_bucket_sized(size, iter.by_ref().take(size))
                }),
            )
        }
    }

    pub fn upper_half(&self) -> FlList {
        let from = self.len() / 2;
        let mut count = 0;
        let mut from_bucket_idx = 0;
        while count < from {
            count += self.buckets[from_bucket_idx].values.len();
            from_bucket_idx += 1;
        }
        if count == from {
            // Lucky case: `from` falls exactly on an existing bucket boundary.
            Self::from_buckets(self.buckets[from_bucket_idx..].iter().map(|b| Bucket {
                values: Arc::clone(&b.values),
            }))
        } else {
            let to_take = self.len() - from;
            let bucket_sizes = Self::create_shape_for_splitting(to_take);
            let num_buckets = bucket_sizes.len();
            let mut iter = self.into_iter().skip(from);
            Self::from_buckets_sized(
                num_buckets,
                bucket_sizes.into_iter().map(move |size| {
                    Self::fill_bucket_sized(size, iter.by_ref().take(size))
                }),
            )
        }
    }

    fn create_shape_for_splitting(size: usize) -> Vec<usize> {
        let mut number_of_buckets: usize = 1;
        let mut bucket_size: usize = size;
        while bucket_size > BASE_SIZE && (2usize << number_of_buckets) < size {
            number_of_buckets *= 2;
            bucket_size = size / number_of_buckets;
        }
        let mut data = vec![0; number_of_buckets];
        Self::create_buckets(&mut data[..], 0, number_of_buckets, size);
        data
    }

    fn create_buckets(data: &mut [usize], offset: usize, number_of_buckets: usize, size: usize) {
        if number_of_buckets == 1 {
            data[offset] = size;
        } else {
            let next = number_of_buckets / 2;
            let lower = size / 2;
            let upper = size - lower;
            Self::create_buckets(data, offset, next, lower);
            Self::create_buckets(data, offset + next, next, upper);
        }
    }

    // TODO
    // public FplList set(int position, FplValue element) throws EvaluationException 
    // public FplList replaceElements(int from, FplList patch, int numReplaced) throws EvaluationException
    // public FplList subList(int fromIndex, int toIndex) throws EvaluationException

}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::super::tests::{create, create_vec, verify};
    use super::*;
    use crate::Value;

    fn bucket_sizes(list: &FlList) -> Vec<usize> {
        list.buckets.iter().map(|b| b.values.len()).collect()
    }

    #[test]
    fn test_remove_first_size_one() {
        let list = FlList::from_value(Value::Text(Arc::from("")));
        let rest = list.remove_first().unwrap();
        assert_eq!(0, rest.len());
    }

    #[test]
    fn test_remove_first_size_two() {
        let list =
            FlList::from_value(Value::Integer(1)).append(&FlList::from_value(Value::Integer(2)));
        let rest = list.remove_first().unwrap();
        assert_eq!(1, rest.len());
        assert_eq!(Value::Integer(2), rest.get(0).unwrap());
    }

    #[test]
    fn test_remove_first_size_hundred() {
        let list = create(0, 100);
        let rest = list.remove_first().unwrap();
        verify(&rest, 1, 100);
    }

    #[test]
    fn test_remove_first_size_fifty_append_fifty() {
        let list = create(0, 50).append(&create(50, 100));
        let rest = list.remove_first().unwrap();
        verify(&rest, 1, 100);
    }

    #[test]
    fn test_remove_first_size_two_from_append() {
        let list = create(1, 2).append(&create(2, 3));
        let rest = list.remove_first().unwrap();
        assert_eq!(1, rest.len());
        assert_eq!(Value::Integer(2), rest.get(0).unwrap());
    }

    #[test]
    fn test_remove_last_size_one() {
        let list = FlList::from_value(Value::Text(Arc::from("")));
        let rest = list.remove_last().unwrap();
        assert_eq!(0, rest.len());
    }

    #[test]
    fn test_remove_last_size_two() {
        let list =
            FlList::from_value(Value::Integer(1)).append(&FlList::from_value(Value::Integer(2)));
        let rest = list.remove_last().unwrap();
        assert_eq!(1, rest.len());
        assert_eq!(Value::Integer(1), rest.get(0).unwrap());
    }

    #[test]
    fn test_remove_last_size_hundred() {
        let list = create(0, 100);
        let rest = list.remove_last().unwrap();
        verify(&rest, 0, 99);
    }

    #[test]
    fn test_remove_last_size_fifty_append_fifty() {
        let list = create(0, 50).append(&create(50, 100));
        let rest = list.remove_last().unwrap();
        verify(&rest, 0, 99);
    }

    #[test]
    fn test_remove_last_size_two_from_append() {
        let list = create(1, 2).append(&create(2, 3));
        let rest = list.remove_last().unwrap();
        assert_eq!(1, rest.len());
        assert_eq!(Value::Integer(1), rest.get(0).unwrap());
    }

    #[test]
    fn test_lower_half_from_empty_list() {
        let list = FlList::empty().lower_half();
        assert!(list.is_empty());
    }

    #[test]
    fn test_lower_half_from_list_with_size_one() {
        let list = create(0, 1).lower_half();
        assert!(list.is_empty());
    }

    #[test]
    fn test_lower_half_from_small_list() {
        let list = create(0, 16);
        let lower = list.lower_half();
        verify(&lower, 0, 8);
        assert_eq!(vec![8], bucket_sizes(&lower));
    }

    #[test]
    fn test_lower_half_from_medium_list() {
        let list = FlList::from_values_with_shape(create_vec(0, 100), &[30, 40, 30]);
        let lower = list.lower_half();
        verify(&lower, 0, 50);
        assert_eq!(vec![6, 6, 6, 7, 6, 6, 6, 7], bucket_sizes(&lower));
    }

    #[test]
    fn test_lower_half_from_big_list() {
        let list = create(0, 1_000_000);
        let mut lower = list.lower_half();
        verify(&lower, 0, 500_000);
        assert_eq!(32, lower.buckets.len());
        lower = lower.lower_half();
        verify(&lower, 0, 250_000);
        assert_eq!(16, lower.buckets.len());
        while lower.buckets.len() > 1 {
            lower = lower.lower_half();
        }
        lower = lower.lower_half();
        assert_eq!(16, lower.buckets.len());
    }

    #[test]
    fn test_upper_half_from_empty_list() {
        let list = FlList::empty().upper_half();
        assert!(list.is_empty());
    }

    #[test]
    fn test_upper_half_from_list_with_size_one() {
        let list = create(0, 1).upper_half();
        verify(&list, 0, 1);
    }

    #[test]
    fn test_upper_half_from_small_list() {
        let list = create(0, 16);
        let upper = list.upper_half();
        verify(&upper, 8, 16);
        assert_eq!(vec![8], bucket_sizes(&upper));
    }

    #[test]
    fn test_upper_half_from_medium_list() {
        let list = FlList::from_values_with_shape(create_vec(0, 100), &[30, 40, 30]);
        let upper = list.upper_half();
        verify(&upper, 50, 100);
        assert_eq!(vec![6, 6, 6, 7, 6, 6, 6, 7], bucket_sizes(&upper));
    }

    #[test]
    fn test_upper_half_from_big_list() {
        let list = create(0, 1_000_000);
        let mut upper = list.upper_half();
        verify(&upper, 500_000, 1_000_000);
        assert_eq!(32, upper.buckets.len());
        upper = upper.upper_half();
        verify(&upper, 750_000, 1_000_000);
        assert_eq!(16, upper.buckets.len());
    }
}
