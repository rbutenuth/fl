use std::sync::Arc;

use crate::Value;

mod construct;
mod access;

#[derive(Debug)]
pub struct FlList {
    buckets: Arc<[Bucket]>,
}


#[derive(Debug)]
struct Bucket {
    values: Arc<[Value]>,
}

impl Clone for FlList {
    fn clone(&self) -> Self {
        Self { buckets: Arc::clone(&self.buckets) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn create_vec(from: usize, to: usize) -> Vec<Value> {
        let mut values: Vec<Value> = Vec::new();
        for i in from..to {
            values.push(Value::Integer(i as i64));
        }
        values
    }

    pub fn create(from: usize, to: usize) -> FlList {
        FlList::from_values(create_vec(from, to))
    }
}
