use std::mem::MaybeUninit;
use std::sync::Arc;

use crate::{FlError, Value, list::{BASE_SIZE, FACTOR}};

use super::{Bucket, FlList};

impl FlList {

	pub fn append(&self, list: &FlList) -> FlList {
        if self.is_empty() {
            if list.is_empty() {
                FlList::empty()
            } else {
                list.clone()
            }
        } else {
            if list.is_empty() {
                self.clone()
            } else {
                let total_size = self.len() + list.len();
                let total_buckets = self.buckets.len() + list.buckets.len();

                let last_bucket = self.buckets.last().unwrap();
                let first_bucket = list.buckets.first().unwrap();

                if last_bucket.values.len() + first_bucket.values.len() <= BASE_SIZE {
                    if FlList::needs_reshaping(total_buckets - 1, total_size) {
                        Self::merge(self, list, total_size)
                    } else {
                        let len = last_bucket.values.len() + first_bucket.values.len();
                        let merged_bucket = Self::fill_bucket_sized(
                            len,
                            last_bucket.values.iter().chain(first_bucket.values.iter()).cloned(),
                        );
                        Self::from_buckets_sized(
                            total_buckets - 1,
                            self.buckets.iter().take(self.buckets.len() - 1)
                                .map(|b| Bucket { values: Arc::clone(&b.values) })
                                .chain(std::iter::once(merged_bucket))
                                .chain(list.buckets.iter().skip(1).map(|b| Bucket { values: Arc::clone(&b.values) })),
                        )
                    }
                } else {
                    if FlList::needs_reshaping(total_buckets, total_size) {
                        Self::merge(self, list, total_size)
                    } else {
                        Self::from_buckets_sized(
                            total_buckets,
                            self.buckets.iter()
                                .chain(list.buckets.iter())
                                .map(|b| Bucket { values: Arc::clone(&b.values) }),
                        )
                    }
                }
            }
        }
    }

    fn merge(left: &FlList, right: &FlList, total_size: usize) -> FlList {
        let mut iter = left.into_iter().chain(right.into_iter());
        Self::from_iterator_values(
            (0..total_size).map(move |_| iter.next().unwrap_or(Value::Nil))
        )
    }

    pub fn add_at_start(&self, value: Value) -> FlList {
        let n = self.buckets.len();
        let mut bucket_idx: usize = 0;
        let mut carry_size: usize = 1;
        let mut max_size: usize = BASE_SIZE;
        let mut last_size: usize = 0;
        while bucket_idx < n {
            let bucket_size = self.buckets[bucket_idx].values.len();
            if bucket_size < last_size {
                break;
            }
            if carry_size + bucket_size < max_size {
                bucket_idx += 1;
                carry_size += bucket_size;
                break;
            }
            if bucket_size >= max_size {
                break;
            }
            last_size = bucket_size;
            bucket_idx += 1;
            carry_size += bucket_size;
            max_size *= FACTOR;
        }
        let carry_bucket = Self::fill_bucket_sized(
            carry_size,
            std::iter::once(value).chain(
                self.buckets[..bucket_idx]
                    .iter()
                    .flat_map(|b| b.values.iter().cloned()),
            ),
        );
        Self::from_buckets_sized(
            n - bucket_idx + 1,
            std::iter::once(carry_bucket).chain(self.buckets[bucket_idx..].iter().map(|b| Bucket {
                values: Arc::clone(&b.values),
            })),
        )
    }

    pub fn add_at_end(&self, value: Value) -> FlList {
        let n = self.buckets.len();
        let mut consumed: usize = 0;
        let mut carry_size: usize = 1;
        let mut max_size: usize = BASE_SIZE;
        let mut last_size: usize = 0;
        while consumed < n {
            let bucket_size = self.buckets[n - 1 - consumed].values.len();
            if bucket_size < last_size {
                break;
            }
            if carry_size + bucket_size < max_size {
                consumed += 1;
                carry_size += bucket_size;
                break;
            }
            if bucket_size >= max_size {
                break;
            }
            last_size = bucket_size;
            consumed += 1;
            carry_size += bucket_size;
            max_size *= FACTOR;
        }
        let kept = n - consumed;
        let carry_bucket = Self::fill_bucket_sized(
            carry_size,
            self.buckets[kept..n]
                .iter()
                .flat_map(|b| b.values.iter().cloned())
                .chain(std::iter::once(value)),
        );
        Self::from_buckets_sized(
            kept + 1,
            self.buckets[..kept]
                .iter()
                .map(|b| Bucket {
                    values: Arc::clone(&b.values),
                })
                .chain(std::iter::once(carry_bucket)),
        )
    }


}

#[cfg(test)]
mod tests {
    use crate::Value;

use super::super::tests::*;
    use super::*;

