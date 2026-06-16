#[cfg(not(test))]
pub(crate) const DEBOUNCE_MS: u64 = 1_500;
#[cfg(test)]
pub(crate) const DEBOUNCE_MS: u64 = 200;
