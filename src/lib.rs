use std::{str::FromStr, sync::Arc, env};
use ldk_node::bitcoin::{secp256k1::PublicKey, Network};
use ldk_node::config::{AnchorChannelsConfig, Config, EsploraSyncConfig, BackgroundSyncConfig};
use ldk_node::logger::LogLevel;
use ldk_node::Builder;

pub fn setup_node(storage_dir: String, log_path: String) -> Arc<ldk_node::Node> {
    // Load environment variables from .env file if present
    let _ = dotenvy::dotenv();

    // ── base config ────────────────────────────────────────────────────────
    let mut cfg = Config::default();
    
    // Network configuration from environment
    let network_str = env::var("NETWORK").unwrap_or_else(|_| "bitcoin".to_string());
    cfg.network = match network_str.to_lowercase().as_str() {
        "bitcoin" => Network::Bitcoin,
        "testnet" => Network::Testnet,
        "regtest" => Network::Regtest,
        "signet" => Network::Signet,
        _ => {
            eprintln!("Warning: Unknown network '{}', defaulting to Bitcoin", network_str);
            Network::Bitcoin
        }
    };

    // LSPS2 peer configuration from environment (required)
    let lsp_pubkey_str = env::var("LSP_PUBKEY")
        .expect("LSP_PUBKEY must be set in .env or environment");
    let lsp_pubkey = PublicKey::from_str(&lsp_pubkey_str)
        .expect("Invalid LSP_PUBKEY format");

    let mut anchor_cfg = AnchorChannelsConfig::default();
    anchor_cfg.trusted_peers_no_reserve.push(lsp_pubkey);
    cfg.anchor_channels_config = Some(anchor_cfg);

    // ── builder ────────────────────────────────────────────────────────────
    let mut builder = Builder::from_config(cfg);
    
    // Log level configuration from environment
    let log_level_str = env::var("LOG_LEVEL").unwrap_or_else(|_| "Debug".to_string());
    let log_level = match log_level_str.to_lowercase().as_str() {
        "trace" => LogLevel::Trace,
        "debug" => LogLevel::Debug,
        "info" => LogLevel::Info,
        "warn" => LogLevel::Warn,
        "error" => LogLevel::Error,
        _ => {
            eprintln!("Warning: Unknown log level '{}', defaulting to Debug", log_level_str);
            LogLevel::Debug
        }
    };

    // Esplora API URL from environment (required)
    let esplora_url = env::var("ESPLORA_API_URL")
        .expect("ESPLORA_API_URL must be set in .env or environment");

    // Configure Esplora sync intervals to reduce API rate limiting
    let mut sync_config = EsploraSyncConfig::default();
    sync_config.background_sync_config = Some(BackgroundSyncConfig {
        onchain_wallet_sync_interval_secs: 120,      // 2 minutes (default: 60)
        lightning_wallet_sync_interval_secs: 60,     // 1 minute (default: 30)
        fee_rate_cache_update_interval_secs: 300,    // 5 minutes (default: 60)
    });

    // LSP address from environment (required)
    let lsp_address = env::var("LSP_ADDRESS")
        .expect("LSP_ADDRESS must be set in .env or environment");

    builder
        .set_storage_dir_path(storage_dir)
        .set_filesystem_logger(Some(log_path), Some(log_level))
        .set_chain_source_esplora(esplora_url, Some(sync_config))
        .set_liquidity_source_lsps2(
            lsp_pubkey,
            lsp_address.parse().expect("Invalid LSP_ADDRESS format"),
            None,
        );

    // ── build & start ──────────────────────────────────────────────────────
    let node = Arc::new(builder.build().unwrap());
    
    // Start node with better error handling for fee estimation timeouts
    if let Err(e) = node.start() {
        eprintln!("WARNING: Node startup encountered an issue: {}", e);
        eprintln!("This is often due to fee estimation timeout from the Esplora API.");
        eprintln!("The node may still be functional for some operations.");
        eprintln!("Consider using a different ESPLORA_API_URL or try again later.");
        eprintln!();
        // Continue anyway - many operations can still work
    }

    // Ctrl-C → clean shutdown
    {
        let n = Arc::clone(&node);
        ctrlc::set_handler(move || {
            let _ = n.stop();
            std::process::exit(0);
        })
        .unwrap();
    }

    node
}