    fn bucket_sizes(list: &FlList) -> Vec<usize> {
        list.buckets.iter().map(|b| b.values.len()).collect()
    }

    #[test]
    fn test_append_both_empty() {
        let left = FlList::empty();
        let right = FlList::empty();
        assert_eq!(0, left.append(&right).len());
    }

    #[test]
    fn test_append_left_empty() {
        let left = FlList::empty();
        let right = FlList::from_value(Value::Integer(42));
        let result = left.append(&right);
        assert_eq!(1, result.len());
        verify(&result, 42, 43);
    }

    #[test]
    fn test_append_right_empty() {
        let left  = FlList::from_value(Value::Integer(42));
        let right= FlList::empty();
        let result = left.append(&right);
        assert_eq!(1, result.len());
        verify(&result, 42, 43);
    }

        #[test]
    fn test_linear_append_linear_result_linear() {
        let left = create(0, 4);
        let right = create(4, 8);
        let result = left.append(&right);
        verify(&result, 0, 8);
    }

    #[test]
    fn test_linear_linear_result_shaped() {
        let list = create(0, 6).append(&create(6, 13));
        verify(&list, 0, 13);
    }

    #[test]
    fn test_shaped_linear_fits_in_last_result_shaped() {
        let list = FlList::from_values_with_shape(create_vec(0, 36), &[32, 4])
            .append(&create(36, 39));
        verify(&list, 0, 39);
    }

    #[test]
    fn test_shaped_linear_does_not_fit_in_last_result_shaped() {
        let list = FlList::from_values_with_shape(create_vec(0, 36), &[32, 4])
            .append(&create(36, 44));
        verify(&list, 0, 44);
    }    

    #[test]
    fn test_linear_shaped_fits_in_first_result_shaped() {
        let list = create(0, 6)
            .append(&FlList::from_values_with_shape(create_vec(6, 106), &[1, 99]));
        verify(&list, 0, 106);
    }

    #[test]
    fn test_linear_shaped_does_not_fit_in_first_result_shaped() {
        let list = create(0, 6)
            .append(&FlList::from_values_with_shape(create_vec(6, 106), &[8, 92]));
        verify(&list, 0, 106);
    }

    #[test]
    fn test_shaped_shaped_buckets_combinable() {
        let list = FlList::from_values_with_shape(create_vec(0, 10), &[6, 4])
            .append(&FlList::from_values_with_shape(create_vec(10, 20), &[4, 6]));
        verify(&list, 0, 20);
    }

    #[test]
    fn test_shaped_shaped_buckets_combinable_need_reshape() {
        let list = FlList::from_values_with_shape(create_vec(0, 6), &[1, 1, 4])
            .append(&FlList::from_values_with_shape(create_vec(6, 12), &[4, 1, 1]));
        verify(&list, 0, 12);
    }

    #[test]
    fn test_shaped_shaped_without_reshape() {
        let list = FlList::from_values_with_shape(create_vec(0, 16), &[8, 8])
            .append(&FlList::from_values_with_shape(create_vec(16, 32), &[8, 8]));
        verify(&list, 0, 32);
    }

    #[test]
    fn test_shaped_shaped_with_reshape() {
        let list = FlList::from_values_with_shape(create_vec(0, 16), &[2, 2, 2, 2, 8])
            .append(&FlList::from_values_with_shape(create_vec(16, 32), &[8, 2, 2, 2, 2]));
        verify(&list, 0, 32);
    }

    #[test]
    fn test_shaped_shaped_with_reshape2() {
        let list = FlList::from_values_with_shape(create_vec(0, 16), &[8, 2, 2, 2, 2])
            .append(&FlList::from_values_with_shape(create_vec(16, 32), &[2, 2, 2, 2, 8]));
        verify(&list, 0, 32);
    }

    #[test]
    fn test_add_at_start_on_empty_list() {
        let list = FlList::empty().add_at_start(Value::Integer(0));
        verify(&list, 0, 1);
        assert_eq!(vec![1], bucket_sizes(&list));
    }

    #[test]
    fn test_add_at_end_on_empty_list() {
        let list = FlList::empty().add_at_end(Value::Integer(0));
        verify(&list, 0, 1);
        assert_eq!(vec![1], bucket_sizes(&list));
    }

    #[test]
    fn test_add_at_start_on_list_with_space_in_first_bucket() {
        let list = create(1, 7).add_at_start(Value::Integer(0));
        verify(&list, 0, 7);
        assert_eq!(vec![7], bucket_sizes(&list));
    }

    #[test]
    fn test_add_at_end_on_list_with_space_in_last_bucket() {
        let list = create(0, 6).add_at_end(Value::Integer(6));
        verify(&list, 0, 7);
        assert_eq!(vec![7], bucket_sizes(&list));
    }

