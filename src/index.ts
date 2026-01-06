import { pumpFunMonitor } from './monitors/pump.fun.monitor';
import { tokenValidator } from './monitors/token.validator';
import { buyer } from './traders/buyer';
import { seller } from './traders/seller';
import { solanaClient } from './utils/solana.client';
import { safetyChecker } from './utils/safety.checker';
import { settings } from './config/settings';
import { TradingStatus } from './config/constants';
import { logInfo, logError, logWarning, logTrade } from './utils/logger';
import { TokenAnalysis } from './types/token.interface';

class PumpFunSniperBot {
  private status: TradingStatus = TradingStatus.STOPPED;
  private analysisInterval: NodeJS.Timeout | null = null;
  private healthCheckInterval: NodeJS.Timeout | null = null;

  constructor() {
    this.setupEventHandlers();
    this.setupHealthChecks();
  }

  /**
   * Start the bot
   */
  async start(): Promise<void> {
    try {
      logInfo('Starting Pump.fun Sniper Bot...');

      // Validate configuration
      await this.validateConfiguration();

      // Start monitoring
      await pumpFunMonitor.startMonitoring();
      this.status = TradingStatus.ACTIVE;

      // Start position analysis for automated selling
      this.startPositionAnalysis();

      logInfo('Pump.fun Sniper Bot started successfully', {
        simulationMode: settings.simulationMode,
        rpcUrl: settings.rpcUrl
      });

      // Log initial balance
      if (!settings.simulationMode) {
        const balance = await solanaClient.getBalance();
        logInfo('Initial wallet balance', { balance: `${balance.toFixed(4)} SOL` });
      }

    } catch (error) {
      logError('Failed to start bot', error);
      this.status = TradingStatus.STOPPED;
      throw error;
    }
  }

  /**
   * Stop the bot
   */
  async stop(): Promise<void> {
    try {
      logInfo('Stopping Pump.fun Sniper Bot...');

      this.status = TradingStatus.STOPPED;

      // Stop monitoring
      await pumpFunMonitor.stopMonitoring();

      // Stop position analysis
      if (this.analysisInterval) {
        clearInterval(this.analysisInterval);
        this.analysisInterval = null;
      }

      // Stop health checks
      if (this.healthCheckInterval) {
        clearInterval(this.healthCheckInterval);
        this.healthCheckInterval = null;
      }

      // Emergency stop traders
      buyer.emergencyStop();
      seller.emergencyStop();

      logInfo('Pump.fun Sniper Bot stopped successfully');

    } catch (error) {
      logError('Error stopping bot', error);
      throw error;
    }
  }

  /**
   * Pause trading (monitoring continues)
   */
  pause(): void {
    this.status = TradingStatus.PAUSED;
    logInfo('Trading paused - monitoring continues');
  }

  /**
   * Resume trading
   */
  resume(): void {
    this.status = TradingStatus.ACTIVE;
    logInfo('Trading resumed');
  }

  /**
   * Setup event handlers for new tokens
   */
  private setupEventHandlers(): void {
    pumpFunMonitor.onNewToken(async (event) => {
      if (this.status !== TradingStatus.ACTIVE) {
        return;
      }

      try {
        logInfo('Processing new token launch', {
          tokenAddress: event.tokenAddress.toString(),
          creator: event.creator.toString(),
          age: `${(Date.now() - event.timestamp.getTime()) / 1000}s ago`
        });

        // Analyze the token
        const analysis = await tokenValidator.analyzeToken(
          event.tokenAddress,
          event.bondingCurveAddress
        );

        if (!analysis) {
          logWarning('Failed to analyze token, skipping');
          return;
        }

        // Check if token meets our criteria
        if (this.shouldTradeToken(analysis)) {
          await this.executeTrade(analysis);
        } else {
          logInfo('Token filtered out', {
            symbol: analysis.token.symbol,
            marketCap: analysis.metrics.marketCap,
            safetyScore: analysis.safety.score,
            opportunityScore: analysis.opportunities.score
          });
        }

      } catch (error) {
        logError('Error processing new token', error, {
          tokenAddress: event.tokenAddress.toString()
        });
      }
    });
  }

  /**
   * Check if token should be traded
   */
  private shouldTradeToken(analysis: TokenAnalysis): boolean {
    // Check safety score
    if (analysis.safety.score < 60) { // Minimum safety score
      return false;
    }

    // Check opportunity score
    if (analysis.opportunities.score < 50) { // Minimum opportunity score
      return false;
    }

    // Check market cap range
    if (analysis.metrics.marketCap < settings.minMarketCap ||
        analysis.metrics.marketCap > settings.maxMarketCap) {
      return false;
    }

    // Check liquidity
    if (analysis.metrics.liquidity < settings.minLiquidity) {
      return false;
    }

    // Additional safety check
    const safetyResult = safetyChecker.performSafetyCheck(
      analysis.token.address,
      analysis.token.creator
    );

    return safetyResult.passed;
  }

