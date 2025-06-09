// Minimal lib.rs - just includes the generated Zngur bindings

#[rustfmt::skip]
mod generated;

/// Application utilities for data processing
pub struct AppUtils;

impl AppUtils {
    /// Process a vector of numbers and return a summary value (sum of even numbers)
    pub fn process_numbers(numbers: std::vec::Vec<i32>) -> i32 {
        numbers.into_iter().filter(|&x| x % 2 == 0).sum()
    }
}
