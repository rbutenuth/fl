use std::{mem::MaybeUninit, sync::Arc};

use crate::{FlError};

use super::{Bucket, FlList};

impl FlList {

	pub fn remove_first(&self) -> Result<FlList, FlError> {
		self.check_not_empty("remove_first on empty list")?;
		Ok(Self::from_iterator_values(self.into_iter().skip(1)))
	}
/*
	public FplList removeFirst() throws EvaluationException {
		checkNotEmpty();
		if (shape[0].length == 1) {
			return new FplList(copyOfRange(shape, 1, shape.length));
		}
		if (shape[0].length <= BASE_SIZE + 1) {
			FplValue[][] bucketsDst = copyOf(shape, shape.length);
			bucketsDst[0] = copyOfRange(shape[0], 1, shape[0].length);
			return new FplList(bucketsDst);
		}
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
	}
 */    
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::Value;
    use super::*;
    use super::super::tests::{create, verify};

    #[test]
    fn test_remove_first_size_one() {
        let list = FlList::from_value(Value::Text(Arc::from("")));
        let rest = list.remove_first().unwrap();
        assert_eq!(0, rest.len());
    }

    #[test]
    fn test_remove_first_size_two() {
        let list = FlList::from_value(Value::Integer(1))
            .append(&FlList::from_value(Value::Integer(2)));
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