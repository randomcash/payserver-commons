//! Integration tests for rate providers using a mocked HTTP backend.

use rates::{
    CoinGeckoRateProvider, ExchangeRate, FallbackRateProvider, KrakenRateProvider, RateError,
    RateProvider, RateProviderConfig,
};
use rust_decimal::Decimal;
use std::sync::Arc;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Real Kraken JSON payload for ETHUSD ticker.
const KRAKEN_ETHUSD_RESPONSE: &str = r#"{
    "error": [],
    "result": {
        "XETHZUSD": {
            "a": ["2456.78000", "1", "1.000"],
            "b": ["2456.77000", "1", "1.000"],
            "c": ["2456.78000", "0.10000000"],
            "v": ["12345.67890000", "98765.43210000"],
            "p": ["2450.12345", "2445.67890"],
            "t": [1234, 5678],
            "l": ["2400.00000", "2380.00000"],
            "h": ["2500.00000", "2520.00000"],
            "o": "2440.00000"
        }
    }
}"#;

/// Kraken error response for unknown pair.
const KRAKEN_UNKNOWN_PAIR: &str = r#"{
    "error": ["EQuery:Unknown asset pair"],
    "result": null
}"#;

/// CoinGecko response for ethereum/usd.
const COINGECKO_ETH_USD_RESPONSE: &str = r#"{
    "ethereum": {
        "usd": 2456.78
    }
}"#;

#[tokio::test]
async fn test_kraken_rate_parsing() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/Ticker"))
        .and(query_param("pair", "ETHUSD"))
        .respond_with(ResponseTemplate::new(200).set_body_string(KRAKEN_ETHUSD_RESPONSE))
        .mount(&server)
        .await;

    let provider = KrakenRateProvider::new(Some(server.uri()));
    let rate = provider.get_rate("ETH", "USD").await.unwrap();

    assert_eq!(rate.from, "ETH");
    assert_eq!(rate.to, "USD");
    assert_eq!(rate.rate, Decimal::new(245678000, 5)); // 2456.78000
}

#[tokio::test]
async fn test_kraken_fiat_to_crypto_inverts() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/Ticker"))
        .and(query_param("pair", "ETHUSD"))
        .respond_with(ResponseTemplate::new(200).set_body_string(KRAKEN_ETHUSD_RESPONSE))
        .mount(&server)
        .await;

    let provider = KrakenRateProvider::new(Some(server.uri()));
    let rate = provider.get_rate("USD", "ETH").await.unwrap();

    assert_eq!(rate.from, "USD");
    assert_eq!(rate.to, "ETH");
    // 1/2456.78 ≈ 0.000407...
    assert!(rate.rate > Decimal::ZERO);
    assert!(rate.rate < Decimal::new(1, 0));
}

#[tokio::test]
async fn test_kraken_unknown_pair() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/Ticker"))
        .respond_with(ResponseTemplate::new(200).set_body_string(KRAKEN_UNKNOWN_PAIR))
        .mount(&server)
        .await;

    let provider = KrakenRateProvider::new(Some(server.uri()));
    let result = provider.get_rate("XYZ", "ABC").await;

    assert!(matches!(result, Err(RateError::UnsupportedPair { .. })));
}

#[tokio::test]
async fn test_kraken_network_error() {
    // Point to a non-existent server
    let provider = KrakenRateProvider::new(Some("http://127.0.0.1:1".to_string()));
    let result = provider.get_rate("ETH", "USD").await;

    assert!(matches!(result, Err(RateError::Network(_))));
}

#[tokio::test]
async fn test_coingecko_rate_parsing() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/simple/price"))
        .and(query_param("ids", "ethereum"))
        .and(query_param("vs_currencies", "usd"))
        .respond_with(ResponseTemplate::new(200).set_body_string(COINGECKO_ETH_USD_RESPONSE))
        .mount(&server)
        .await;

    let provider = CoinGeckoRateProvider::new(Some(server.uri()));
    let rate = provider.get_rate("ETH", "USD").await.unwrap();

    assert_eq!(rate.from, "ETH");
    assert_eq!(rate.to, "USD");
    assert_eq!(rate.rate, Decimal::new(245678, 2)); // 2456.78
}

#[tokio::test]
async fn test_coingecko_fiat_to_crypto_inverts() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/simple/price"))
        .respond_with(ResponseTemplate::new(200).set_body_string(COINGECKO_ETH_USD_RESPONSE))
        .mount(&server)
        .await;

    let provider = CoinGeckoRateProvider::new(Some(server.uri()));
    let rate = provider.get_rate("USD", "ETH").await.unwrap();

    assert_eq!(rate.from, "USD");
    assert_eq!(rate.to, "ETH");
    assert!(rate.rate > Decimal::ZERO);
    assert!(rate.rate < Decimal::ONE);
}

#[tokio::test]
async fn test_fallback_provider_with_mock() {
    let kraken_server = MockServer::start().await;
    let coingecko_server = MockServer::start().await;

    // Kraken returns 500
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&kraken_server)
        .await;

    // CoinGecko returns valid data
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string(COINGECKO_ETH_USD_RESPONSE))
        .mount(&coingecko_server)
        .await;

    let primary = Arc::new(KrakenRateProvider::new(Some(kraken_server.uri())));
    let secondary = Arc::new(CoinGeckoRateProvider::new(Some(coingecko_server.uri())));
    let fallback = FallbackRateProvider::new(primary, secondary);

    let rate = fallback.get_rate("ETH", "USD").await.unwrap();
    assert_eq!(rate.rate, Decimal::new(245678, 2));
}

#[tokio::test]
async fn test_config_creates_composed_provider() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/Ticker"))
        .respond_with(ResponseTemplate::new(200).set_body_string(KRAKEN_ETHUSD_RESPONSE))
        .mount(&server)
        .await;

    let config = RateProviderConfig {
        provider: "kraken".to_string(),
        api_url: Some(server.uri()),
        fallback_provider: None,
        cache_ttl_secs: 0,
    };

    let provider = config.create_provider();
    let rate = provider.get_rate("ETH", "USD").await.unwrap();
    assert_eq!(rate.rate, Decimal::new(245678000, 5));
}

#[tokio::test]
async fn test_exchange_rate_staleness() {
    let fresh = ExchangeRate {
        from: "USD".to_string(),
        to: "ETH".to_string(),
        rate: Decimal::new(5, 4),
        timestamp: chrono::Utc::now(),
    };
    assert!(!fresh.is_stale(chrono::Duration::seconds(60)));

    let old = ExchangeRate {
        from: "USD".to_string(),
        to: "ETH".to_string(),
        rate: Decimal::new(5, 4),
        timestamp: chrono::Utc::now() - chrono::Duration::seconds(120),
    };
    assert!(old.is_stale(chrono::Duration::seconds(60)));
    assert!(!old.is_stale(chrono::Duration::seconds(300)));
}
