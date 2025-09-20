use std::{env, time::Duration};
use ldk_node::lightning_invoice::Bolt11Invoice;
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

	// Zero-amount invoices require specifying an amount; this bin expects invoices with an amount.
	if invoice.amount_milli_satoshis().is_none() {
		eprintln!("invoice has no amount; please supply an invoice with an embedded amount");
		std::process::exit(2);
	}

	// ── pay ────────────────────────────────────────────────────────────────
	let payment_id = node.bolt11_payment().send(&invoice, None).unwrap_or_else(|e| {
		eprintln!("payment send failed: {e}");
		std::process::exit(3);
	});

	println!("PAYMENT SENT id={payment_id:?}");
	println!("Logs ➜ {log_path}");

	// keep running so background processing/HTLC settlement can complete
	loop {
		std::thread::sleep(Duration::from_secs(30));
	}
}