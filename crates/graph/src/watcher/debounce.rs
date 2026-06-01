#[cfg(not(test))]
pub(crate) const WATCH_INTERVAL_MS: u64 = 1_000;
#[cfg(test)]
pub(crate) const WATCH_INTERVAL_MS: u64 = 100;

#[cfg(not(test))]
pub(crate) const DEBOUNCE_MS: u64 = 1_500;
#[cfg(test)]
pub(crate) const DEBOUNCE_MS: u64 = 200;
