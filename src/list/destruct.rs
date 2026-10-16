use std::{fs::TryLockError::Error, mem::MaybeUninit, sync::Arc};

use crate::{FlError, list::{BASE_SIZE, FACTOR}};

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
                values: self.buckets[0].values[1..].iter().cloned().collect(),
            };
            Ok(Self::from_buckets_sized(
                self.buckets.len(),
                std::iter::once(new_first).chain(self.buckets[1..].iter().map(|b| Bucket {
                    values: Arc::clone(&b.values),
                })),
            ))
        } else {
			/*
			   // First bucket is too large, split it according "ideal" shape
			   int count = shape[0].length - 1;
			   int additionalBuckets = -1;
			   int bucketFillSize = BASE_SIZE / 2;
			   while (count > 0) {
				   additionalBuckets++;
				   count -= bucketFillSize;
				   bucketFillSize *= FACTOR;
			   }
			   FplValue[][] bucketsDst = new FplValue[shape.length + additionalBuckets][];
			   bucketFillSize = BASE_SIZE / 2;
			   bucketsDst[0] = copyOfRange(shape[0], 1, 1 + bucketFillSize);
			   int dstIdx = 1;
			   count = shape[0].length - 1 - bucketFillSize;
			   int inBucketIdx = bucketFillSize + 1;
			   while (count > 0) {
				   bucketFillSize *= FACTOR;
				   if (bucketFillSize > count) {
					   bucketFillSize = count;
				   }
				   bucketsDst[dstIdx] = copyOfRange(shape[0], inBucketIdx, inBucketIdx + bucketFillSize);
				   dstIdx++;
				   inBucketIdx += bucketFillSize;
				   count -= bucketFillSize;
			   }
			   int srcIdx = 1;
			   while (dstIdx < bucketsDst.length) {
				   bucketsDst[dstIdx++] = shape[srcIdx++];
			   }
			   return new FplList(bucketsDst);
			*/
            let first_values = &self.buckets[0].values;
            let count = first_values.len() - 1;
            let mut src_idx: usize = 1;
            let split_buckets = std::iter::successors(
                Some(BASE_SIZE / 2),
                |&s| s.checked_mul(FACTOR),
            )
            .scan(count, |remaining, size| {
                if *remaining == 0 {
                    None
                } else {
                    let take = size.min(*remaining);
                    *remaining -= take;
                    Some(take)
                }
            })
            .map(move |size| {
                let bucket = Bucket {
                    values: first_values[src_idx..src_idx + size].iter().cloned().collect(),
                };
                src_idx += size;
                bucket
            });

            let buckets: Arc<[Bucket]> = split_buckets
                .chain(self.buckets[1..].iter().map(|b| Bucket {
                    values: Arc::clone(&b.values),
                }))
                .collect();
            Ok(FlList { buckets })
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::super::tests::{create, verify};
    use super::*;
    use crate::Value;

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
}
