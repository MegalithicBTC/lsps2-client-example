Basic usage of the Megalith LSPS2 service, can be demonstrated in the following way:

1. Clone this repository

2. **Configure the environment** (optional): Copy `.env.example` to `.env` and customize the configuration:

   ```bash
   cp .env.example .env
   # Edit .env with your preferred LSP settings
   ```

3. `cargo run --bin megalith_lsps2` .. this generates a JIT invoice

4. pay the invoice normally, or with MPP. LSP opens a channel.

5. `cargo run --bin make_invoice` This generates a private invoice from the Client LDK node.

You can pay the invoice from any node and the balance will reflect in the LDK client.

5. `cargo run --bin pay_invoice <bolt11-invoice>` Pay an existing BOLT11 invoice using the LDK client.

## Available Binaries

- **`megalith_lsps2`**: Main binary that creates JIT invoices via LSPS2 service
- **`make_invoice`**: Creates standard invoices (10,000 satoshis) from the LDK client
- **`pay_invoice`**: Pays BOLT11 invoices with embedded amounts using the LDK client

## Configuration

The application uses environment variables for configuration. You can either:

1. **Use environment variables directly**:

   ```bash
   export LSP_PUBKEY="your_lsp_public_key_here"
   export LSP_ADDRESS="your_lsp_ip:port"
   export ESPLORA_API_URL="https://your-esplora-instance.com/api"
   cargo run --bin megalith_lsps2
   ```

2. **Use a `.env` file** (recommended):
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   cargo run --bin megalith_lsps2
   ```

### Configuration Variables

| Variable          | Description                           | Default Value                                          |
| ----------------- | ------------------------------------- | ------------------------------------------------------ |
| `LSP_PUBKEY`      | LSPS2 liquidity provider's public key | `<your_lsp_public_key>`                                |
| `LSP_ADDRESS`     | LSP's IP address and port             | `<your_lsp_address>`                                   |
| `NETWORK`         | Bitcoin network to use                | `bitcoin` (options: bitcoin, testnet, regtest, signet) |
| `ESPLORA_API_URL` | Blockchain data API endpoint          | `https://blockstream.info/api`                         |
| `LOG_LEVEL`       | Application log level                 | `Debug` (options: Trace, Debug, Info, Warn, Error)     |
