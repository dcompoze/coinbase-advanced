//! API service modules for the Coinbase Advanced Trade API.

mod accounts;
mod convert;
mod data;
mod fees;
mod futures;
mod order_builder;
mod orders;
mod payment_methods;
mod perpetuals;
mod portfolios;
mod products;
mod public;

pub use accounts::AccountsApi;
pub use convert::ConvertApi;
pub use data::DataApi;
pub use fees::FeesApi;
pub use futures::FuturesApi;
pub use order_builder::{
    LimitOrderGtcBuilder, LimitOrderGtdBuilder, MarketOrderBuilder, StopLimitOrderGtcBuilder,
};
pub use orders::OrdersApi;
pub use payment_methods::PaymentMethodsApi;
pub use perpetuals::PerpetualsApi;
pub use portfolios::PortfoliosApi;
pub use products::ProductsApi;
pub use public::{PublicApi, ServerTime};

/// Split a candle time range into windows that fit the per-request candle limit.
pub(crate) fn candle_windows(start: u64, end: u64, granularity_seconds: u64) -> Vec<(u64, u64)> {
    let span = crate::constants::MAX_CANDLES_PER_REQUEST * granularity_seconds;
    let mut windows = Vec::new();
    let mut window_start = start;
    while window_start < end {
        let window_end = std::cmp::min(window_start + span, end);
        windows.push((window_start, window_end));
        window_start = window_end;
    }
    windows
}

#[cfg(test)]
mod tests {
    use super::candle_windows;

    #[test]
    fn test_candle_windows() {
        // 1000 one-minute candles split into 350-candle windows.
        let windows = candle_windows(0, 60_000, 60);
        assert_eq!(
            windows,
            vec![(0, 21_000), (21_000, 42_000), (42_000, 60_000)]
        );
        // A range within the limit is a single window.
        assert_eq!(candle_windows(0, 600, 60), vec![(0, 600)]);
        // An empty range has no windows.
        assert!(candle_windows(600, 600, 60).is_empty());
    }
}
