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
            let num_split = Self::split_sizes(count).count();
            let total_buckets = num_split + self.buckets.len() - 1;
            let mut src_idx: usize = 1;
            Ok(Self::from_buckets_sized(
                total_buckets,
                Self::split_sizes(count)
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
		Ok(Self::from_iterator_values(self.into_iter().take(self.len() - 1)))
	}

    fn split_sizes(count: usize) -> impl Iterator<Item = usize> {
        std::iter::successors(Some(BASE_SIZE / 2), |&s| s.checked_mul(FACTOR)).scan(
            count,
            |rem, size| {
                if *rem == 0 {
                    None
                } else {
                    let take = size.min(*rem);
                    *rem -= take;
                    Some(take)
                }
            },
        )
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
}
