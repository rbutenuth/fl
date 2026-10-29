use std::{sync::Arc};

#[derive(Debug)]
pub struct Symbol {
    pub name: Arc<str>,
    pub comment: Option<Arc<str>>,
}

impl Clone for Symbol {
    fn clone(&self) -> Self {
        Self {
            name: Arc::clone(&self.name),
            comment: match &self.comment {
                Some(c) => Some(Arc::clone(&c)),
                None => None,
            },
        }
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        &self.name == &other.name
    }
}

impl Symbol {
    pub fn new(name: Arc<str>, comment: Option<Arc<str>>) -> Symbol {
        Symbol {
            name: Arc::clone(&name),
            comment: match comment {
                Some(c) => Some(Arc::clone(&c)),
                None => None,
            }
        }
    }
}
