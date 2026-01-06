# Solana Pump.fun Sniper Bot (Rust Edition) 🦀

A high-performance Solana Pump.fun sniper bot written in Rust, designed for detecting and trading new token launches with lightning-fast execution.

## ⚠️ Disclaimer

This software is for educational purposes only. Trading cryptocurrencies involves significant risk of loss. The authors are not responsible for any financial losses incurred through the use of this software. Always trade responsibly and never invest more than you can afford to lose.

## 🚀 Features

- **Blazing Fast**: Written in Rust for maximum performance and minimal latency
- **Real-time Monitoring**: WebSocket-based monitoring for instant token launch detection
- **Smart Filtering**: Advanced token analysis with safety checks and opportunity scoring
- **Automated Trading**: Configurable auto-buy and auto-sell with take-profit/stop-loss
- **Safety First**: Comprehensive security checks and blacklist management
- **Simulation Mode**: Test strategies without risking real funds
- **Production Ready**: Async architecture, error handling, and graceful shutdown

## 🛠️ Tech Stack

- **Rust** - High-performance systems programming language
- **Tokio** - Async runtime for concurrent operations
- **Solana SDK** - Native Solana blockchain integration
- **WebSocket** - Real-time program log monitoring
- **Serde** - Serialization/deserialization
- **Tracing** - Structured logging

## 📦 Installation

### Prerequisites

