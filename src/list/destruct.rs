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
            Ok(Self::from_buckets(self.buckets[..last_idx].iter().map(
                |b| Bucket {
                    values: Arc::clone(&b.values),
                },
            )))
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
                bucket_sizes
                    .into_iter()
                    .map(move |size| Self::fill_bucket_sized(size, iter.by_ref().take(size))),
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
                bucket_sizes
                    .into_iter()
                    .map(move |size| Self::fill_bucket_sized(size, iter.by_ref().take(size))),
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

    pub fn sub_list(&self, from_index: usize, to_index: usize) -> Result<FlList, FlError> {
        if from_index > to_index {
            return Err(FlError::new("from_index > to_index"));
        }
        if from_index == to_index {
            return Ok(Self::empty());
        }
        if self.is_empty() {
            return Err(FlError::new("from_index >= size"));
        }

        // Locate first bucket containing `from_index`.
        let mut bucket_from_idx = 0;
        let mut index = 0;
        while index + self.buckets[bucket_from_idx].values.len() <= from_index {
            index += self.buckets[bucket_from_idx].values.len();
            bucket_from_idx += 1;
            if bucket_from_idx >= self.buckets.len() {
                return Err(FlError::new("from_index >= size"));
            }
        }
        let in_bucket_from_idx = from_index - index;
        let mut bucket_to_idx = bucket_from_idx;

        // Locate last bucket containing `to_index - 1`.
        while index + self.buckets[bucket_to_idx].values.len() <= to_index - 1 {
            index += self.buckets[bucket_to_idx].values.len();
            bucket_to_idx += 1;
            if bucket_to_idx >= self.buckets.len() {
                return Err(FlError::new("to_index > size"));
            }
        }
        let in_bucket_to_idx = to_index - index;

        // Full-list optimization: share the backing Arc<[Bucket]>.
        if from_index == 0
            && bucket_to_idx == self.buckets.len() - 1
            && in_bucket_to_idx == self.buckets[bucket_to_idx].values.len()
        {
            return Ok(self.clone());
        }

        if bucket_from_idx == bucket_to_idx {
            return Ok(Self::sub_list_from_one_large_array(
                &self.buckets[bucket_from_idx].values,
                in_bucket_from_idx,
                in_bucket_to_idx,
            ));
        }

        // Multi-bucket case: left + center + right.
        let left =
            Self::left_sub_buckets(&self.buckets[bucket_from_idx].values, in_bucket_from_idx);
        let right = Self::right_sub_buckets(&self.buckets[bucket_to_idx].values, in_bucket_to_idx);
        let center_count = bucket_to_idx - bucket_from_idx - 1;
        let total = left.len() + center_count + right.len();
        Ok(Self::from_buckets_sized(
            total,
            left.into_iter()
                .chain(
                    self.buckets[bucket_from_idx + 1..bucket_to_idx]
                        .iter()
                        .map(|b| Bucket {
                            values: Arc::clone(&b.values),
                        }),
                )
                .chain(right.into_iter()),
        ))
    }

    fn sub_list_from_one_large_array(bucket: &[Value], first: usize, behind_last: usize) -> FlList {
        let size = behind_last - first;
        if size <= BASE_SIZE {
            Self::from_buckets(std::iter::once(Bucket {
                values: Arc::<[Value]>::from(&bucket[first..behind_last]),
            }))
        } else {
            let bucket_sizes = Self::compute_bucket_sizes(size);
            let num = bucket_sizes.len();
            let mut idx = first;
            Self::from_buckets_sized(
                num,
                bucket_sizes.into_iter().map(move |sz| {
                    let b = Bucket {
                        values: Arc::<[Value]>::from(&bucket[idx..idx + sz]),
                    };
                    idx += sz;
                    b
                }),
            )
        }
    }

    fn left_sub_buckets(bucket: &Arc<[Value]>, in_from: usize) -> Vec<Bucket> {
        if in_from == 0 {
            return vec![Bucket {
                values: Arc::clone(bucket),
            }];
        }
        let mut result = Vec::new();
        let mut bucket_size = 3 * BASE_SIZE / 4;
        let mut in_idx = in_from;
        let mut rest = bucket.len() - in_from;
        while rest > bucket_size {
            let size = std::cmp::min(bucket_size / 2, rest);
            result.push(Bucket {
                values: Arc::<[Value]>::from(&bucket[in_idx..in_idx + size]),
            });
            in_idx += size;
            rest -= size;
            bucket_size *= FACTOR;
        }
        result.push(Bucket {
            values: Arc::<[Value]>::from(&bucket[in_idx..in_idx + rest]),
        });
        result
    }

    fn right_sub_buckets(bucket: &Arc<[Value]>, in_to: usize) -> Vec<Bucket> {
        if in_to == bucket.len() {
            return vec![Bucket {
                values: Arc::clone(bucket),
            }];
        }
        // Build from the right end backward, then reverse into natural order.
        let mut result = Vec::new();
        let mut bucket_size = 3 * BASE_SIZE / 4;
        let mut in_idx = in_to;
        let mut rest = in_to;
        while rest > bucket_size {
            let size = std::cmp::min(bucket_size / 2, rest);
            result.push(Bucket {
                values: Arc::<[Value]>::from(&bucket[in_idx - size..in_idx]),
            });
            in_idx -= size;
            rest -= size;
            bucket_size *= FACTOR;
        }
        result.push(Bucket {
            values: Arc::<[Value]>::from(&bucket[..rest]),
        });
        result.reverse();
        result
    }

    pub fn set(&self, position: usize, element: Value) -> Result<FlList, FlError> {
        self.check_not_empty("set on empty list")?;

        // Locate bucket containing `position`.
        let mut bucket_idx = 0;
        let mut count = 0;
        while count + self.buckets[bucket_idx].values.len() <= position {
            count += self.buckets[bucket_idx].values.len();
            bucket_idx += 1;
            if bucket_idx >= self.buckets.len() {
                return Err(FlError::new("position >= size"));
            }
        }
        let in_bucket_idx = position - count;
        let bucket_size = self.buckets[bucket_idx].values.len();

        if bucket_size <= BASE_SIZE {
            // Simple case: replace one small bucket; splice `element` in at the target index.
            let old_values = &self.buckets[bucket_idx].values;
            let new_bucket = Self::fill_bucket_sized(
                bucket_size,
                old_values[..in_bucket_idx]
                    .iter().cloned()
                    .chain(std::iter::once(element))
                    .chain(old_values[in_bucket_idx + 1..].iter().cloned()),
            );
            Ok(Self::from_buckets_sized(
                self.buckets.len(),
                self.buckets[..bucket_idx]
                    .iter()
                    .map(|b| Bucket {
                        values: Arc::clone(&b.values),
                    })
                    .chain(std::iter::once(new_bucket))
                    .chain(self.buckets[bucket_idx + 1..].iter().map(|b| Bucket {
                        values: Arc::clone(&b.values),
                    })),
            ))
        } else {
            let total_size = self.len();
            if Self::needs_reshaping(self.buckets.len() + 1, total_size) {
                // Rebuild entire list into ideal shape, splicing `element` at `position`.
                let bucket_sizes = Self::compute_bucket_sizes(total_size);
                let num_buckets = bucket_sizes.len();
                let mut iter = self
                    .into_iter()
                    .take(position)
                    .chain(std::iter::once(element))
                    .chain(self.into_iter().skip(position + 1));
                Ok(Self::from_buckets_sized(
                    num_buckets,
                    bucket_sizes.into_iter().map(move |sz| {
                        Self::fill_bucket_sized(sz, iter.by_ref().take(sz))
                    }),
                ))
            } else {
                // Split the large bucket into two halves; splice `element` into the half that holds it.
                let big_values = &self.buckets[bucket_idx].values;
                let left_size = bucket_size / 2;
                let right_size = bucket_size - left_size;
                let (left_bucket, right_bucket) = if in_bucket_idx < left_size {
                    let left = Self::fill_bucket_sized(
                        left_size,
                        big_values[..in_bucket_idx]
                            .iter()
                            .cloned()
                            .chain(std::iter::once(element))
                            .chain(big_values[in_bucket_idx + 1..left_size].iter().cloned()),
                    );
                    let right = Bucket {
                        values: Arc::<[Value]>::from(&big_values[left_size..]),
                    };
                    (left, right)
                } else {
                    let left = Bucket {
                        values: Arc::<[Value]>::from(&big_values[..left_size]),
                    };
                    let r_idx = in_bucket_idx - left_size;
                    let right = Self::fill_bucket_sized(
                        right_size,
                        big_values[left_size..left_size + r_idx]
                            .iter()
                            .cloned()
                            .chain(std::iter::once(element))
                            .chain(big_values[left_size + r_idx + 1..].iter().cloned()),
                    );
                    (left, right)
                };
                Ok(Self::from_buckets_sized(
                    self.buckets.len() + 1,
                    self.buckets[..bucket_idx]
                        .iter()
                        .map(|b| Bucket {
                            values: Arc::clone(&b.values),
                        })
                        .chain(std::iter::once(left_bucket))
                        .chain(std::iter::once(right_bucket))
                        .chain(self.buckets[bucket_idx + 1..].iter().map(|b| Bucket {
                            values: Arc::clone(&b.values),
                        })),
                ))
            }
        }
    }

    // TODO public FplList replaceElements(int from, FplList patch, int numReplaced) throws EvaluationException
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

    #[test]
    fn test_sub_list_bad_range() {
        let list = FlList::empty();
        assert!(list.sub_list(10, 0).is_err());
    }

    #[test]
    fn test_sub_list_from_equals_to_results_in_empty() {
        let list = create(0, 10);
        assert_eq!(0, list.sub_list(3, 3).unwrap().len());
    }

    #[test]
    fn test_large_sub_list_end_beyond_end_of_list() {
        let list = create(1, 11);
        assert!(list.sub_list(3, 12).is_err());
    }

    #[test]
    fn test_small_sub_list_end_beyond_end_of_list() {
        let list = create(1, 8);
        assert!(list.sub_list(3, 8).is_err());
    }

    #[test]
    fn test_sub_list_start_beyond_end_of_list() {
        let list = FlList::from_values_with_shape(create_vec(0, 16), &[4, 8, 4]);
        assert!(list.sub_list(16, 17).is_err());
    }

    #[test]
    fn test_sub_list_complete_of_short_list() {
        let list = create(1, 7);
        let sub = list.sub_list(0, 6).unwrap();
        assert!(Arc::ptr_eq(&list.buckets, &sub.buckets));
    }

    #[test]
    fn test_sub_list_complete_of_large_list() {
        let list = FlList::from_values_with_shape(create_vec(1, 24), &[4, 15, 4]);
        let sub = list.sub_list(0, 23).unwrap();
        assert!(Arc::ptr_eq(&list.buckets, &sub.buckets));
    }

    #[test]
    fn test_sub_list_of_short_list() {
        let list = create(1, 11).sub_list(3, 5).unwrap();
        verify(&list, 4, 6);
    }

    #[test]
    fn test_sub_list_start_of_short_list() {
        let list = create(0, 7).sub_list(0, 6).unwrap();
        verify(&list, 0, 6);
    }

    #[test]
    fn test_sub_list_end_of_short_list() {
        let list = create(0, 7).sub_list(1, 7).unwrap();
        verify(&list, 1, 7);
    }

    #[test]
    fn test_sub_list_from_one_small_bucket() {
        let list = FlList::from_values_with_shape(create_vec(0, 16), &[4, 8, 4]);
        verify(&list.sub_list(5, 7).unwrap(), 5, 7);
    }

    #[test]
    fn test_sub_list_buckets_start() {
        let list = FlList::from_values_with_shape(create_vec(0, 16), &[4, 8, 4]);
        verify(&list.sub_list(0, 7).unwrap(), 0, 7);
    }

    #[test]
    fn test_sub_list_buckets_within() {
        let list = FlList::from_values_with_shape(create_vec(0, 16), &[4, 8, 4]);
        verify(&list.sub_list(2, 7).unwrap(), 2, 7);
    }

    #[test]
    fn test_sub_list_buckets_start_with_part_from_last_bucket() {
        let list = FlList::from_values_with_shape(create_vec(0, 16), &[4, 8, 4]);
        verify(&list.sub_list(0, 13).unwrap(), 0, 13);
    }

    #[test]
    fn test_sub_list_buckets_end() {
        let list = FlList::from_values_with_shape(create_vec(0, 16), &[4, 8, 4]);
        verify(&list.sub_list(3, 16).unwrap(), 3, 16);
    }

    #[test]
    fn test_sub_list_from_one_large_bucket() {
        let list = FlList::from_values_with_shape(create_vec(0, 40), &[4, 32, 4]);
        verify(&list.sub_list(5, 17).unwrap(), 5, 17);
    }

    #[test]
    fn test_sub_list_from_several_large_buckets() {
        let list = FlList::from_values_with_shape(create_vec(0, 100), &[20, 20, 20, 20, 20]);
        verify(&list, 0, 100);
        verify(&list.sub_list(5, 95).unwrap(), 5, 95);
    }

    #[test]
    fn test_sub_list_from_shape() {
        let list = FlList::from_values_with_shape(create_vec(0, 30), &[4, 22, 4]);
        verify(&list.sub_list(0, 5).unwrap(), 0, 5);
    }

    #[test]
    fn test_sub_list_from_special_shape1() {
        let list = FlList::from_values_with_shape(create_vec(0, 45), &[3, 37, 5]);
        verify(&list.sub_list(0, 38).unwrap(), 0, 38);
    }

    #[test]
    fn test_sub_list_from_special_shape2() {
        let list = FlList::from_values_with_shape(create_vec(0, 10), &[2, 8]);
        verify(&list.sub_list(0, 9).unwrap(), 0, 9);
    }
