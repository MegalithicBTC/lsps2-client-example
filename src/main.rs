
use ldk_node::lightning_invoice::{Bolt11InvoiceDescription, Description};
use megalith_lsps2::setup_node;

fn main() {
    // ── paths ──────────────────────────────────────────────────────────────
    let storage_dir = "tmp".to_string();
    let log_path = format!("{}/ldk_node.log", storage_dir);

    // ── setup node ─────────────────────────────────────────────────────────
    let node = setup_node(storage_dir, log_path.clone());

    // ── create invoice ─────────────────────────────────────────────────────
    let desc = Bolt11InvoiceDescription::Direct(
        Description::new("test-megalith-lsps2".into()).unwrap(),
    );
    let invoice = node
        .bolt11_payment()
        .receive_via_jit_channel(25_000_000, &desc, 3_600, None)
        .unwrap();

    println!("JIT INVOICE:\n{invoice}\n");
    println!("Logs ➜ {log_path}");

    // keep running so the LSP can actually open the channel
    loop {
        std::thread::sleep(std::time::Duration::from_secs(120));
    }
}