  /**
   * Execute trade for a token
   */
  private async executeTrade(analysis: TokenAnalysis): Promise<void> {
    try {
      logTrade('Executing trade for token', {
        symbol: analysis.token.symbol,
        marketCap: analysis.metrics.marketCap,
        safetyScore: analysis.safety.score,
        opportunityScore: analysis.opportunities.score
      });

      // Execute buy
      const buyResult = await buyer.executeBuy({
        tokenAddress: analysis.token.address,
        bondingCurveAddress: analysis.bondingCurve.address,
        analysis
      });

      if (buyResult && buyResult.success) {
        // Create position for tracking
        const position = {
          tokenAddress: analysis.token.address,
          tokenSymbol: analysis.token.symbol,
          amount: buyResult.amount,
          entryPrice: buyResult.price,
          currentPrice: buyResult.price,
          pnl: 0,
          pnlPercentage: 0,
          openedAt: new Date(),
          lastUpdated: new Date(),
          takeProfitPrice: buyResult.price * (1 + settings.takeProfitPercentage / 100),
          stopLossPrice: buyResult.price * (1 - settings.stopLossPercentage / 100),
          status: 'OPEN' as const
        };

        // Add to seller for tracking
        seller.addPosition(position);

        logTrade('Trade completed successfully', {
          symbol: analysis.token.symbol,
          amount: buyResult.amount.toString(),
          price: buyResult.price,
          totalValue: buyResult.totalValue
        });

      } else {
        logWarning('Buy failed', {
          symbol: analysis.token.symbol,
          error: buyResult?.error
        });
      }

    } catch (error) {
      logError('Error executing trade', error, {
        symbol: analysis.token.symbol
      });
    }
  }

  /**
   * Start position analysis for automated selling
   */
  private startPositionAnalysis(): void {
    this.analysisInterval = setInterval(async () => {
      if (this.status === TradingStatus.ACTIVE) {
        try {
          await seller.checkAutomatedSells();
        } catch (error) {
          logError('Error in position analysis', error);
        }
      }
    }, 10000); // Check every 10 seconds
  }

  /**
   * Setup health checks
   */
  private setupHealthChecks(): void {
    this.healthCheckInterval = setInterval(async () => {
      try {
        await this.performHealthCheck();
      } catch (error) {
        logError('Health check failed', error);
      }
    }, 60000); // Check every minute
  }

  /**
   * Perform health check
   */
  private async performHealthCheck(): Promise<void> {
    const healthStatus = {
      timestamp: new Date(),
      status: this.status,
      monitoring: pumpFunMonitor.getHealthStatus(),
      buyer: buyer.getStatus(),
      seller: seller.getStatus(),
      safety: safetyChecker.getSafetyStats(),
      simulationMode: settings.simulationMode
    };

    // Check Solana connection
    const solanaHealthy = await solanaClient.healthCheck();
    if (!solanaHealthy) {
      logWarning('Solana connection health check failed');
    }

    // Log health status periodically
    if (Math.random() < 0.1) { // Log ~10% of health checks
      logInfo('Health check', healthStatus);
    }
  }

  /**
   * Validate configuration
   */
  private async validateConfiguration(): Promise<void> {
    if (!settings.rpcUrl) {
      throw new Error('RPC_URL is required');
    }

    if (!settings.simulationMode && !settings.privateKey) {
      throw new Error('PRIVATE_KEY is required when not in simulation mode');
    }

    // Test Solana connection
    const healthy = await solanaClient.healthCheck();
    if (!healthy) {
      throw new Error('Cannot connect to Solana network');
    }

    logInfo('Configuration validated successfully');
  }

  /**
   * Get bot status
   */
  getStatus() {
    return {
      status: this.status,
      monitoring: pumpFunMonitor.getHealthStatus(),
      buyer: buyer.getStatus(),
      seller: seller.getStatus(),
      settings: {
        simulationMode: settings.simulationMode,
        buyAmountSol: settings.buyAmountSol,
        maxSlippage: settings.maxSlippage,
        takeProfitPercentage: settings.takeProfitPercentage,
        stopLossPercentage: settings.stopLossPercentage
      }
    };
  }
}

// Export bot instance
export const bot = new PumpFunSniperBot();

// Handle graceful shutdown
process.on('SIGINT', async () => {
  logInfo('Received SIGINT, shutting down gracefully...');
  await bot.stop();
  process.exit(0);
});

process.on('SIGTERM', async () => {
  logInfo('Received SIGTERM, shutting down gracefully...');
  await bot.stop();
  process.exit(0);
});

// Start bot if this file is run directly
if (require.main === module) {
  bot.start().catch((error) => {
    logError('Failed to start bot', error);
    process.exit(1);
  });
}