/*
	private void checkUpdated(int updatedIndex, int expectedSize, FplList list) {
		assertEquals(expectedSize, list.size());
		for (int i = 0; i < expectedSize; i++) {
			if (i == updatedIndex) {
				assertEquals(number, list.get(i));
			} else {
				assertEquals(FplInteger.valueOf(i), list.get(i));
			}
		}
	}
 */
    fn check_updated(updated_index: usize, expected_size: usize, list: &FlList) {
        assert_eq!(expected_size, list.len());
        for i in 0..expected_size {
            let expected = if i == updated_index {
                Value::Integer(-1)
            } else {
                Value::Integer(i as i64)
            };
            assert_eq!(expected, list.get(i as isize).unwrap(), "at index {}", i);
        }
    }

    #[test]
    fn test_set_on_empty_throws() {
        let list = FlList::empty();
        assert!(list.set(0, Value::Text(Arc::from("foo"))).is_err());
    }

    #[test]
    fn test_set_with_index_out_of_bounds_one_bucket() {
        let list = create(0, 5);
        assert!(list.set(5, Value::Text(Arc::from("foo"))).is_err());
    }

    #[test]
    fn test_set_with_index_out_of_bounds() {
        let list = FlList::from_values_with_shape(create_vec(0, 10), &[5, 5]);
        assert!(list.set(10, Value::Text(Arc::from("foo"))).is_err());
    }

    #[test]
    fn test_update_first_one_small_bucket() {
        let list = create(0, 8).set(0, Value::Integer(-1)).unwrap();
        check_updated(0, 8, &list);
    }

    #[test]
    fn test_update_first_two_buckets() {
        let list = FlList::from_values_with_shape(create_vec(0, 20), &[10, 10])
            .set(0, Value::Integer(-1))
            .unwrap();
        check_updated(0, 20, &list);
    }

    #[test]
    fn test_update_second_two_buckets() {
        let list = FlList::from_values_with_shape(create_vec(0, 20), &[10, 10])
            .set(1, Value::Integer(-1))
            .unwrap();
        check_updated(1, 20, &list);
    }

    #[test]
    fn test_update_first_with_reshape() {
        let list = FlList::from_values_with_shape(
            create_vec(0, 100),
            &[10, 10, 10, 10, 10, 10, 10, 10, 10, 10],
        )
        .set(0, Value::Integer(-1))
        .unwrap();
        check_updated(0, 100, &list);
    }

    #[test]
    fn test_update_tenths_with_reshape() {
        let list = FlList::from_values_with_shape(
            create_vec(0, 100),
            &[4, 15, 11, 10, 10, 10, 10, 10, 10, 10],
        )
        .set(10, Value::Integer(-1))
        .unwrap();
        check_updated(10, 100, &list);
    }

    #[test]
    fn test_update_last_one_small_bucket() {
        let list = create(0, 8).set(7, Value::Integer(-1)).unwrap();
        check_updated(7, 8, &list);
    }

    #[test]
    fn test_update_last_two_buckets() {
        let list = FlList::from_values_with_shape(create_vec(0, 20), &[10, 10])
            .set(19, Value::Integer(-1))
            .unwrap();
        check_updated(19, 20, &list);
    }

    #[test]
    fn test_update_second_last_two_buckets() {
        let list = FlList::from_values_with_shape(create_vec(0, 20), &[10, 10])
            .set(18, Value::Integer(-1))
            .unwrap();
        check_updated(18, 20, &list);
    }

    #[test]
    fn test_update_last_with_reshape() {
        let list = FlList::from_values_with_shape(
            create_vec(0, 100),
            &[10, 10, 10, 10, 10, 10, 10, 10, 10, 10],
        )
        .set(99, Value::Integer(-1))
        .unwrap();
        check_updated(99, 100, &list);
    }
}
