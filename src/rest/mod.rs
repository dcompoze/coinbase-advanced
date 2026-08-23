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

/// Split an inclusive candle time range into non-overlapping windows
/// that each fit the per-request candle limit.
pub(crate) fn candle_windows(start: u64, end: u64, granularity_seconds: u64) -> Vec<(u64, u64)> {
    debug_assert!(granularity_seconds > 0);
    // Both window bounds are inclusive, so a full window spans limit - 1 steps.
    let span = (crate::constants::MAX_CANDLES_PER_REQUEST - 1) * granularity_seconds;
    let mut windows = Vec::new();
    let mut window_start = start;
    while window_start <= end {
        let window_end = std::cmp::min(window_start.saturating_add(span), end);
        windows.push((window_start, window_end));
        window_start = match window_end.checked_add(granularity_seconds) {
            Some(next) => next,
            None => break,
        };
    }
    windows
}

/// Fetch candles for a range longer than the per-request limit.
///
/// Splits the range into windows, fetches each window with `fetch`, and
/// returns the combined result deduplicated and sorted by start time.
pub(crate) async fn fetch_candles_windowed<F, Fut>(
    params: crate::models::GetCandlesParams,
    fetch: F,
) -> crate::error::Result<Vec<crate::models::Candle>>
where
    F: Fn(crate::models::GetCandlesParams) -> Fut,
    Fut: std::future::Future<Output = crate::error::Result<Vec<crate::models::Candle>>>,
{
    use crate::error::Error;
    use crate::models::GetCandlesParams;

    let start: u64 = params
        .start
        .parse()
        .map_err(|_| Error::request("start must be a unix timestamp"))?;
    let end: u64 = params
        .end
        .parse()
        .map_err(|_| Error::request("end must be a unix timestamp"))?;

    let mut candles = Vec::new();
    for (window_start, window_end) in candle_windows(start, end, params.granularity.seconds()) {
        let window_params = GetCandlesParams::new(
            &params.product_id,
            window_start.to_string(),
            window_end.to_string(),
            params.granularity,
        );
        candles.extend(fetch(window_params).await?);
    }

    candles.sort_by_key(|c| c.start.parse::<u64>().unwrap_or(0));
    candles.dedup_by(|a, b| a.start == b.start);
    Ok(candles)
}

#[cfg(test)]
mod tests {
    use super::candle_windows;

    #[test]
    fn test_candle_windows() {
        // 1001 one-minute candles split into 350-candle inclusive windows.
        let windows = candle_windows(0, 60_000, 60);
        assert_eq!(
            windows,
            vec![(0, 20_940), (21_000, 41_940), (42_000, 60_000)]
        );
        // Each full window holds exactly 350 candles.
        assert_eq!(20_940 / 60 + 1, 350);
        assert_eq!(candle_windows(0, 600, 60), vec![(0, 600)]);
        assert_eq!(candle_windows(600, 600, 60), vec![(600, 600)]);
        assert!(candle_windows(601, 600, 60).is_empty());
    }
}