- **Rust**: Install from [rustup.rs](https://rustup.rs/)
- **Solana CLI** (optional, for development): Install from [docs.solana.com](https://docs.solana.com/cli/install-solana-cli-tools)

### Build from Source

1. **Clone the repository**:
   ```bash
   git clone <repository-url>
   cd solana-pumpfun-sniper-bot
   ```

2. **Install dependencies**:
   ```bash
   cargo build --release
   ```

## ⚙️ Configuration

1. **Copy environment file**:
   ```bash
   cp env.example .env
   ```

2. **Configure your settings in `.env`**:

   ### Required Settings
   ```env
   # Solana RPC (Use Helius for best performance)
   RPC_URL=https://mainnet.helius-rpc.com/?api-key=YOUR_HELIUS_API_KEY
   WS_URL=wss://mainnet.helius-rpc.com/?api-key=YOUR_HELIUS_API_KEY

   # Your trading wallet private key (KEEP SECURE!)
   PRIVATE_KEY=your_wallet_private_key_here
   ```

   ### Trading Configuration
   ```env
   # Buy Settings
   BUY_AMOUNT_SOL=0.1
   MAX_SLIPPAGE=25

   # Auto-Sell Settings
   TAKE_PROFIT_PERCENTAGE=100
   STOP_LOSS_PERCENTAGE=30
   TRAILING_STOP_LOSS_PERCENTAGE=10

   # Safety Settings
   MIN_LIQUIDITY=5
   MIN_MARKET_CAP=1000
   MAX_MARKET_CAP=25000
   TRADING_COOLDOWN_MS=5000
   MAX_TRADES_PER_HOUR=10
   ```

   ### Optional Settings
   ```env
   # Simulation mode (set to true for testing)
   SIMULATION_MODE=true

   # Logging level
   RUST_LOG=solana_pumpfun_sniper=debug

   # Telegram notifications (future feature)
   TELEGRAM_BOT_TOKEN=your_bot_token
   TELEGRAM_CHAT_ID=your_chat_id
   ```

## 🔑 Wallet Security

**CRITICAL**: Never share your private key or `.env` file with anyone.

### Recommended Setup:
1. **Use a dedicated trading wallet** - Don't use your main wallet
2. **Start with small amounts** - Test with minimal funds (0.01-0.05 SOL per trade)
3. **Enable 2FA** - If using exchanges for funding
4. **Monitor transactions** - Use Solscan to verify trades

## 🚀 Usage

### Development Mode (Recommended First)
```bash
# Enable simulation mode in .env
SIMULATION_MODE=true

# Run in debug mode
RUST_LOG=solana_pumpfun_sniper=debug cargo run
```

### Production Mode
```bash
# Disable simulation mode in .env
SIMULATION_MODE=false

# Build optimized binary
cargo build --release

# Run the bot
./target/release/solana-pumpfun-sniper
```

### Testing
```bash
cargo test
```

## 📊 How It Works

### 1. Real-time Monitoring
- WebSocket connection to Solana mainnet
- Subscribes to Pump.fun program logs
- Instant detection of new token launches

### 2. Token Analysis
- **Safety Checks**: Authority verification, honeypot detection, creator analysis
- **Metrics Calculation**: Market cap, liquidity, holder distribution
- **Opportunity Scoring**: Combines safety and market factors

### 3. Automated Trading
- **Lightning Fast**: Sub-millisecond execution using native Solana SDK
- **Risk Management**: Trading cooldowns, maximum loss limits
- **Position Tracking**: Real-time P&L monitoring

### 4. Safety Features
- **Authority Checks**: Verifies mint/freeze authorities are properly set
- **Liquidity Verification**: Ensures sufficient liquidity before trading
- **Blacklist Management**: Automatic filtering of suspicious tokens/creators

## 📈 Configuration Guide

### Conservative Settings (Recommended for beginners)
```env
BUY_AMOUNT_SOL=0.05
MIN_LIQUIDITY=10
MIN_MARKET_CAP=5000
MAX_MARKET_CAP=25000
TAKE_PROFIT_PERCENTAGE=50
STOP_LOSS_PERCENTAGE=20
TRADING_COOLDOWN_MS=10000
MAX_TRADES_PER_HOUR=5
```

### Aggressive Settings (Higher risk)
```env
BUY_AMOUNT_SOL=0.2
MIN_LIQUIDITY=2
MIN_MARKET_CAP=500
MAX_MARKET_CAP=100000
TAKE_PROFIT_PERCENTAGE=200
STOP_LOSS_PERCENTAGE=50
TRADING_COOLDOWN_MS=2000
MAX_TRADES_PER_HOUR=20
```

## 🔍 Monitoring & Logs

### Log Levels
```bash
# Debug mode (very verbose)
RUST_LOG=solana_pumpfun_sniper=debug

# Info mode (recommended)
RUST_LOG=solana_pumpfun_sniper=info

# Warning only
RUST_LOG=solana_pumpfun_sniper=warn
```

### Health Monitoring
The bot performs automatic health checks every 60 seconds, logging:
- Solana RPC connection status
- WebSocket monitoring status
- Active trading positions
- Daily trade statistics

## 🛡️ Safety Recommendations

1. **Start Small**: Begin with 0.01-0.05 SOL per trade
2. **Use Simulation**: Always test strategies in simulation mode first
3. **Monitor Logs**: Regularly check logs for unusual activity
4. **Diversify**: Don't allocate all funds to one bot
5. **Stay Updated**: Keep dependencies updated for security patches

## 🐛 Troubleshooting

### Common Issues

**"WebSocket connection failed"**
- Check your WS_URL configuration
- Verify internet connection and firewall settings
- Try a different RPC provider

**"Transaction simulation failed"**
- Verify sufficient SOL balance (account for fees)
- Check slippage settings are reasonable
- Ensure token meets minimum requirements

**"Token filtered out"**
- Adjust filtering criteria in config
- Check token meets minimum safety requirements
- Review log messages for filtering reasons

**"Rate limited"**
- Reduce MAX_TRADES_PER_HOUR
- Increase TRADING_COOLDOWN_MS
- Consider using a paid RPC provider

### Debug Mode
```bash
RUST_LOG=solana_pumpfun_sniper=debug cargo run
```

## 📚 Architecture

```
src/
├── main.rs           # Application entry point
├── lib.rs            # Library interface
├── config.rs         # Configuration management
├── types.rs          # Data structures
├── monitors/         # Token launch monitoring
├── traders/          # Trading logic
└── utils/            # Utilities and helpers
```

### Key Components

- **PumpFunMonitor**: WebSocket-based token launch detection
- **TokenAnalyzer**: Safety and opportunity analysis
- **Trader**: Buy/sell execution and position management
- **TransactionBuilder**: Solana instruction construction
- **SolanaClient**: RPC and WebSocket client wrapper

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Write tests for new functionality
4. Ensure code compiles (`cargo check`)
5. Run tests (`cargo test`)
6. Commit your changes (`git commit -m 'Add amazing feature'`)
7. Push to the branch (`git push origin feature/amazing-feature`)
8. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## ⚠️ Risk Warning

Cryptocurrency trading is highly speculative and involves substantial risk. Past performance does not guarantee future results. The use of automated trading bots does not eliminate risk. Always conduct your own research and consider consulting with financial advisors before engaging in cryptocurrency trading.

---

**Happy Trading! 🚀**