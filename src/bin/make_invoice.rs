use ldk_node::lightning_invoice::{Bolt11InvoiceDescription, Description};
use megalith_lsps2::setup_node;

fn main() {
    // ── paths ──────────────────────────────────────────────────────────────
    let storage_dir = "tmp".to_string();
    let log_path = format!("{}/make_invoice.log", storage_dir);

    // ── setup node ─────────────────────────────────────────────────────────
    let node = setup_node(storage_dir, log_path.clone());

    // ── create invoice for 10,000 satoshis ─────────────────────────────────
    let desc = Bolt11InvoiceDescription::Direct(
        Description::new("test-invoice-10000-sats".into()).unwrap(),
    );
    let invoice = node
        .bolt11_payment()
        .receive(10_000_000, &desc, 3_600)
        .unwrap();

    println!("INVOICE for 10,000 satoshis:\n{invoice}\n");
    println!("Logs ➜ {log_path}");

    // keep running to allow invoice to be paid
    loop {
        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}