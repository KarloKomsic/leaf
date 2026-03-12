use crate::domain::{
    self,
    document::{self, Document},
};

#[derive(Debug)]
pub struct Library {
    pub documents: Vec<Document>,
}

impl Library {
    pub fn new(documents: Vec<Document>) -> Self {
        Self { documents }
    }

    pub fn document_count(&self) -> usize {
        self.documents.len()
    }
}
