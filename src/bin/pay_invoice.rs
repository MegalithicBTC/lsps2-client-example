use std::{env, thread, time::Duration};
use ldk_node::lightning_invoice::Bolt11Invoice;
use ldk_node::payment::PaymentStatus; // available in ldk-node 0.6.x
use megalith_lsps2::setup_node;

fn main() {
	// ── paths ──────────────────────────────────────────────────────────────
	let storage_dir = "tmp".to_string();
	let log_path = format!("{}/pay_invoice.log", storage_dir);

	// ── setup node ─────────────────────────────────────────────────────────
	let node = setup_node(storage_dir, log_path.clone());

	// ── arg: <bolt11-invoice> ─────────────────────────────────────────────
	let invoice_str = env::args().nth(1).unwrap_or_else(|| {
		eprintln!("usage: pay_invoice <bolt11-invoice>");
		std::process::exit(1);
	});
	let invoice: Bolt11Invoice = invoice_str.parse().unwrap_or_else(|e| {
		eprintln!("invalid BOLT11 invoice: {e}");
		std::process::exit(2);
	});
	let amount_msat = invoice.amount_milli_satoshis().unwrap_or_else(|| {
		eprintln!("invoice has no amount; please supply an invoice with an embedded amount");
		std::process::exit(2);
	});

	// small headroom for fees
	let need_msat = amount_msat + amount_msat / 50 + 10_000;

	// ── wait for usable channel + outbound liquidity ───────────────────────
	loop {
		let chans = node.list_channels();
		let mut total_out_msat: u64 = 0;
		let mut ready = 0usize;
		let mut usable = 0usize;

		if chans.is_empty() {
			println!("[wait] no channels yet; sleeping…");
		}

		for c in &chans {
			if c.is_channel_ready { ready += 1; }
			if c.is_usable { usable += 1; }
			total_out_msat = total_out_msat.saturating_add(c.outbound_capacity_msat);

			let scid = c.short_channel_id.map(|v| v.to_string()).unwrap_or_else(|| "-".into());
			println!(
				"[chan] scid={} ready={} usable={} out_msat={} in_msat={} capacity_sat={}",
				scid,
				c.is_channel_ready,
				c.is_usable,
				c.outbound_capacity_msat,
				c.inbound_capacity_msat,
				c.channel_value_sats
			);
		}

		println!(
			"[wait] ready_chans={} usable_chans={} total_out_msat={} need_msat={}",
			ready, usable, total_out_msat, need_msat
		);

		if usable > 0 && total_out_msat >= need_msat {
			break;
		}
		thread::sleep(Duration::from_secs(2));
	}

	// ── send + watch status; retry on async failure ────────────────────────
	let mut attempt: u32 = 0;
	let mut backoff = 2u64;
	let mut current_payment_id = loop {
		attempt += 1;
		match node.bolt11_payment().send(&invoice, None) {
			Ok(pid) => {
				println!("attempt={} sent id={pid:?}", attempt);
				break pid;
			}
			Err(e) => {
				eprintln!("attempt={} immediate send failed: {e}; retrying in {}s", attempt, backoff);
				thread::sleep(Duration::from_secs(backoff));
				if backoff < 30 { backoff = (backoff * 2).min(30); }
			}
		}
	};

	println!("Logs ➜ {log_path}");

	loop {
		// Poll payments to see if our id succeeded/failed after the fact
		let payments = node.list_payments();
		let mut seen = false;
		for p in payments {
			if p.id == current_payment_id {
				seen = true;
				match p.status {
					PaymentStatus::Succeeded => {
						println!("payment id={:?} SUCCEEDED amount_msat={}", p.id, p.amount_msat.unwrap_or(0));
						loop { thread::sleep(Duration::from_secs(30)); }
					}
					PaymentStatus::Failed => {
						eprintln!("payment id={:?} FAILED; retrying in {}s", p.id, backoff);
						thread::sleep(Duration::from_secs(backoff));
						if backoff < 30 { backoff = (backoff * 2).min(30); }

						attempt += 1;
						match node.bolt11_payment().send(&invoice, None) {
							Ok(new_id) => {
								println!("attempt={} re-sent id={new_id:?}", attempt);
								current_payment_id = new_id;
							}
							Err(e) => {
								eprintln!("attempt={} re-send failed immediately: {e}", attempt);
							}
						}
					}
					PaymentStatus::Pending => {
						// In flight; keep waiting
					}
				}
			}
		}

		// If we didn't see our payment (very old ldk-node), just sleep
		if !seen {
			println!("[watch] payment not listed yet; sleeping…");
		}

		thread::sleep(Duration::from_secs(2));
	}
}
