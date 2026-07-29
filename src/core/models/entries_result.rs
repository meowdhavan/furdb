use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntriesResult {
    result_count: usize,
    results: Vec<Entry>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    index: usize,
    data: Vec<u128>,
}

impl EntriesResult {
    pub fn new(data: &[Entry]) -> Self {
        Self {
            result_count: data.len(),
            results: data.to_vec(),
        }
    }

    pub fn get_result_count(&self) -> usize {
        self.result_count
    }

    pub fn get_results(&self) -> Vec<Entry> {
        self.results.to_owned()
    }
}

impl Entry {
    pub fn new(index: usize, data: Vec<u128>) -> Self {
        Self { index, data }
    }

    pub fn get_index(&self) -> usize {
        self.index
    }

    pub fn get_data(&self) -> Vec<u128> {
        self.data.to_owned()
    }
}
