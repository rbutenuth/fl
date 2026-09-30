use std::{mem::MaybeUninit, sync::Arc};

use crate::{FlError};

use super::{Bucket, FlList};

impl FlList {

	pub fn remove_first(&self) -> Result<FlList, FlError> {
		self.check_not_empty("remove_first on empty list")?;
		if self.buckets[0].values.iter().len() == 1 {

		}
        let u_buckets: Arc<[MaybeUninit<Bucket>]> = Arc::new_uninit_slice(1);

        Ok(FlList {
            buckets: unsafe { u_buckets.assume_init() },
        }
)
	}
/*
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
//    use super::*;
//    use super::super::tests::{create, verify};
}