    #[test]
    fn test_add_at_start_on_list_with_nearly_full_first_bucket() {
        let list = create(1, 8).add_at_start(Value::Integer(0));
        verify(&list, 0, 8);
        assert_eq!(vec![8], bucket_sizes(&list));
    }

    #[test]
    fn test_add_at_end_on_list_with_nearly_full_last_bucket() {
        let list = create(0, 7).add_at_end(Value::Integer(7));
        verify(&list, 0, 8);
        assert_eq!(vec![8], bucket_sizes(&list));
    }

    #[test]
    fn test_add_at_start_on_list_with_full_first_bucket() {
        let list = create(1, 9).add_at_start(Value::Integer(0));
        verify(&list, 0, 9);
        assert_eq!(vec![1, 8], bucket_sizes(&list));
    }

    #[test]
    fn test_add_at_end_on_list_with_full_last_bucket() {
        let list = create(0, 8).add_at_end(Value::Integer(8));
        verify(&list, 0, 9);
        assert_eq!(vec![8, 1], bucket_sizes(&list));
    }

    #[test]
    fn test_add_at_start_on_list_7_8_16() {
        let list = FlList::from_values_with_shape(create_vec(1, 32), &[7, 8, 16])
            .add_at_start(Value::Integer(0));
        verify(&list, 0, 32);
        assert_eq!(vec![16, 16], bucket_sizes(&list));
    }

    #[test]
    fn test_add_at_end_on_list_16_8_7() {
        let list = FlList::from_values_with_shape(create_vec(0, 31), &[16, 8, 7])
            .add_at_end(Value::Integer(31));
        verify(&list, 0, 32);
        assert_eq!(vec![16, 16], bucket_sizes(&list));
    }

    #[test]
    fn test_add_at_start_on_list_with_big_first_bucket() {
        let list = FlList::from_values_with_shape(create_vec(1, 100), &[99])
            .add_at_start(Value::Integer(0));
        verify(&list, 0, 100);
        assert_eq!(vec![1, 99], bucket_sizes(&list));
    }

    #[test]
    fn test_add_at_end_on_list_with_big_last_bucket() {
        let list = FlList::from_values_with_shape(create_vec(0, 99), &[99])
            .add_at_end(Value::Integer(99));
        verify(&list, 0, 100);
        assert_eq!(vec![99, 1], bucket_sizes(&list));
    }

    #[test]
    fn test_add_at_start_on_list_with_smaller_buckets_at_end() {
        let list = FlList::from_values_with_shape(create_vec(1, 40), &[7, 24, 8])
            .add_at_start(Value::Integer(0));
        verify(&list, 0, 40);
        assert_eq!(vec![32, 8], bucket_sizes(&list));
    }

    #[test]
    fn test_add_at_end_on_list_with_smaller_buckets_at_start() {
        let list = FlList::from_values_with_shape(create_vec(0, 39), &[8, 24, 7])
            .add_at_end(Value::Integer(39));
        verify(&list, 0, 40);
        assert_eq!(vec![8, 32], bucket_sizes(&list));
    }

    #[test]
    fn test_add_at_start_with_big_carry() {
        let list = create(1, 101).append(&create(101, 201)).add_at_start(Value::Integer(0));
        verify(&list, 0, 201);
    }

    #[test]
    fn test_add_at_end_with_big_carry() {
        let list = create(0, 100).append(&create(100, 200)).add_at_end(Value::Integer(200));
        verify(&list, 0, 201);
    }

    #[test]
    fn test_add_at_start_on_long_array() {
        let list = create(1, 101).add_at_start(Value::Integer(0));
        verify(&list, 0, 101);
    }

    #[test]
    fn test_add_at_end_on_long_array() {
        let list = create(0, 100).add_at_end(Value::Integer(100));
        verify(&list, 0, 101);
    }

    #[test]
    fn test_add_at_start_with_start_overflow_in_big_bucket() {
        let list = create(1, 9).append(&create(9, 109)).add_at_start(Value::Integer(0));
        verify(&list, 0, 109);
    }

    #[test]
    fn test_add_at_end_with_start_overflow_in_big_bucket() {
        let list = create(0, 100).append(&create(100, 109)).add_at_end(Value::Integer(109));
        verify(&list, 0, 110);
    }

    #[test]
    fn test_add_at_start_with_start_overflow_in_small_bucket() {
        let list = create(1, 9).append(&create(9, 17)).add_at_start(Value::Integer(0));
        verify(&list, 0, 17);
    }

    #[test]
    fn test_add_at_end_with_start_overflow_in_small_bucket() {
        let list = create(0, 8).append(&create(8, 16)).add_at_end(Value::Integer(16));
        verify(&list, 0, 17);
    }
}
