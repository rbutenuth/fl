use std::mem::MaybeUninit;
use std::sync::Arc;

use crate::{FlError, Value, list::BASE_SIZE};

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
                        let merged_bucket = Bucket {
                            values: last_bucket.values.iter()
                                .chain(first_bucket.values.iter())
                                .cloned()
                                .collect(),
                        };
                        let buckets: Arc<[Bucket]> = self.buckets.iter().take(self.buckets.len() - 1)
                            .map(|b| Bucket { values: Arc::clone(&b.values) })
                            .chain(std::iter::once(merged_bucket))
                            .chain(list.buckets.iter().skip(1).map(|b| Bucket { values: Arc::clone(&b.values) }))
                            .collect();
                        FlList { buckets }
                    }
                } else {
                    if FlList::needs_reshaping(total_buckets, total_size) {
                        Self::merge(self, list, total_size)
                    } else {
                        let buckets: Arc<[Bucket]> = self.buckets.iter()
                            .chain(list.buckets.iter())
                            .map(|b| Bucket { values: Arc::clone(&b.values) })
                            .collect();
                        FlList { buckets }
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

}

#[cfg(test)]
mod tests {
    use crate::Value;

use super::super::tests::*;
    use super::*;

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
}
