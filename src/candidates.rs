/// Collection of candidates, in the same order as the `csv`.
#[derive(Debug, Clone)]
pub struct Candidates(Vec<String>);

impl Candidates {
    /// Creates a new instance of `Candidates` from a `Vec<String>`.
    pub fn new(candidates : Vec<String>) -> Self {
        Candidates(candidates)
    }

    /// Gets a candidate's name based on their index.
    pub fn get(&self, candidate : usize) -> Option<&String> {
        self.0.get(candidate)
    }

    /// Returns the number of candidates.
    pub fn len(&self) -> usize {
        self.0.len()
    }
}
