use std::{env, process, thread, time::Duration};
use ldk_node::bitcoin::{Address, Network};
use megalith_lsps2::setup_node;

fn main() {
	// ── args ───────────────────────────────────────────────────────────────
	let addr_str = env::args().nth(1).unwrap_or_else(|| {
		eprintln!("usage: sweep_onchain <bitcoin-address> [--no-reserves]");
		process::exit(1);
	});
	let drain_including_anchor_reserves = env::args().any(|a| a == "--no-reserves");

	// ── start node ────────────────────────────────────────────────────────
	let storage_dir = "tmp".to_string();
	let log_path = format!("{}/sweep_onchain.log", storage_dir);
	let node = setup_node(storage_dir, log_path.clone());

	// get node network first to validate address
	let node_net: Network = node.config().network;
	
	// parse and validate address for the correct network
	let addr: Address = addr_str.parse::<Address<_>>().unwrap_or_else(|e| {
		eprintln!("invalid address: {e}");
		process::exit(2);
	}).require_network(node_net).unwrap_or_else(|e| {
		eprintln!("address is not valid for network {:?}: {e}", node_net);
		process::exit(2);
	});

	// ── check initial balance (before sync) ───────────────────────────────
	println!("Checking current balances...");
	let initial_balances = node.list_balances();
	println!("Initial on-chain balances (may not be fully synced):");
	println!("  Total:            {} sats", initial_balances.total_onchain_balance_sats);
	println!("  Spendable:        {} sats", initial_balances.spendable_onchain_balance_sats);
	println!("  Lightning total:  {} sats", initial_balances.total_lightning_balance_sats);
	println!();

	// ── wait until wallets are usable (and fee cache filled) ──────────────
	// node.start() already launched background tasks; we just force a sync and retry briefly if chain source isn't ready yet
	let mut attempts = 0;
	loop {
		match node.sync_wallets() {
			Ok(()) => break,
			Err(e) => {
				attempts += 1;
				if attempts > 10 {
					eprintln!("wallet sync still failing after retries: {e}");
					process::exit(3);
				}
				eprintln!("wallet sync not ready: {e}; retrying…");
				thread::sleep(Duration::from_secs(2));
			}
		}
	}

	// ── check and display current balance ─────────────────────────────────
	let balances = node.list_balances();
	println!("Current on-chain balances:");
	println!("  Total:            {} sats", balances.total_onchain_balance_sats);
	println!("  Spendable:        {} sats", balances.spendable_onchain_balance_sats);
	println!("  Lightning total:  {} sats", balances.total_lightning_balance_sats);
	println!();

	if balances.spendable_onchain_balance_sats == 0 {
		eprintln!("No spendable on-chain balance to sweep!");
		process::exit(5);
	}

	// ── sweep ─────────────────────────────────────────────────────────────
	// If retain_reserves=true we leave anchor reserves; with --no-reserves we attempt a full drain.
	let retain_reserves = !drain_including_anchor_reserves;
	match node.onchain_payment().send_all_to_address(&addr, retain_reserves, None) {
		Ok(txid) => {
			println!("SWEEP BROADCAST txid={txid}");
			println!("Logs ➜ {log_path}");
			println!("\nKeeping node online to monitor transaction...");
			
			// Keep node running to allow transaction to be broadcast and monitored
			loop {
				thread::sleep(Duration::from_secs(60));
			}
		}
		Err(e) => {
			eprintln!("sweep failed: {e}");
			process::exit(4);
		}
	}
}
