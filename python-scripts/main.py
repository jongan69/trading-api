# Penny Stock Screener with Kelly Criterion Analysis
# Comprehensive stock screening, portfolio optimization, and options analysis

# Imports
from finvizfinance.screener.technical import Technical
from finvizfinance.quote import finvizfinance
from finvizfinance.screener.technical import Technical
import yfinance as yf
import pandas as pd
import numpy as np
from scipy.stats import norm
import requests
from bs4 import BeautifulSoup
import praw
from datetime import datetime
import time
import os
from dotenv import load_dotenv

# Load environment variables
load_dotenv()

# Reddit API Configuration
REDDIT_CLIENT_ID = os.getenv('REDDIT_CLIENT_ID')
REDDIT_CLIENT_SECRET = os.getenv('REDDIT_CLIENT_SECRET')
REDDIT_USERNAME = os.getenv('REDDIT_USERNAME')
REDDIT_PASSWORD = os.getenv('REDDIT_PASSWORD')

# Configuration
load_dotenv()
REDDIT_CLIENT_ID = os.getenv('REDDIT_CLIENT_ID')
REDDIT_CLIENT_SECRET = os.getenv('REDDIT_CLIENT_SECRET')
REDDIT_USERNAME = os.getenv('REDDIT_USERNAME')
REDDIT_PASSWORD = os.getenv('REDDIT_PASSWORD')

# Constants
DEFAULT_PORTFOLIO_VALUE = 1000
DEFAULT_SCALING_FACTOR = 0.5
DEFAULT_RISK_AVERSION = 1.0
DEFAULT_LOOKBACK_DAYS = 252
DEFAULT_CONFIDENCE_LEVEL = 0.95
DEFAULT_RISK_FREE_RATE = 0.05
MAX_POSITION_SIZE = 0.20
MIN_ALLOCATION = 10
OPTIONS_MAX_POSITION = 0.05
OPTIONS_MIN_ALLOCATION = 5
KELLY_WEIGHT = 0.4
SORTINO_WEIGHT = 0.3
CALMAR_WEIGHT = 0.3

# Utility Functions
def get_screener_filters():
    return {'Market Cap.': 'Small ($300mln to $2bln)', 'Price': 'Under $10', 'Average Volume': 'Over 500K', 'Relative Volume': 'Over 1.5', 'Performance': 'Today Up'}

def get_reddit_subreddits():
    return ["pennystocks", "wallstreetbets", "stocks", "investing", "StockMarket"]

def get_known_penny_options():
    """Dynamically discover penny stocks with options available - NO hardcoded tickers"""
    penny_options_tickers = []
    
    print("🔍 Dynamically discovering penny stocks with options...")
    
    # Strategy 1: Use Finviz to find optionable penny stocks
    try:
        from finvizfinance.screener.overview import Overview
        
        fviz = Overview()
        filters_dict = {
            'Price': 'Under $5',
            'Market Cap.': '+Small (over $300mln)',
            'Average Volume': 'Over 500K',
            'Option/Short': 'Optionable'  # Only stocks with options
        }
        
        fviz.set_filter(filters_dict=filters_dict)
        df = fviz.screener_view(verbose=0)
        
        if df is not None and not df.empty:
            penny_options_tickers.extend(df['Ticker'].head(30).tolist())
            print(f"  Found {len(df)} optionable penny stocks from Finviz")
    
    except Exception as e:
        print(f"  Finviz penny options search failed: {e}")
    
    # Strategy 2: Check trending stocks from Reddit/social media
    try:
        trending = get_trending_penny_stocks()
        print(f"  Checking {len(trending)} trending stocks for options...")
        
        for ticker in trending:
            try:
                yft = yf.Ticker(ticker)
                current_price = yft.history(period="1d")['Close'].iloc[-1]
                
                if current_price < 10 and yft.options:
                    penny_options_tickers.append(ticker)
                    print(f"    Trending: {ticker} (${current_price:.2f}) has options")
                    
            except Exception:
                continue
                
    except Exception as e:
        print(f"  Trending stocks check failed: {e}")
    
    # Strategy 3: Scan Reddit mentions for optionable penny stocks
    try:
        reddit_stocks = get_reddit_trending_stocks()
        if reddit_stocks:
            print(f"  Checking {len(reddit_stocks)} Reddit stocks for options...")
            
            for ticker in reddit_stocks:
                try:
                    yft = yf.Ticker(ticker)
                    current_price = yft.history(period="1d")['Close'].iloc[-1]
                    
                    if current_price < 10 and yft.options:
                        penny_options_tickers.append(ticker)
                        print(f"    Reddit: {ticker} (${current_price:.2f}) has options")
                        
                except Exception:
                    continue
                    
    except Exception as e:
        print(f"  Reddit stocks check failed: {e}")
    
    # Strategy 4: Use additional Finviz filters to find more options
    try:
        # Search for slightly higher priced but still affordable stocks
        fviz2 = Overview()
        filters_dict2 = {
            'Price': '$5 to $10',
            'Market Cap.': '+Small (over $300mln)',
            'Average Volume': 'Over 100K',
            'Option/Short': 'Optionable'
        }
        
        fviz2.set_filter(filters_dict=filters_dict2)
        df2 = fviz2.screener_view(verbose=0)
        
        if df2 is not None and not df2.empty:
            penny_options_tickers.extend(df2['Ticker'].head(20).tolist())
            print(f"  Found {len(df2)} additional optionable stocks ($5-$10)")
    
    except Exception as e:
        print(f"  Additional Finviz search failed: {e}")
    
    # Remove duplicates and return
    penny_options_tickers = list(set(penny_options_tickers))
    print(f"📊 Total dynamic penny options discovered: {len(penny_options_tickers)}")
    
    return penny_options_tickers[:30]  # Return top 30

def validate_reddit_credentials():
    return all([REDDIT_CLIENT_ID, REDDIT_CLIENT_SECRET, REDDIT_USERNAME, REDDIT_PASSWORD])

def initialize_reddit_client():
    if not validate_reddit_credentials():
        return None
    try:
        return praw.Reddit(client_id=REDDIT_CLIENT_ID, client_secret=REDDIT_CLIENT_SECRET, user_agent="PennyStockScreener/1.0", username=REDDIT_USERNAME, password=REDDIT_PASSWORD)
    except Exception as e:
        print(f"Error initializing Reddit client: {e}")
        return None

def safe_divide(numerator, denominator, default=0):
    return numerator / denominator if denominator != 0 else default



# Kelly Criterion Functions
def calculate_kelly_fraction(p, g, l):
    """Calculate Kelly Criterion fraction for single asset allocation"""
    if l == 0:
        return 0
    q = 1 - p
    b = g / l
    kelly_fraction = (b * p - q) / b
    return max(0, kelly_fraction)

def calculate_portfolio_kelly(returns_matrix, risk_free_rate=DEFAULT_RISK_FREE_RATE):
    """Calculate Kelly Criterion for multi-asset portfolio"""
    try:
        excess_returns = returns_matrix - risk_free_rate
        mu = excess_returns.mean()
        sigma = excess_returns.cov()
        sigma_inv = np.linalg.inv(sigma.values)
        kelly_weights = sigma_inv @ mu.values
        kelly_weights = kelly_weights / np.sum(np.abs(kelly_weights))
        return pd.Series(kelly_weights, index=returns_matrix.columns)
    except Exception as e:
        print(f"Error calculating portfolio Kelly: {e}")
        return None

def estimate_stock_probabilities(ticker, lookback_days=DEFAULT_LOOKBACK_DAYS, with_confidence=False):
    """Estimate win probability and gain/loss parameters for a stock"""
    try:
        yft = yf.Ticker(ticker)
        hist = yft.history(period=f'{lookback_days}d')
        
        if len(hist) < 30:
            return None
        
        returns = hist['Close'].pct_change().dropna()
        p = (returns > 0).mean()
        gains = returns[returns > 0]
        losses = returns[returns < 0]
        
        if len(gains) == 0 or len(losses) == 0:
            return None
        
        g = gains.mean()
        l = abs(losses.mean())
        kelly_fraction = calculate_kelly_fraction(p, g, l)
        volatility = returns.std()
        sharpe_ratio = (returns.mean() - 0.05/252) / volatility if volatility > 0 else 0
        
        result = {
            'ticker': ticker,
            'win_probability': p,
            'avg_gain': g,
            'avg_loss': l,
            'kelly_fraction': kelly_fraction,
            'volatility': volatility,
            'sharpe_ratio': sharpe_ratio,
            'total_return': (hist['Close'].iloc[-1] / hist['Close'].iloc[0] - 1),
            'max_drawdown': calculate_max_drawdown(hist['Close'])
        }
        
        if with_confidence:
            # Add confidence intervals
            from scipy import stats
            wins = (returns > 0).sum()
            total = len(returns)
            p_hat = wins / total
            se = np.sqrt(p_hat * (1 - p_hat) / total)
            z_score = stats.norm.ppf((1 + DEFAULT_CONFIDENCE_LEVEL) / 2)
            margin_of_error = z_score * se
            sample_confidence = min(0.95, 0.5 + (total / 1000) * 0.4)
            
            result.update({
                'win_probability_confidence': sample_confidence,
                'win_probability_ci': (max(0, p_hat - margin_of_error), min(1, p_hat + margin_of_error)),
                'avg_gain_confidence': min(0.95, 0.5 + (len(gains) / 500) * 0.4),
                'avg_loss_confidence': min(0.95, 0.5 + (len(losses) / 500) * 0.4),
                'sample_size': len(returns)
            })
        
        return result
        
    except Exception as e:
        print(f"Error estimating probabilities for {ticker}: {e}")
        return None

def calculate_max_drawdown(prices):
    """Calculate maximum drawdown from peak"""
    peak = prices.expanding().max()
    drawdown = (prices - peak) / peak
    return drawdown.min()

def calculate_scaled_kelly(kelly_fraction, scaling_factor=DEFAULT_SCALING_FACTOR):
    """
    Calculate scaled Kelly fraction to reduce volatility
    
    Parameters:
    kelly_fraction: full Kelly fraction
    scaling_factor: fraction of Kelly to use (0.5 = half Kelly)
    
    Returns:
    scaled_fraction: scaled Kelly allocation
    """
    return kelly_fraction * scaling_factor

def calculate_portfolio_allocation(stocks_data, portfolio_value=DEFAULT_PORTFOLIO_VALUE, scaling_factor=DEFAULT_SCALING_FACTOR):
    """
    Calculate optimal portfolio allocation using Kelly Criterion
    
    Parameters:
    stocks_data: list of stock analysis dictionaries
    portfolio_value: total portfolio value in dollars
    scaling_factor: Kelly scaling factor (0.5 = half Kelly)
    
    Returns:
    dict: portfolio allocation details
    """
    allocations = []
    total_allocation = 0
    
    for stock in stocks_data:
        ticker = stock['Ticker']
        
        # Get Kelly analysis
        kelly_data = estimate_stock_probabilities(ticker)
        
        if kelly_data and kelly_data['kelly_fraction'] > 0:
            # Calculate scaled Kelly allocation
            scaled_kelly = calculate_scaled_kelly(kelly_data['kelly_fraction'], scaling_factor)
            
            # Calculate dollar allocation
            dollar_allocation = scaled_kelly * portfolio_value
            
            # Don't allocate more than MAX_POSITION_SIZE to any single stock
            max_allocation = portfolio_value * MAX_POSITION_SIZE
            dollar_allocation = min(dollar_allocation, max_allocation)
            
            if dollar_allocation > MIN_ALLOCATION:  # Only include if allocation > MIN_ALLOCATION
                allocations.append({
                    'ticker': ticker,
                    'current_price': stock['Current_Price'],
                    'kelly_fraction': kelly_data['kelly_fraction'],
                    'scaled_kelly': scaled_kelly,
                    'dollar_allocation': dollar_allocation,
                    'shares_to_buy': int(dollar_allocation / stock['Current_Price']),
                    'win_probability': kelly_data['win_probability'],
                    'avg_gain': kelly_data['avg_gain'],
                    'avg_loss': kelly_data['avg_loss'],
                    'volatility': kelly_data['volatility'],
                    'sharpe_ratio': kelly_data['sharpe_ratio'],
                    'doubling_score': stock['Doubling_Score'],
                    'reasons': stock['Reasons']
                })
                
                total_allocation += dollar_allocation
    
    # Sort by Kelly fraction (highest first)
    allocations.sort(key=lambda x: x['kelly_fraction'], reverse=True)
    
    return {
        'allocations': allocations,
        'total_allocated': total_allocation,
        'cash_remaining': portfolio_value - total_allocation,
        'allocation_percentage': (total_allocation / portfolio_value) * 100
    }

def calculate_dynamic_valuation_thresholds(results_df):
    """
    Calculate dynamic valuation thresholds based on actual data distribution
    
    Parameters:
    results_df: DataFrame with stock analysis results
    
    Returns:
    dict: dynamic thresholds for PE, PEG, and P/S ratios
    """
    thresholds = {}
    
    # PE Ratio thresholds
    pe_ratios = results_df[results_df['PE_Ratio'] > 0]['PE_Ratio']
    if not pe_ratios.empty:
        pe_mean = pe_ratios.mean()
        pe_std = pe_ratios.std()
        pe_median = pe_ratios.median()
        
        # Dynamic thresholds based on distribution
        thresholds['pe_low'] = max(10, min(pe_median * 0.7, 25))  # Conservative low threshold
        thresholds['pe_medium'] = max(20, min(pe_median * 1.2, 40))  # Medium threshold
        thresholds['pe_high'] = max(30, min(pe_median * 1.8, 60))  # High threshold
    else:
        # Fallback to standard thresholds if no data
        thresholds['pe_low'] = 20
        thresholds['pe_medium'] = 30
        thresholds['pe_high'] = 50
    
    # PEG Ratio thresholds
    peg_ratios = results_df[results_df['PEG_Ratio'] > 0]['PEG_Ratio']
    if not peg_ratios.empty:
        peg_mean = peg_ratios.mean()
        peg_median = peg_ratios.median()
        
        # Dynamic thresholds based on distribution
        thresholds['peg_undervalued'] = max(0.5, min(peg_median * 0.6, 1.0))  # Undervalued threshold
        thresholds['peg_fair'] = max(1.0, min(peg_median * 1.2, 2.0))  # Fair value threshold
        thresholds['peg_overvalued'] = max(1.5, min(peg_median * 1.8, 3.0))  # Overvalued threshold
    else:
        # Fallback to standard thresholds if no data
        thresholds['peg_undervalued'] = 1.0
        thresholds['peg_fair'] = 2.0
        thresholds['peg_overvalued'] = 3.0
    
    # P/S Ratio thresholds
    ps_ratios = results_df[results_df['Price_to_Sales'] > 0]['Price_to_Sales']
    if not ps_ratios.empty:
        ps_mean = ps_ratios.mean()
        ps_median = ps_ratios.median()
        
        # Dynamic thresholds based on distribution
        thresholds['ps_low'] = max(0.5, min(ps_median * 0.5, 1.5))  # Low threshold
        thresholds['ps_medium'] = max(1.0, min(ps_median * 1.0, 3.0))  # Medium threshold
        thresholds['ps_high'] = max(2.0, min(ps_median * 1.5, 5.0))  # High threshold
    else:
        # Fallback to standard thresholds if no data
        thresholds['ps_low'] = 1.0
        thresholds['ps_medium'] = 3.0
        thresholds['ps_high'] = 5.0
    
    return thresholds

def create_equal_weight_allocation(stocks_data, portfolio_value=DEFAULT_PORTFOLIO_VALUE, max_positions=5):
    """
    Create a simple equal-weight allocation when Kelly Criterion doesn't find suitable allocations
    
    Parameters:
    stocks_data: list of stock analysis dictionaries
    portfolio_value: total portfolio value in dollars
    max_positions: maximum number of positions to hold
    
    Returns:
    dict: portfolio allocation details
    """
    # Sort stocks by doubling score (highest first)
    sorted_stocks = sorted(stocks_data, key=lambda x: x['Doubling_Score'], reverse=True)
    
    # Take top stocks up to max_positions
    selected_stocks = sorted_stocks[:max_positions]
    
    if not selected_stocks:
        return {
            'allocations': [],
            'total_allocated': 0,
            'cash_remaining': portfolio_value,
            'allocation_percentage': 0
        }
    
    # Calculate equal weight allocation
    allocation_per_stock = portfolio_value / len(selected_stocks)
    allocations = []
    total_allocation = 0
    
    for stock in selected_stocks:
        ticker = stock['Ticker']
        current_price = stock['Current_Price']
        
        # Calculate shares to buy
        shares_to_buy = int(allocation_per_stock / current_price)
        actual_allocation = shares_to_buy * current_price
        
        allocations.append({
            'ticker': ticker,
            'current_price': current_price,
            'kelly_fraction': 0.0,  # Not calculated for equal weight
            'scaled_kelly': allocation_per_stock / portfolio_value,
            'dollar_allocation': actual_allocation,
            'shares_to_buy': shares_to_buy,
            'win_probability': 0.5,  # Default assumption
            'avg_gain': 0.1,  # Default assumption
            'avg_loss': 0.05,  # Default assumption
            'volatility': stock.get('Volatility', 0.1),
            'sharpe_ratio': 0.0,  # Not calculated for equal weight
            'doubling_score': stock['Doubling_Score'],
            'reasons': stock['Reasons']
        })
        
        total_allocation += actual_allocation
    
    return {
        'allocations': allocations,
        'total_allocated': total_allocation,
        'cash_remaining': portfolio_value - total_allocation,
        'allocation_percentage': (total_allocation / portfolio_value) * 100
    }

def calculate_risk_metrics(portfolio_allocation):
    """
    Calculate risk metrics for the portfolio allocation
    
    Parameters:
    portfolio_allocation: portfolio allocation dictionary
    
    Returns:
    dict: risk metrics
    """
    allocations = portfolio_allocation['allocations']
    
    if not allocations:
        return {}
    
    # Calculate portfolio-level metrics
    total_value = sum(alloc['dollar_allocation'] for alloc in allocations)
    
    # Weighted average metrics
    weighted_volatility = sum(
        alloc['volatility'] * (alloc['dollar_allocation'] / total_value) 
        for alloc in allocations
    )
    
    weighted_sharpe = sum(
        alloc['sharpe_ratio'] * (alloc['dollar_allocation'] / total_value) 
        for alloc in allocations
    )
    
    # Calculate expected return (simplified)
    expected_return = sum(
        alloc['win_probability'] * alloc['avg_gain'] * (alloc['dollar_allocation'] / total_value)
        for alloc in allocations
    )
    
    # Calculate maximum drawdown estimate (simplified)
    max_dd_estimate = weighted_volatility * 2  # Rough estimate
    
    return {
        'expected_return': expected_return,
        'portfolio_volatility': weighted_volatility,
        'portfolio_sharpe': weighted_sharpe,
        'max_drawdown_estimate': max_dd_estimate,
        'number_of_positions': len(allocations),
        'concentration_risk': max(alloc['dollar_allocation'] for alloc in allocations) / total_value if allocations else 0
    }

# =============================================================================
# CONFIDENCE-WEIGHTED KELLY CRITERION FUNCTIONS
# =============================================================================

def calculate_confidence_interval(returns, confidence_level=DEFAULT_CONFIDENCE_LEVEL):
    """
    Calculate confidence interval for probability estimates
    
    Parameters:
    returns: array of historical returns
    confidence_level: confidence level (e.g., 0.95 for 95%)
    
    Returns:
    dict: confidence interval statistics
    """
    try:
        from scipy import stats
        
        # Calculate win probability
        wins = (returns > 0).sum()
        total = len(returns)
        p_hat = wins / total
        
        # Calculate standard error
        se = np.sqrt(p_hat * (1 - p_hat) / total)
        
        # Calculate confidence interval
        z_score = stats.norm.ppf((1 + confidence_level) / 2)
        margin_of_error = z_score * se
        
        lower_bound = max(0, p_hat - margin_of_error)
        upper_bound = min(1, p_hat + margin_of_error)
        
        # Calculate confidence level based on sample size
        # More data = higher confidence
        sample_confidence = min(0.95, 0.5 + (total / 1000) * 0.4)  # 50% to 95% based on sample size
        
        return {
            'point_estimate': p_hat,
            'lower_bound': lower_bound,
            'upper_bound': upper_bound,
            'margin_of_error': margin_of_error,
            'confidence_level': confidence_level,
            'sample_confidence': sample_confidence,
            'sample_size': total
        }
        
    except Exception as e:
        print(f"Error calculating confidence interval: {e}")
        return None

def calculate_confidence_weighted_kelly(p_estimate, g_estimate, l_estimate, 
                                      p_confidence, g_confidence, l_confidence,
                                      risk_aversion=DEFAULT_RISK_AVERSION):
    """
    Calculate Kelly fraction weighted by confidence in estimates
    
    Parameters:
    p_estimate: point estimate of win probability
    g_estimate: point estimate of gain
    l_estimate: point estimate of loss
    p_confidence: confidence in probability estimate (0-1)
    g_confidence: confidence in gain estimate (0-1)
    l_confidence: confidence in loss estimate (0-1)
    risk_aversion: risk aversion factor (higher = more conservative)
    
    Returns:
    dict: confidence-weighted Kelly results
    """
    # Calculate base Kelly
    base_kelly = calculate_kelly_fraction(p_estimate, g_estimate, l_estimate)
    
    # Calculate confidence-weighted adjustments
    # Lower confidence = more conservative allocation
    confidence_factor = (p_confidence + g_confidence + l_confidence) / 3
    
    # Risk aversion adjustment
    risk_adjustment = 1 / (1 + risk_aversion * (1 - confidence_factor))
    
    # Calculate conservative Kelly using worst-case estimates
    # Use lower bound for gains, upper bound for losses
    conservative_p = p_estimate * p_confidence
    conservative_g = g_estimate * g_confidence
    conservative_l = l_estimate / l_confidence if l_confidence > 0 else l_estimate * 2
    
    conservative_kelly = calculate_kelly_fraction(conservative_p, conservative_g, conservative_l)
    
    # Calculate optimistic Kelly using best-case estimates
    optimistic_p = p_estimate + (1 - p_confidence) * 0.1  # Slight upward bias
    optimistic_g = g_estimate + (1 - g_confidence) * 0.05
    optimistic_l = l_estimate * (1 - (1 - l_confidence) * 0.5)
    
    optimistic_kelly = calculate_kelly_fraction(optimistic_p, optimistic_g, optimistic_l)
    
    # Final confidence-weighted Kelly
    confidence_weighted_kelly = base_kelly * confidence_factor * risk_adjustment
    
    return {
        'base_kelly': base_kelly,
        'confidence_weighted_kelly': confidence_weighted_kelly,
        'conservative_kelly': conservative_kelly,
        'optimistic_kelly': optimistic_kelly,
        'confidence_factor': confidence_factor,
        'risk_adjustment': risk_adjustment,
        'p_confidence': p_confidence,
        'g_confidence': g_confidence,
        'l_confidence': l_confidence
    }



def calculate_confidence_weighted_portfolio_allocation(stocks_data, portfolio_value=1000, 
                                                     scaling_factor=0.5, risk_aversion=1.0):
    """Calculate portfolio allocation using confidence-weighted Kelly Criterion"""
    allocations = []
    total_allocation = 0
    
    for stock in stocks_data:
        ticker = stock['Ticker']
        
        # Get confidence-weighted Kelly analysis
        kelly_data = estimate_stock_probabilities(ticker, with_confidence=True)
        
        if kelly_data and kelly_data.get('win_probability_confidence'):
            # Calculate confidence-weighted Kelly
            p_estimate = kelly_data['win_probability']
            g_estimate = kelly_data['avg_gain']
            l_estimate = kelly_data['avg_loss']
            p_confidence = kelly_data['win_probability_confidence']
            g_confidence = kelly_data['avg_gain_confidence']
            l_confidence = kelly_data['avg_loss_confidence']
            
            confidence_kelly = calculate_confidence_weighted_kelly(
                p_estimate, g_estimate, l_estimate,
                p_confidence, g_confidence, l_confidence,
                risk_aversion
            )
            
            confidence_weighted_kelly_value = confidence_kelly['confidence_weighted_kelly']
            
            if confidence_weighted_kelly_value <= 0:
                continue
            # Apply scaling
            scaled_kelly = calculate_scaled_kelly(confidence_weighted_kelly_value, scaling_factor)
            
            # Calculate dollar allocation
            dollar_allocation = scaled_kelly * portfolio_value
            
            # Position limits based on confidence
            confidence_factor = confidence_kelly['confidence_factor']
            max_allocation = portfolio_value * (0.20 * confidence_factor)
            dollar_allocation = min(dollar_allocation, max_allocation)
            
            # Minimum allocation based on confidence
            min_allocation = 10 * confidence_factor
            if dollar_allocation > min_allocation:
                allocations.append({
                    'ticker': ticker,
                    'current_price': stock['Current_Price'],
                    'base_kelly': kelly_data['kelly_fraction'],
                    'confidence_weighted_kelly': confidence_weighted_kelly_value,
                    'scaled_kelly': scaled_kelly,
                    'dollar_allocation': dollar_allocation,
                    'shares_to_buy': int(dollar_allocation / stock['Current_Price']),
                    'win_probability': kelly_data['win_probability'],
                    'win_probability_confidence': kelly_data['win_probability_confidence'],
                    'avg_gain': kelly_data['avg_gain'],
                    'avg_gain_confidence': kelly_data['avg_gain_confidence'],
                    'avg_loss': kelly_data['avg_loss'],
                    'avg_loss_confidence': kelly_data['avg_loss_confidence'],
                    'volatility': kelly_data['volatility'],
                    'confidence_factor': confidence_factor,
                    'sharpe_ratio': kelly_data['sharpe_ratio'],
                    'doubling_score': stock['Doubling_Score'],
                    'reasons': stock['Reasons'],
                    'sample_size': kelly_data['sample_size']
                })
                
                total_allocation += dollar_allocation
    
    # Sort by confidence-weighted Kelly fraction (highest first)
    allocations.sort(key=lambda x: x['confidence_weighted_kelly'], reverse=True)
    
    return {
        'allocations': allocations,
        'total_allocated': total_allocation,
        'cash_remaining': portfolio_value - total_allocation,
        'allocation_percentage': (total_allocation / portfolio_value) * 100
    }



# =============================================================================
# MEDALLION-STYLE UNIFIED RISK-REWARD FUNCTIONS
# =============================================================================

def calculate_sortino_ratio(returns, target_return=0.0, risk_free_rate=0.05):
    """
    Calculate Sortino Ratio focusing on downside volatility only
    
    Parameters:
    returns: array of returns
    target_return: target return (default 0)
    risk_free_rate: annual risk-free rate (default 5%)
    
    Returns:
    float: Sortino ratio
    """
    try:
        # Convert annual risk-free rate to daily
        daily_rf = (1 + risk_free_rate) ** (1/252) - 1
        
        # Calculate excess returns
        excess_returns = returns - daily_rf
        
        # Calculate downside deviation (only negative returns)
        downside_returns = np.minimum(excess_returns - target_return, 0)
        downside_deviation = np.sqrt(np.mean(downside_returns ** 2))
        
        # Calculate average excess return
        avg_excess_return = np.mean(excess_returns)
        
        # Sortino ratio
        if downside_deviation > 0:
            sortino_ratio = avg_excess_return / downside_deviation
        else:
            sortino_ratio = 0
            
        return sortino_ratio
        
    except Exception as e:
        print(f"Error calculating Sortino ratio: {e}")
        return 0

def calculate_calmar_ratio(returns, lookback_days=252):
    """
    Calculate Calmar Ratio (CAGR / Max Drawdown)
    
    Parameters:
    returns: array of returns
    lookback_days: number of days to calculate CAGR
    
    Returns:
    float: Calmar ratio
    """
    try:
        # Calculate cumulative returns
        cumulative_returns = (1 + returns).cumprod()
        
        # Calculate CAGR
        total_return = cumulative_returns.iloc[-1] - 1
        years = lookback_days / 252
        cagr = (1 + total_return) ** (1/years) - 1 if years > 0 else 0
        
        # Calculate maximum drawdown
        rolling_max = cumulative_returns.expanding().max()
        drawdown = (cumulative_returns - rolling_max) / rolling_max
        max_drawdown = abs(drawdown.min())
        
        # Calmar ratio
        if max_drawdown > 0:
            calmar_ratio = cagr / max_drawdown
        else:
            calmar_ratio = 0
            
        return calmar_ratio
        
    except Exception as e:
        print(f"Error calculating Calmar ratio: {e}")
        return 0

def calculate_unified_risk_reward_metric(ticker, lookback_days=252, risk_free_rate=0.05):
    """
    Calculate unified risk-reward metric combining Kelly, Sortino, and Calmar principles
    
    Parameters:
    ticker: stock symbol
    lookback_days: number of days to look back
    risk_free_rate: annual risk-free rate
    
    Returns:
    dict: unified risk-reward metrics
    """
    try:
        yft = yf.Ticker(ticker)
        hist = yft.history(period=f'{lookback_days}d')
        
        if len(hist) < 30:
            return None
        
        # Calculate daily returns
        returns = hist['Close'].pct_change().dropna()
        
        # Calculate Kelly Criterion
        kelly_data = estimate_stock_probabilities(ticker, lookback_days, with_confidence=True)
        
        if not kelly_data:
            return None
        
        # Calculate Sortino Ratio
        sortino_ratio = calculate_sortino_ratio(returns, 0.0, risk_free_rate)
        
        # Calculate Calmar Ratio
        calmar_ratio = calculate_calmar_ratio(returns, lookback_days)
        
        # Calculate Sharpe Ratio
        sharpe_ratio = kelly_data['sharpe_ratio']
        
        # Calculate volatility and other metrics
        volatility = returns.std()
        avg_return = returns.mean()
        max_drawdown = kelly_data['max_drawdown']
        
        # Calculate confidence-weighted Kelly
        p_estimate = kelly_data['win_probability']
        g_estimate = kelly_data['avg_gain']
        l_estimate = kelly_data['avg_loss']
        p_confidence = kelly_data.get('win_probability_confidence', 0.5)
        g_confidence = kelly_data.get('avg_gain_confidence', 0.5)
        l_confidence = kelly_data.get('avg_loss_confidence', 0.5)
        
        confidence_kelly = calculate_confidence_weighted_kelly(
            p_estimate, g_estimate, l_estimate,
            p_confidence, g_confidence, l_confidence,
            1.0  # risk_aversion
        )
        
        # Unified Risk-Reward Score (Medallion-inspired)
        # Combines Kelly, Sortino, and Calmar principles
        kelly_weight = 0.4  # Position sizing importance
        sortino_weight = 0.3  # Downside risk importance
        calmar_weight = 0.3  # Drawdown control importance
        
        # Normalize ratios to 0-1 scale (assuming reasonable ranges)
        kelly_score = min(1.0, max(0.0, confidence_kelly['confidence_weighted_kelly'] * 10))
        sortino_score = min(1.0, max(0.0, sortino_ratio / 2))  # Normalize to 0-2 range
        calmar_score = min(1.0, max(0.0, calmar_ratio / 3))    # Normalize to 0-3 range
        
        # Calculate unified score
        unified_score = (kelly_weight * kelly_score + 
                        sortino_weight * sortino_score + 
                        calmar_weight * calmar_score)
        
        # Risk-adjusted Kelly allocation
        risk_adjusted_kelly = confidence_kelly['confidence_weighted_kelly'] * unified_score
        
        return {
            'ticker': ticker,
            'unified_score': unified_score,
            'kelly_score': kelly_score,
            'sortino_score': sortino_score,
            'calmar_score': calmar_score,
            'kelly_ratio': confidence_kelly['confidence_weighted_kelly'],
            'sortino_ratio': sortino_ratio,
            'calmar_ratio': calmar_ratio,
            'sharpe_ratio': sharpe_ratio,
            'risk_adjusted_kelly': risk_adjusted_kelly,
            'volatility': volatility,
            'avg_return': avg_return,
            'max_drawdown': max_drawdown,
            'win_probability': kelly_data['win_probability'],
            'confidence_factor': confidence_kelly['confidence_factor'],
            'sample_size': len(returns)
        }
        
    except Exception as e:
        print(f"Error calculating unified risk-reward metric for {ticker}: {e}")
        return None

def calculate_medallion_style_portfolio_allocation(stocks_data, portfolio_value=1000, 
                                                 scaling_factor=0.5, risk_aversion=1.0):
    """
    Calculate portfolio allocation using Medallion-inspired unified risk-reward metric
    
    Parameters:
    stocks_data: list of stock analysis dictionaries
    portfolio_value: total portfolio value
    scaling_factor: Kelly scaling factor
    risk_aversion: risk aversion factor
    
    Returns:
    dict: Medallion-style portfolio allocation
    """
    allocations = []
    total_allocation = 0
    
    print("🔬 Calculating Medallion-style unified risk-reward metrics...")
    
    for stock in stocks_data:
        ticker = stock['Ticker']
        
        # Calculate unified risk-reward metric
        unified_data = calculate_unified_risk_reward_metric(ticker)
        
        if unified_data and unified_data['unified_score'] > 0.1:  # Minimum threshold
            # Use risk-adjusted Kelly for allocation
            risk_adjusted_kelly = unified_data['risk_adjusted_kelly']
            
            # Apply scaling and risk aversion
            scaled_kelly = calculate_scaled_kelly(risk_adjusted_kelly, scaling_factor)
            
            # Calculate dollar allocation
            dollar_allocation = scaled_kelly * portfolio_value
            
            # Position limits based on unified score
            max_allocation = portfolio_value * (0.20 * unified_data['unified_score'])
            dollar_allocation = min(dollar_allocation, max_allocation)
            
            # Minimum allocation based on unified score
            min_allocation = 10 * unified_data['unified_score']
            if dollar_allocation > min_allocation:
                allocations.append({
                    'ticker': ticker,
                    'current_price': stock['Current_Price'],
                    'unified_score': unified_data['unified_score'],
                    'kelly_score': unified_data['kelly_score'],
                    'sortino_score': unified_data['sortino_score'],
                    'calmar_score': unified_data['calmar_score'],
                    'risk_adjusted_kelly': risk_adjusted_kelly,
                    'scaled_kelly': scaled_kelly,
                    'dollar_allocation': dollar_allocation,
                    'shares_to_buy': int(dollar_allocation / stock['Current_Price']),
                    'kelly_ratio': unified_data['kelly_ratio'],
                    'sortino_ratio': unified_data['sortino_ratio'],
                    'calmar_ratio': unified_data['calmar_ratio'],
                    'sharpe_ratio': unified_data['sharpe_ratio'],
                    'volatility': unified_data['volatility'],
                    'avg_return': unified_data['avg_return'],
                    'max_drawdown': unified_data['max_drawdown'],
                    'win_probability': unified_data['win_probability'],
                    'confidence_factor': unified_data['confidence_factor'],
                    'doubling_score': stock['Doubling_Score'],
                    'reasons': stock['Reasons'],
                    'sample_size': unified_data['sample_size']
                })
                
                total_allocation += dollar_allocation
    
    # Sort by unified score (highest first)
    allocations.sort(key=lambda x: x['unified_score'], reverse=True)
    
    return {
        'allocations': allocations,
        'total_allocated': total_allocation,
        'cash_remaining': portfolio_value - total_allocation,
        'allocation_percentage': (total_allocation / portfolio_value) * 100
    }

def display_medallion_style_allocation(portfolio_allocation, portfolio_value=1000):
    """
    Display Medallion-style portfolio allocation results
    
    Parameters:
    portfolio_allocation: portfolio allocation dictionary
    portfolio_value: total portfolio value
    """
    print(f"\n🏆 MEDALLION-STYLE UNIFIED RISK-REWARD ALLOCATION (${portfolio_value:,})")
    print("=" * 140)
    
    allocations = portfolio_allocation['allocations']
    
    if not allocations:
        print("❌ No suitable allocations found based on unified risk-reward metric")
        return
    
    print(f"📊 PORTFOLIO SUMMARY:")
    print(f"   Total Allocated: ${portfolio_allocation['total_allocated']:.2f}")
    print(f"   Cash Remaining: ${portfolio_allocation['cash_remaining']:.2f}")
    print(f"   Allocation %: {portfolio_allocation['allocation_percentage']:.1f}%")
    print(f"   Number of Positions: {len(allocations)}")
    
    print(f"\n📈 UNIFIED RISK-REWARD ALLOCATION BREAKDOWN:")
    print("-" * 160)
    print(f"{'Rank':<4} {'Ticker':<8} {'Price':<8} {'Unified':<8} {'Kelly':<8} {'Sortino':<8} {'Calmar':<8} {'Allocation':<12} {'Shares':<8} {'Kelly%':<7} {'Sortino':<7} {'Calmar':<7}")
    print("-" * 160)
    
    for i, alloc in enumerate(allocations, 1):
        # Check if this is a Medallion-style allocation or fallback equal-weight allocation
        if 'unified_score' in alloc:
            # Medallion-style allocation
            print(f"{i:<4} {alloc['ticker']:<8} ${alloc['current_price']:<7.2f} "
                  f"{alloc['unified_score']:<7.1%} {alloc['kelly_score']:<7.1%} "
                  f"{alloc['sortino_score']:<7.1%} {alloc['calmar_score']:<7.1%} "
                  f"${alloc['dollar_allocation']:<11.2f} {alloc['shares_to_buy']:<8} "
                  f"{alloc['kelly_ratio']:<6.1%} {alloc['sortino_ratio']:<6.2f} {alloc['calmar_ratio']:<6.2f}")
        else:
            # Fallback equal-weight allocation
            print(f"{i:<4} {alloc['ticker']:<8} ${alloc['current_price']:<7.2f} "
                  f"{'N/A':<7} {'N/A':<7} "
                  f"{'N/A':<7} {'N/A':<7} "
                  f"${alloc['dollar_allocation']:<11.2f} {alloc['shares_to_buy']:<8} "
                  f"{'N/A':<6} {'N/A':<6} {'N/A':<6}")
    
    print(f"\n🎯 DETAILED UNIFIED ANALYSIS:")
    print("-" * 140)
    
    for i, alloc in enumerate(allocations[:5], 1):  # Show top 5
        # Check if this is a Medallion-style allocation or fallback equal-weight allocation
        if 'unified_score' in alloc:
            print(f"\n🏆 #{i}: {alloc['ticker']} - Medallion-Style Allocation")
            print(f"   Current Price: ${alloc['current_price']:.2f}")
            print(f"   Unified Score: {alloc['unified_score']:.1%}")
            print(f"   Risk-Adjusted Kelly: {alloc['risk_adjusted_kelly']:.1%}")
            print(f"   Scaled Allocation: {alloc['scaled_kelly']:.1%}")
            print(f"   Dollar Allocation: ${alloc['dollar_allocation']:.2f}")
            print(f"   Shares to Buy: {alloc['shares_to_buy']}")
            print(f"   Kelly Ratio: {alloc['kelly_ratio']:.1%} | Sortino Ratio: {alloc['sortino_ratio']:.2f} | Calmar Ratio: {alloc['calmar_ratio']:.2f}")
            print(f"   Sharpe Ratio: {alloc['sharpe_ratio']:.2f} | Volatility: {alloc['volatility']:.1%}")
            print(f"   Win Probability: {alloc['win_probability']:.1%} | Confidence: {alloc['confidence_factor']:.1%}")
            print(f"   Max Drawdown: {alloc['max_drawdown']:.1%} | Sample Size: {alloc['sample_size']} days")
            print(f"   Doubling Score: {alloc['doubling_score']}")
            print(f"   Reasons: {alloc['reasons']}")
        else:
            print(f"\n🏆 #{i}: {alloc['ticker']} - Equal-Weight Allocation")
            print(f"   Current Price: ${alloc['current_price']:.2f}")
            print(f"   Weight: {alloc['scaled_kelly']:.1%}")
            print(f"   Dollar Allocation: ${alloc['dollar_allocation']:.2f}")
            print(f"   Shares to Buy: {alloc['shares_to_buy']}")
            print(f"   Win Probability: {alloc['win_probability']:.1%}")
            print(f"   Volatility: {alloc['volatility']:.1%}")
            print(f"   Doubling Score: {alloc['doubling_score']}")
            print(f"   Reasons: {alloc['reasons']}")
    
    # Check if this is a Medallion-style allocation or fallback equal-weight allocation
    if allocations and 'unified_score' in allocations[0]:
        print(f"\n💡 MEDALLION-STYLE PRINCIPLES:")
        print("-" * 60)
        print("• **Kelly Criterion (40%)**: Optimal position sizing for growth")
        print("• **Sortino Ratio (30%)**: Focus on downside risk only")
        print("• **Calmar Ratio (30%)**: Drawdown control and capital preservation")
        print("• **Unified Score**: Combines all three metrics for optimal allocation")
        print("• **Risk-Adjusted Kelly**: Kelly allocation weighted by unified score")
        print("• **Short-term focus**: Designed for active trading strategies")
    else:
        print(f"\n💡 EQUAL-WEIGHT PRINCIPLES:")
        print("-" * 60)
        print("• **Equal Distribution**: Balanced allocation across top stocks")
        print("• **Score-Based Selection**: Based on doubling scores when Kelly unavailable")
        print("• **Conservative Approach**: Reduced risk through diversification")
        print("• **Simple Strategy**: Easy to understand and implement")
        print("• **Monthly Rebalancing**: Regular updates based on new data")
        print("• **Transition Ready**: Can switch to Kelly when data improves")

def calculate_dynamic_holding_timeframe(unified_score, volatility, max_drawdown, calmar_ratio, sortino_ratio, base_holding_days=1.5):
    """
    Calculate dynamic holding timeframe based on Medallion-style metrics
    
    Parameters:
    unified_score: unified risk-reward score (0-1)
    volatility: stock volatility
    max_drawdown: maximum historical drawdown
    calmar_ratio: Calmar ratio
    sortino_ratio: Sortino ratio
    base_holding_days: base holding period in days (default 30)
    
    Returns:
    dict: holding timeframe recommendations
    """
    # Base holding period adjustments based on unified score
    # Higher unified score = longer holding period (more confidence)
    # Lower unified score = shorter holding period (less confidence)
    unified_adjustment = 1.0 + (unified_score - 0.5) * 0.4  # ±20% adjustment
    
    # Volatility adjustment
    # Higher volatility = shorter holding period (more risk)
    # Lower volatility = longer holding period (less risk)
    volatility_factor = max(0.5, min(1.5, 1.0 - (volatility - 0.05) * 10))  # ±50% adjustment
    
    # Drawdown adjustment
    # Higher drawdown = shorter holding period (more risk)
    # Lower drawdown = longer holding period (less risk)
    drawdown_factor = max(0.3, min(1.2, 1.0 - max_drawdown * 0.8))  # ±70% adjustment
    
    # Calmar ratio adjustment
    # Higher Calmar = longer holding period (better risk-adjusted returns)
    # Lower Calmar = shorter holding period (worse risk-adjusted returns)
    calmar_factor = max(0.6, min(1.4, 1.0 + (calmar_ratio - 1.0) * 0.2))  # ±40% adjustment
    
    # Sortino ratio adjustment
    # Higher Sortino = longer holding period (better downside protection)
    # Lower Sortino = shorter holding period (worse downside protection)
    sortino_factor = max(0.7, min(1.3, 1.0 + (sortino_ratio - 0.5) * 0.4))  # ±30% adjustment
    
    # Calculate final holding period
    adjusted_holding_days = base_holding_days * unified_adjustment * volatility_factor * drawdown_factor * calmar_factor * sortino_factor
    
    # Apply minimum and maximum bounds (Medallion-style ultra-short term)
    min_holding_days = 0.5   # Minimum 12 hours
    max_holding_days = 7     # Maximum 7 days
    final_holding_days = max(min_holding_days, min(max_holding_days, adjusted_holding_days))
    
    # Calculate rebalancing frequency (Medallion-style ultra-short term)
    if final_holding_days <= 1:
        rebalancing_frequency = "Intraday (Multiple times)"
    elif final_holding_days <= 2:
        rebalancing_frequency = "Daily"
    elif final_holding_days <= 3:
        rebalancing_frequency = "Every 1-2 days"
    elif final_holding_days <= 5:
        rebalancing_frequency = "Every 2-3 days"
    else:
        rebalancing_frequency = "Weekly"
    
    # Risk level assessment (Medallion-style ultra-short term)
    if final_holding_days <= 1:
        risk_level = "Ultra-High Risk - Intraday"
        risk_color = "🔴"
    elif final_holding_days <= 2:
        risk_level = "High Risk - Very Short Term"
        risk_color = "🟠"
    elif final_holding_days <= 3:
        risk_level = "Medium-High Risk - Short Term"
        risk_color = "🟡"
    elif final_holding_days <= 5:
        risk_level = "Medium Risk - Medium-Short Term"
        risk_color = "🟢"
    else:
        risk_level = "Medium-Low Risk - Medium Term"
        risk_color = "🔵"
    
    # Exit strategy recommendations (Medallion-style ultra-short term)
    if final_holding_days <= 1:
        exit_strategy = "Ultra-tight stops (1-2%), immediate profit taking (3-5%)"
    elif final_holding_days <= 2:
        exit_strategy = "Tight stops (2-3%), quick profit taking (5-10%)"
    elif final_holding_days <= 3:
        exit_strategy = "Moderate stops (3-5%), profit taking at 8-12%"
    elif final_holding_days <= 5:
        exit_strategy = "Standard stops (4-6%), profit taking at 10-15%"
    else:
        exit_strategy = "Wider stops (5-8%), profit taking at 12-18%"
    
    return {
        'holding_days': int(final_holding_days),
        'rebalancing_frequency': rebalancing_frequency,
        'risk_level': risk_level,
        'risk_color': risk_color,
        'exit_strategy': exit_strategy,
        'factors': {
            'unified_adjustment': unified_adjustment,
            'volatility_factor': volatility_factor,
            'drawdown_factor': drawdown_factor,
            'calmar_factor': calmar_factor,
            'sortino_factor': sortino_factor
        }
    }

def display_dynamic_holding_timeframes(portfolio_allocation):
    """
    Display dynamic holding timeframes for Medallion-style allocations
    
    Parameters:
    portfolio_allocation: portfolio allocation dictionary
    """
    print(f"\n⏰ DYNAMIC HOLDING TIMEFRAMES (MEDALLION-STYLE)")
    print("=" * 120)
    print("Calculating optimal holding periods based on unified risk-reward metrics...")
    print("Shorter holding periods reduce drawdown risk and improve portfolio performance.")
    
    allocations = portfolio_allocation['allocations']
    
    if not allocations:
        print("❌ No allocations to calculate holding timeframes for")
        return
    
    print(f"\n📊 HOLDING TIMEFRAME BREAKDOWN:")
    print("-" * 140)
    print(f"{'Rank':<4} {'Ticker':<8} {'Unified':<8} {'Vol%':<6} {'Drawdown':<10} {'Calmar':<7} {'Sortino':<7} {'Days':<5} {'Risk':<25} {'Rebalance':<15}")
    print("-" * 140)
    
    holding_data = []
    
    for i, alloc in enumerate(allocations, 1):
        # Check if this is a Medallion-style allocation or fallback equal-weight allocation
        if 'unified_score' in alloc:
            # Calculate dynamic holding timeframe for Medallion-style allocation
            holding_info = calculate_dynamic_holding_timeframe(
                unified_score=alloc['unified_score'],
                volatility=alloc['volatility'],
                max_drawdown=alloc['max_drawdown'],
                calmar_ratio=alloc['calmar_ratio'],
                sortino_ratio=alloc['sortino_ratio']
            )
            
            print(f"{i:<4} {alloc['ticker']:<8} {alloc['unified_score']:<7.1%} "
                  f"{alloc['volatility']:<5.1%} {alloc['max_drawdown']:<9.1%} "
                  f"{alloc['calmar_ratio']:<6.2f} {alloc['sortino_ratio']:<6.2f} "
                  f"{holding_info['holding_days']:<5} {holding_info['risk_level'][:24]:<24} "
                  f"{holding_info['rebalancing_frequency']:<15}")
        else:
            # Fallback equal-weight allocation - use simplified holding timeframe
            holding_info = {
                'holding_days': 30,  # Default 30-day holding period
                'risk_level': 'Medium',
                'rebalancing_frequency': 'Monthly',
                'exit_strategy': 'Standard exit at 10-15% gains',
                'factors': {
                    'unified_adjustment': 1.0,
                    'volatility_factor': 1.0,
                    'drawdown_factor': 1.0,
                    'calmar_factor': 1.0,
                    'sortino_factor': 1.0
                }
            }
            
            print(f"{i:<4} {alloc['ticker']:<8} {'N/A':<7} "
                  f"{alloc['volatility']:<5.1%} {'N/A':<9} "
                  f"{'N/A':<6} {'N/A':<6} "
                  f"{holding_info['holding_days']:<5} {holding_info['risk_level'][:24]:<24} "
                  f"{holding_info['rebalancing_frequency']:<15}")
        
        holding_data.append({
            'rank': i,
            'ticker': alloc['ticker'],
            'holding_info': holding_info,
            'allocation': alloc
        })
    
    print(f"\n🎯 DETAILED HOLDING TIMEFRAME ANALYSIS:")
    print("-" * 120)
    
    for data in holding_data[:5]:  # Show top 5
        alloc = data['allocation']
        holding = data['holding_info']
        
        print(f"\n⏰ #{data['rank']}: {alloc['ticker']} - Dynamic Holding Timeframe")
        print(f"   📊 Risk Level: {holding['risk_level']}")
        print(f"   📅 Holding Period: {holding['holding_days']} days")
        print(f"   🔄 Rebalancing: {holding['rebalancing_frequency']}")
        print(f"   🎯 Exit Strategy: {holding['exit_strategy']}")
        
        # Check if this is a Medallion-style allocation or fallback equal-weight allocation
        if 'unified_score' in alloc:
            print(f"   📊 Unified Score: {alloc['unified_score']:.1%}")
            print(f"   📈 Volatility: {alloc['volatility']:.1%}")
            print(f"   📉 Max Drawdown: {alloc['max_drawdown']:.1%}")
            print(f"   📊 Calmar Ratio: {alloc['calmar_ratio']:.2f}")
            print(f"   📊 Sortino Ratio: {alloc['sortino_ratio']:.2f}")
            
            # Show adjustment factors
            factors = holding['factors']
            print(f"   🔧 Adjustment Factors:")
            print(f"      - Unified Score: {factors['unified_adjustment']:.2f}x")
            print(f"      - Volatility: {factors['volatility_factor']:.2f}x")
            print(f"      - Drawdown: {factors['drawdown_factor']:.2f}x")
            print(f"      - Calmar: {factors['calmar_factor']:.2f}x")
            print(f"      - Sortino: {factors['sortino_factor']:.2f}x")
        else:
            print(f"   📊 Unified Score: N/A (Equal-weight allocation)")
            print(f"   📈 Volatility: {alloc['volatility']:.1%}")
            print(f"   📉 Max Drawdown: N/A (Equal-weight allocation)")
            print(f"   📊 Calmar Ratio: N/A (Equal-weight allocation)")
            print(f"   📊 Sortino Ratio: N/A (Equal-weight allocation)")
            print(f"   🔧 Adjustment Factors: Standard (1.0x) - Equal-weight allocation")
    
    # Portfolio-level recommendations
    avg_holding_days = sum(data['holding_info']['holding_days'] for data in holding_data) / len(holding_data)
    high_risk_positions = sum(1 for data in holding_data if data['holding_info']['holding_days'] <= 15)
    low_risk_positions = sum(1 for data in holding_data if data['holding_info']['holding_days'] >= 45)
    
    print(f"\n📊 PORTFOLIO HOLDING TIMEFRAME SUMMARY:")
    print("-" * 60)
    print(f"   Average Holding Period: {avg_holding_days:.1f} days")
    print(f"   High Risk Positions (≤15 days): {high_risk_positions}")
    print(f"   Low Risk Positions (≥45 days): {low_risk_positions}")
    print(f"   Total Positions: {len(holding_data)}")
    
    # Portfolio-level recommendations (Medallion-style ultra-short term)
    if avg_holding_days <= 2:
        portfolio_style = "Ultra-Short Term (Medallion-Style)"
        portfolio_advice = "• Monitor positions intraday\n• Use ultra-tight stop-losses\n• Take profits immediately (3-5%)\n• Very high turnover expected"
    elif avg_holding_days <= 3:
        portfolio_style = "Very Short Term (Active Trading)"
        portfolio_advice = "• Monitor positions daily\n• Use tight stop-losses\n• Take profits quickly (5-10%)\n• High turnover expected"
    elif avg_holding_days <= 5:
        portfolio_style = "Short Term (Swing Trading)"
        portfolio_advice = "• Monitor positions every 1-2 days\n• Use moderate stop-losses\n• Take profits at 8-12%\n• Moderate turnover expected"
    else:
        portfolio_style = "Medium-Short Term (Position Trading)"
        portfolio_advice = "• Monitor positions every 2-3 days\n• Use standard stop-losses\n• Take profits at 10-15%\n• Lower turnover expected"
    
    print(f"   Portfolio Style: {portfolio_style}")
    print(f"   📋 Portfolio Advice:")
    for line in portfolio_advice.split('\n'):
        print(f"      {line}")
    
    print(f"\n💡 MEDALLION-STYLE HOLDING TIMEFRAME PRINCIPLES:")
    print("-" * 60)
    print("• **Shorter holding periods** reduce drawdown risk")
    print("• **Higher volatility** = shorter holding periods")
    print("• **Higher drawdowns** = shorter holding periods")
    print("• **Better risk-adjusted ratios** = longer holding periods")
    print("• **Dynamic adjustment** based on unified risk-reward metrics")
    print("• **Active management** required for optimal performance")

def display_portfolio_allocation(portfolio_allocation, portfolio_value=1000, allocation_type="Kelly", risk_metrics=None):
    """Display portfolio allocation results"""
    print(f"\n🎯 {allocation_type.upper()} PORTFOLIO ALLOCATION (${portfolio_value:,})")
    print("=" * 100)
    
    allocations = portfolio_allocation['allocations']
    
    if not allocations:
        print(f"❌ No suitable allocations found based on {allocation_type}")
        return
    
    print(f"📊 PORTFOLIO SUMMARY:")
    print(f"   Total Allocated: ${portfolio_allocation['total_allocated']:.2f}")
    print(f"   Cash Remaining: ${portfolio_allocation['cash_remaining']:.2f}")
    print(f"   Allocation %: {portfolio_allocation['allocation_percentage']:.1f}%")
    print(f"   Number of Positions: {len(allocations)}")
    
    if risk_metrics:
        print(f"   Expected Return: {risk_metrics['expected_return']:.2%}")
        print(f"   Portfolio Volatility: {risk_metrics['portfolio_volatility']:.2%}")
        print(f"   Portfolio Sharpe Ratio: {risk_metrics['portfolio_sharpe']:.2f}")
    
    print(f"\n📈 ALLOCATION BREAKDOWN:")
    print("-" * 100)
    print(f"{'Rank':<4} {'Ticker':<8} {'Price':<8} {'Allocation':<12} {'Shares':<8} {'Win%':<6} {'Gain%':<7} {'Loss%':<7}")
    print("-" * 100)
    
    for i, alloc in enumerate(allocations, 1):
        print(f"{i:<4} {alloc['ticker']:<8} ${alloc['current_price']:<7.2f} "
              f"${alloc['dollar_allocation']:<11.2f} {alloc['shares_to_buy']:<8} "
              f"{alloc.get('win_probability', 0):<5.1%} {alloc.get('avg_gain', 0):<6.1%} "
              f"{alloc.get('avg_loss', 0):<6.1%}")
    
    print(f"\n🎯 DETAILED ANALYSIS (TOP 3):")
    print("-" * 100)
    
    for i, alloc in enumerate(allocations[:3], 1):
        print(f"\n🥇 #{i}: {alloc['ticker']} - {allocation_type} Allocation")
        print(f"   Current Price: ${alloc['current_price']:.2f}")
        print(f"   Dollar Allocation: ${alloc['dollar_allocation']:.2f}")
        print(f"   Shares to Buy: {alloc['shares_to_buy']}")
        print(f"   Win Probability: {alloc.get('win_probability', 0):.1%}")
        print(f"   Volatility: {alloc.get('volatility', 0):.1%}")
        print(f"   Doubling Score: {alloc.get('doubling_score', 0)}")
        print(f"   Reasons: {alloc.get('reasons', 'N/A')}")

def calculate_options_kelly_allocation(options_data, portfolio_value=1000, scaling_factor=0.25):
    """
    Calculate Kelly Criterion allocation for options
    
    Parameters:
    options_data: list of option opportunities
    portfolio_value: total portfolio value
    scaling_factor: Kelly scaling factor (more conservative for options)
    
    Returns:
    dict: options allocation details
    """
    allocations = []
    total_allocation = 0
    
    for opt in options_data:
        ticker = opt['ticker']
        
        # Estimate option probabilities based on historical data
        stock_data = estimate_stock_probabilities(ticker)
        
        if stock_data:
            # For options, we need to estimate the probability of the stock moving enough
            # to make the option profitable
            
            # Calculate probability of stock moving to different price levels
            current_price = opt['current_price']
            strike = opt['strike']
            ask = opt['ask']
            
            # Estimate probability of stock moving 25%, 50%, 100%
            # This is a simplified approach - in practice you'd use more sophisticated models
            
            # Use historical volatility to estimate move probabilities
            volatility = stock_data['volatility']
            
            # Simplified probability estimates based on normal distribution
            # Probability of stock moving up by X% in the option's time frame
            days_to_expiry = opt.get('days_to_expiry', 30)
            time_factor = np.sqrt(days_to_expiry / 252)  # Annualized to option timeframe
            
            # Calculate probabilities for different price moves
            prob_25 = 1 - norm.cdf(0.25 / (volatility * time_factor))
            prob_50 = 1 - norm.cdf(0.50 / (volatility * time_factor))
            prob_100 = 1 - norm.cdf(1.00 / (volatility * time_factor))
            
            # Use the most realistic probability for Kelly calculation
            # For penny stocks, use 25% move probability as baseline
            p = prob_25
            
            # Calculate potential gains and losses
            if current_price * 1.25 > strike:
                g = (current_price * 1.25 - strike) / ask  # Return if stock moves 25%
            else:
                g = 0.1  # Small gain if option becomes slightly profitable
            
            l = 1.0  # Maximum loss is 100% of option premium
            
            # Calculate Kelly fraction
            kelly_fraction = calculate_kelly_fraction(p, g, l)
            
            # Apply more conservative scaling for options
            scaled_kelly = calculate_scaled_kelly(kelly_fraction, scaling_factor)
            
            # Calculate dollar allocation
            dollar_allocation = scaled_kelly * portfolio_value
            
            # Very conservative limits for options
            max_allocation = portfolio_value * 0.05  # Max 5% per option
            dollar_allocation = min(dollar_allocation, max_allocation)
            
            if dollar_allocation > 5:  # Only include if allocation > $5
                contracts_to_buy = int(dollar_allocation / ask / 100)  # Options are typically 100 shares
                
                if contracts_to_buy > 0:
                    allocations.append({
                        'ticker': ticker,
                        'strike': strike,
                        'ask': ask,
                        'expiry': opt['expiry'],
                        'current_price': current_price,
                        'kelly_fraction': kelly_fraction,
                        'scaled_kelly': scaled_kelly,
                        'dollar_allocation': dollar_allocation,
                        'contracts_to_buy': contracts_to_buy,
                        'win_probability': p,
                        'potential_gain': g,
                        'max_loss': l,
                        'volatility': volatility,
                        'days_to_expiry': days_to_expiry,
                        'return_25': opt.get('return_25', 0),
                        'return_50': opt.get('return_50', 0),
                        'return_100': opt.get('return_100', 0),
                        'score': opt.get('score', 0),
                        'reasons': opt.get('reasons', [])
                    })
                    
                    total_allocation += dollar_allocation
    
    # Sort by Kelly fraction (highest first)
    allocations.sort(key=lambda x: x['kelly_fraction'], reverse=True)
    
    return {
        'allocations': allocations,
        'total_allocated': total_allocation,
        'cash_remaining': portfolio_value - total_allocation,
        'allocation_percentage': (total_allocation / portfolio_value) * 100
    }

def display_options_kelly_allocation(options_allocation, portfolio_value=1000):
    """
    Display Kelly Criterion allocation for options
    
    Parameters:
    options_allocation: options allocation dictionary
    portfolio_value: total portfolio value
    """
    print(f"\n🎯 KELLY CRITERION OPTIONS ALLOCATION (${portfolio_value:,})")
    print("=" * 100)
    
    allocations = options_allocation['allocations']
    
    if not allocations:
        print("❌ No suitable options allocations found based on Kelly Criterion")
        return
    
    print(f"📊 OPTIONS PORTFOLIO SUMMARY:")
    print(f"   Total Allocated: ${options_allocation['total_allocated']:.2f}")
    print(f"   Cash Remaining: ${options_allocation['cash_remaining']:.2f}")
    print(f"   Allocation %: {options_allocation['allocation_percentage']:.1f}%")
    print(f"   Number of Positions: {len(allocations)}")
    
    print(f"\n📈 OPTIONS ALLOCATION BREAKDOWN:")
    print("-" * 120)
    print(f"{'Rank':<4} {'Ticker':<8} {'Strike':<8} {'Cost':<8} {'Expiry':<12} {'Kelly%':<8} {'Scaled%':<8} {'Allocation':<12} {'Contracts':<10} {'Win%':<6} {'Gain%':<7} {'25%':<6} {'50%':<6} {'100%':<6}")
    print("-" * 120)
    
    for i, alloc in enumerate(allocations, 1):
        print(f"{i:<4} {alloc['ticker']:<8} ${alloc['strike']:<7.2f} ${alloc['ask']:<7.2f} "
              f"{alloc['expiry']:<12} {alloc['kelly_fraction']:<7.1%} {alloc['scaled_kelly']:<7.1%} "
              f"${alloc['dollar_allocation']:<11.2f} {alloc['contracts_to_buy']:<10} "
              f"{alloc['win_probability']:<5.1%} {alloc['potential_gain']:<6.1%} "
              f"{alloc['return_25']:<5.0f}% {alloc['return_50']:<5.0f}% {alloc['return_100']:<5.0f}%")
    
    print(f"\n🎯 DETAILED OPTIONS ANALYSIS:")
    print("-" * 100)
    
    for i, alloc in enumerate(allocations[:3], 1):  # Show top 3
        print(f"\n🥇 #{i}: {alloc['ticker']} - Options Kelly Allocation")
        print(f"   Strike: ${alloc['strike']:.2f} | Cost: ${alloc['ask']:.2f} | Expiry: {alloc['expiry']}")
        print(f"   Current Price: ${alloc['current_price']:.2f} | Days to Expiry: {alloc['days_to_expiry']}")
        print(f"   Kelly Fraction: {alloc['kelly_fraction']:.1%} (Full Kelly)")
        print(f"   Scaled Allocation: {alloc['scaled_kelly']:.1%} (Quarter Kelly)")
        print(f"   Dollar Allocation: ${alloc['dollar_allocation']:.2f}")
        print(f"   Contracts to Buy: {alloc['contracts_to_buy']}")
        print(f"   Win Probability: {alloc['win_probability']:.1%}")
        print(f"   Potential Gain: {alloc['potential_gain']:.1%}")
        print(f"   Max Loss: {alloc['max_loss']:.1%}")
        print(f"   Volatility: {alloc['volatility']:.1%}")
        print(f"   If Stock +25%: {alloc['return_25']:.0f}% return")
        print(f"   If Stock +50%: {alloc['return_50']:.0f}% return")
        print(f"   If Stock +100%: {alloc['return_100']:.0f}% return")
        print(f"   Score: {alloc['score']}")
        print(f"   Reasons: {', '.join(alloc['reasons'])}")
    
    print(f"\n⚠️  OPTIONS RISK MANAGEMENT:")
    print("-" * 50)
    print("• Using Quarter-Kelly (25% scaling) for options due to high risk")
    print("• Maximum 5% allocation per option position")
    print("• Minimum $5 allocation per position")
    print("• Options have asymmetric risk (limited upside, unlimited downside)")
    print("• Monitor time decay (theta) closely")
    print("• Consider rolling positions before expiration")
    print("• Use stop-losses or protective puts for risk management")

# =============================================================================
# STOCK ANALYSIS FUNCTIONS
# =============================================================================

def analyze_stock_potential(ticker):
    """Deep analysis of a stock for doubling potential"""
    try:
        yft = yf.Ticker(ticker)
        info = yft.info
        
        hist = yft.history(period='1mo')
        if hist.empty:
            return None
            
        current_price = hist['Close'].iloc[-1]
        
        # Calculate custom PEG ratio if Yahoo Finance doesn't provide it
        yf_peg = info.get('pegRatio', 0)
        earnings_growth = info.get('earningsGrowth', 0)
        pe_ratio = info.get('trailingPE', 0)
        
        # Calculate custom PEG if we have PE and earnings growth but no PEG
        custom_peg = 0
        if yf_peg == 0 and pe_ratio > 0 and earnings_growth > 0:
            custom_peg = pe_ratio / (earnings_growth * 100)  # Convert growth to decimal
        
        # Use Yahoo Finance PEG if available, otherwise use custom calculation
        final_peg = yf_peg if yf_peg > 0 else custom_peg
        
        analysis = {
            'Ticker': ticker,
            'Current_Price': current_price,
            'Market_Cap': info.get('marketCap', 0),
            'Volume_Avg': info.get('averageVolume', 0),
            'Beta': info.get('beta', 0),
            'PE_Ratio': pe_ratio,
            'Forward_PE': info.get('forwardPE', 0),
            'PEG_Ratio': final_peg,
            'Price_to_Sales': info.get('priceToSalesTrailing12Months', 0),
            'Price_to_Book': info.get('priceToBook', 0),
            'Debt_to_Equity': info.get('debtToEquity', 0),
            'ROE': info.get('returnOnEquity', 0),
            'ROA': info.get('returnOnAssets', 0),
            'Profit_Margins': info.get('profitMargins', 0),
            'Operating_Margins': info.get('operatingMargins', 0),
            'Revenue_Growth': info.get('revenueGrowth', 0),
            'Earnings_Growth': earnings_growth,
            'Analyst_Target': info.get('targetMeanPrice', 0),
            'Analyst_Recommendation': info.get('recommendationMean', 0),
            'Insider_Ownership': 0,
            'Institutional_Ownership': 0,
            'Short_Ratio': info.get('shortRatio', 0),
            'Days_To_Cover': info.get('sharesShort', 0) / info.get('averageVolume', 1) if info.get('averageVolume', 0) > 0 else 0,
            'Price_Change_1D': ((current_price - hist['Close'].iloc[-2]) / hist['Close'].iloc[-2] * 100) if len(hist) > 1 else 0,
            'Price_Change_5D': ((current_price - hist['Close'].iloc[-6]) / hist['Close'].iloc[-6] * 100) if len(hist) > 5 else 0,
            'Price_Change_1M': ((current_price - hist['Close'].iloc[-21]) / hist['Close'].iloc[-21] * 100) if len(hist) > 21 else 0,
            'Volatility': hist['Close'].pct_change().std() * 100,
            'Volume_Spike': info.get('volume', 0) / info.get('averageVolume', 1) if info.get('averageVolume', 0) > 0 else 0,
            'Distance_from_52W_High': ((info.get('fiftyTwoWeekHigh', current_price) - current_price) / info.get('fiftyTwoWeekHigh', current_price) * 100) if info.get('fiftyTwoWeekHigh', 0) > 0 else 0,
            'Distance_from_52W_Low': ((current_price - info.get('fiftyTwoWeekLow', current_price)) / info.get('fiftyTwoWeekLow', current_price) * 100) if info.get('fiftyTwoWeekLow', 0) > 0 else 0,
        }
        
        try:
            finviz_stock = finvizfinance(ticker)
            finviz_data = finviz_stock.TickerOverview()
            analysis['Insider_Ownership'] = float(finviz_data.get('Insider Own', '0').replace('%', '')) if finviz_data.get('Insider Own') else 0
            analysis['Institutional_Ownership'] = float(finviz_data.get('Inst Own', '0').replace('%', '')) if finviz_data.get('Inst Own') else 0
            
            # Try multiple field names for short float
            short_float = 0
            for field_name in ['Short Float', 'Short Ratio', 'Shs Float']:
                if finviz_data.get(field_name):
                    try:
                        short_float = float(finviz_data.get(field_name, '0').replace('%', ''))
                        break
                    except:
                        continue
            analysis['Float_Short'] = short_float
            
        except Exception:
            analysis['Insider_Ownership'] = 0
            analysis['Institutional_Ownership'] = 0
            analysis['Float_Short'] = 0
            
        # Fallback to Yahoo Finance for short interest if Finviz failed
        if analysis['Float_Short'] == 0:
            try:
                shares_short = info.get('sharesShort', 0)
                shares_outstanding = info.get('sharesOutstanding', 0)
                if shares_short > 0 and shares_outstanding > 0:
                    analysis['Float_Short'] = (shares_short / shares_outstanding) * 100
            except Exception:
                pass
        
        doubling_score = 0
        reasons = []
        
        # Price-based factors
        if current_price < 5:
            doubling_score += 20
            reasons.append("Low price (<$5)")
        elif current_price < 10:
            doubling_score += 15
            reasons.append("Moderate price (<$10)")
        
        # Market cap
        if analysis['Market_Cap'] < 500_000_000:
            doubling_score += 25
            reasons.append("Micro cap (<$500M)")
        elif analysis['Market_Cap'] < 2_000_000_000:
            doubling_score += 20
            reasons.append("Small cap (<$2B)")
        
        # Volume spike
        if analysis['Volume_Spike'] > 2:
            doubling_score += 15
            reasons.append("High volume spike")
        
        # Short interest
        if analysis['Float_Short'] > 20:
            doubling_score += 20
            reasons.append("High short interest")
        elif analysis['Float_Short'] > 10:
            doubling_score += 10
            reasons.append("Moderate short interest")
        
        # Momentum
        if analysis['Price_Change_1D'] > 5:
            doubling_score += 10
            reasons.append("Strong daily momentum")
        if analysis['Price_Change_5D'] > 15:
            doubling_score += 10
            reasons.append("Strong weekly momentum")
        
        # Volatility
        if analysis['Volatility'] > 5:
            doubling_score += 10
            reasons.append("High volatility")
        
        # Analyst targets
        if analysis['Analyst_Target'] > 0 and analysis['Analyst_Target'] > current_price * 1.5:
            doubling_score += 15
            reasons.append("High analyst targets")
        
        # Growth metrics
        if analysis['Revenue_Growth'] and analysis['Revenue_Growth'] > 0.2:
            doubling_score += 10
            reasons.append("Strong revenue growth")
        if analysis['Earnings_Growth'] and analysis['Earnings_Growth'] > 0.3:
            doubling_score += 10
            reasons.append("Strong earnings growth")
        
        # Valuation metrics - will be updated with dynamic thresholds in main function
        # Placeholder for dynamic valuation scoring
        pass
        
        # Ownership
        if analysis['Insider_Ownership'] > 20:
            doubling_score += 10
            reasons.append("High insider ownership")
        if analysis['Institutional_Ownership'] < 30:
            doubling_score += 5
            reasons.append("Low institutional ownership")
        
        analysis['Doubling_Score'] = doubling_score
        analysis['Reasons'] = ', '.join(reasons)
        
        return analysis
        
    except Exception as e:
        print(f"❌ Error analyzing {ticker}: {e}")
        return None

# =============================================================================
# OPTIONS ANALYSIS FUNCTIONS
# =============================================================================

def analyze_options(ticker_str, min_upside_ratio=0.1, min_open_interest=10, max_expiry_days=90, with_greeks=False):
    """Enhanced options analysis for finding high-reward opportunities"""
    ticker = yf.Ticker(ticker_str)
    try:
        expiries = ticker.options
        if not expiries:
            return None
        
        current_price = ticker.history(period="1d")['Close'].iloc[-1]
        today = pd.Timestamp.today()
        opportunities = []
        
        # Get volatility for Greeks calculation
        if with_greeks:
            hist = ticker.history(period="30d")
            if len(hist) > 1:
                returns = hist['Close'].pct_change().dropna()
                volatility = returns.std() * np.sqrt(252)
            else:
                volatility = 0.5
        
        for expiry in expiries[:3]:  # Limit to first 3 expiries
            expiry_date = pd.Timestamp(expiry)
            days_to_expiry = (expiry_date - today).days
            
            if days_to_expiry > max_expiry_days:
                continue
            
            try:
                opt_chain = ticker.option_chain(expiry)
                calls = opt_chain.calls
                
                for _, row in calls.iterrows():
                    ask = row['ask']
                    strike = row['strike']
                    open_interest = row.get('openInterest', 0)
                    
                    if ask == 0 or pd.isna(ask) or ask < 0.01 or ask > current_price * 0.3:
                        continue
                    
                    # Calculate returns
                    return_25 = (current_price * 1.25 - strike) / ask if current_price * 1.25 > strike else 0
                    return_50 = (current_price * 1.5 - strike) / ask if current_price * 1.5 > strike else 0
                    return_100 = (current_price * 2 - strike) / ask if current_price * 2 > strike else 0
                    
                    # Scoring
                    score = 0
                    reasons = []
                    
                    if ask < current_price * 0.05:
                        score += 25
                        reasons.append("Ultra leverage")
                    elif ask < current_price * 0.1:
                        score += 20
                        reasons.append("High leverage")
                    
                    if return_25 > 2:
                        score += 25
                        reasons.append("Massive upside")
                    elif return_25 > 1:
                        score += 15
                        reasons.append("High upside")
                    
                    if days_to_expiry < 30:
                        score += 10
                        reasons.append("High gamma")
                    
                    if open_interest > min_open_interest:
                        score += 5
                        reasons.append("Some liquidity")
                    
                    if score >= 30 and return_25 >= 0.5:
                        opt_data = {
                            'expiry': expiry,
                            'strike': strike,
                            'ask': ask,
                            'current_price': current_price,
                            'days_to_expiry': days_to_expiry,
                            'return_25': return_25,
                            'return_50': return_50,
                            'return_100': return_100,
                            'score': score,
                            'reasons': reasons,
                            'symbol': ticker_str
                        }
                        
                        if with_greeks:
                            greeks = calculate_greeks(current_price, strike, days_to_expiry, 0.05, volatility, 'call')
                            if greeks:
                                opt_data['greeks'] = greeks
                                opt_data['volatility'] = volatility
                        
                        opportunities.append(opt_data)
                        
            except Exception:
                continue
        
        opportunities.sort(key=lambda x: x['score'], reverse=True)
        return opportunities[:3]
        
    except Exception:
        return None

def calculate_greeks(S, K, T, r, sigma, option_type='call'):
    """Calculate option Greeks using Black-Scholes model"""
    try:
        # Convert time to years
        T = T / 365.0
        
        # Validate inputs to prevent invalid calculations
        if T <= 0 or sigma <= 0 or S <= 0 or K <= 0:
            return None
        
        # Calculate d1 and d2
        d1 = (np.log(S / K) + (r + 0.5 * sigma**2) * T) / (sigma * np.sqrt(T))
        d2 = d1 - sigma * np.sqrt(T)
        
        if option_type == 'call':
            # Delta
            delta = norm.cdf(d1)
            
            # Gamma
            gamma = norm.pdf(d1) / (S * sigma * np.sqrt(T))
            
            # Theta
            theta = (-S * norm.pdf(d1) * sigma / (2 * np.sqrt(T)) - 
                    r * K * np.exp(-r * T) * norm.cdf(d2))
            
            # Vega
            vega = S * np.sqrt(T) * norm.pdf(d1)
            
        else:  # Put
            # Delta
            delta = norm.cdf(d1) - 1
            
            # Gamma
            gamma = norm.pdf(d1) / (S * sigma * np.sqrt(T))
            
            # Theta
            theta = (-S * norm.pdf(d1) * sigma / (2 * np.sqrt(T)) + 
                    r * K * np.exp(-r * T) * norm.cdf(-d2))
            
            # Vega
            vega = S * np.sqrt(T) * norm.pdf(d1)
        
        return {
            'delta': delta,
            'gamma': gamma,
            'theta': theta,
            'vega': vega
        }
    except Exception:
        return None



def find_penny_stock_options():
    """Dynamically find penny stocks with options for maximum leverage"""
    print("\n🔍 Dynamically searching for penny stock options...")
    penny_options = []
    
    # Strategy 1: Use Finviz screener to find penny stocks with options
    try:
        
        # Create screener for penny stocks with high volume
        penny_filters = {
            'Price': 'Under $5',
            'Average Volume': 'Over 1M',  # Higher volume for options liquidity
            'Market Cap.': 'Any',  # Include all market caps
            'Performance': 'Today Up'  # Momentum
        }
        
        screener = Technical()
        screener.set_filter(filters_dict=penny_filters)
        penny_df = screener.screener_view(order='Volume', limit=50, ascend=False)
        
        if penny_df is not None and not penny_df.empty:
            print(f"Found {len(penny_df)} penny stocks from screener")
            
            # Test each stock for options availability
            for ticker in penny_df['Ticker'].head(20):  # Test top 20
                try:
                    yft = yf.Ticker(ticker)
                    
                    # Check if options exist
                    if not yft.options:
                        continue
                    
                    current_price = yft.history(period="1d")['Close'].iloc[-1]
                    
                    # Double-check price is under $5
                    if current_price > 5:
                        continue
                    
                    print(f"  Testing {ticker} (${current_price:.2f}) for options...")
                    
                    opportunities = analyze_options(ticker, min_upside_ratio=0.1, min_open_interest=5, max_expiry_days=60, with_greeks=True)
                    if opportunities:
                        for opt in opportunities:
                            opt['ticker'] = ticker
                            opt['current_price'] = current_price
                            penny_options.append(opt)
                            
                except Exception as e:
                    continue
                    
    except Exception as e:
        print(f"Error with Finviz screener: {e}")
    
    # Strategy 2: Use dynamic scraping to find trending penny stocks
    if len(penny_options) < 5:  # If we need more options
        print("Supplementing with dynamically found trending penny stocks...")
        
        try:
            trending_stocks = get_trending_penny_stocks()
            print(f"Found {len(trending_stocks)} trending stocks to test")
            
            # Add Reddit trending stocks
            reddit_stocks = get_reddit_trending_stocks()
            if reddit_stocks:
                print(f"Found {len(reddit_stocks)} Reddit trending stocks")
                trending_stocks.extend(reddit_stocks)
                trending_stocks = list(set(trending_stocks))  # Remove duplicates
            
            for ticker in trending_stocks[:15]:  # Test top 15
                try:
                    yft = yf.Ticker(ticker)
                    
                    # Check if options exist
                    if not yft.options:
                        continue
                    
                    current_price = yft.history(period="1d")['Close'].iloc[-1]
                    
                    # Only analyze penny stocks
                    if current_price > 5:
                        continue
                    
                    print(f"  Testing {ticker} (${current_price:.2f}) for options...")
                    
                    opportunities = analyze_options(ticker, min_upside_ratio=0.1, min_open_interest=5, max_expiry_days=60, with_greeks=True)
                    if opportunities:
                        for opt in opportunities:
                            opt['ticker'] = ticker
                            opt['current_price'] = current_price
                            penny_options.append(opt)
                            
                except Exception:
                    continue
                    
        except Exception as e:
            print(f"Error with dynamic scraping: {e}")
    
    return penny_options

def find_ultra_cheap_options():
    """Find ultra-cheap options with massive potential returns"""
    print("\n🔍 Searching for ultra-cheap options...")
    ultra_options = []
    
    # Get dynamically discovered penny stocks for ultra-cheap options
    ultra_penny_stocks = get_known_penny_options()
    
    # Also check trending stocks for ultra-cheap options
    try:
        trending = get_trending_penny_stocks()
        ultra_penny_stocks.extend(trending[:10])
    except:
        pass
    
    # Remove duplicates
    ultra_penny_stocks = list(set(ultra_penny_stocks))
    print(f"Checking {len(ultra_penny_stocks)} stocks for ultra-cheap options...")
    
    for ticker in ultra_penny_stocks:
        try:
            yft = yf.Ticker(ticker)
            current_price = yft.history(period="1d")['Close'].iloc[-1]
            
            if current_price > 3 or not yft.options:  # Only very cheap stocks
                continue
            
            print(f"  Testing ultra-cheap {ticker} (${current_price:.2f})...")
            
            # Look for very cheap options
            for expiry in yft.options[:3]:  # Check first 3 expiries
                try:
                    opt_chain = yft.option_chain(expiry)
                    calls = opt_chain.calls
                    
                    for _, row in calls.iterrows():
                        ask = row['ask']
                        strike = row['strike']
                        
                        # Only ultra-cheap options
                        if ask < 0.10 and ask > 0.01:  # Between $0.01 and $0.10
                            # Calculate potential returns
                            if current_price * 1.5 > strike:  # Stock moves 50%
                                return_50 = (current_price * 1.5 - strike) / ask
                                if return_50 > 2:  # At least 200% return
                                    # Calculate Greeks for ultra-cheap options
                                    expiry_date = datetime.strptime(expiry, '%Y-%m-%d')
                                    days_to_expiry = (expiry_date - datetime.now()).days
                                    greeks = calculate_greeks(current_price, strike, days_to_expiry, 0.05, 0.8, 'call')
                                    ultra_options.append({
                                        'ticker': ticker,
                                        'current_price': current_price,
                                        'strike': strike,
                                        'ask': ask,
                                        'expiry': expiry,
                                        'return_50': return_50,
                                        'return_100': (current_price * 2 - strike) / ask if current_price * 2 > strike else 0,
                                        'greeks': greeks
                                    })
                                    
                except Exception:
                    continue
                    
        except Exception:
            continue
    
    return ultra_options

# =============================================================================
# DATA COLLECTION FUNCTIONS
# =============================================================================

def get_trending_penny_stocks():
    """Scrape trending penny stocks from various sources"""
    trending_stocks = set()
    
    # Strategy 1: Scrape from Finviz gainers
    try:
        url = "https://finviz.com/screener.ashx?v=111&s=ta_topgainers&f=sh_price_u5,sh_avgvol_o500"
        headers = {'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36'}
        response = requests.get(url, headers=headers, timeout=10)
        
        if response.status_code == 200:
            soup = BeautifulSoup(response.content, 'html.parser')
            # Find ticker symbols in the table
            ticker_links = soup.find_all('a', href=lambda x: x and 'quote.ashx' in x)
            for link in ticker_links:
                ticker = link.text.strip()
                if ticker and len(ticker) <= 5:  # Valid ticker format
                    trending_stocks.add(ticker)
                    
    except Exception as e:
        print(f"Error scraping Finviz: {e}")
    
    # Strategy 2: Get from Yahoo Finance trending
    try:
        url = "https://finance.yahoo.com/trending-tickers"
        headers = {'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36'}
        response = requests.get(url, headers=headers, timeout=10)
        
        if response.status_code == 200:
            soup = BeautifulSoup(response.content, 'html.parser')
            # Look for ticker symbols
            ticker_elements = soup.find_all(string=lambda text: text and len(text.strip()) <= 5 and text.strip().isupper())
            for ticker in ticker_elements:
                if ticker.strip():
                    trending_stocks.add(ticker.strip())
                    
    except Exception as e:
        print(f"Error scraping Yahoo: {e}")
    
    # Strategy 3: Use known penny stocks with options
    known_penny_options = get_known_penny_options()
    
    for ticker in known_penny_options:
        trending_stocks.add(ticker)
    
    return list(trending_stocks)

def get_reddit_trending_stocks():
    """Get trending stocks from Reddit communities"""
    print("\n🔍 Scraping Reddit for trending stocks...")
    reddit_stocks = set()
    
    try:
        # Initialize Reddit client
        reddit = initialize_reddit_client()
        if not reddit:
            print("  Reddit credentials not found in environment variables. Skipping Reddit scraping.")
            return []
            
        # Get subreddits to monitor
        subreddits = get_reddit_subreddits()
        
        for subreddit_name in subreddits:
            try:
                subreddit = reddit.subreddit(subreddit_name)
                
                # Get hot posts from the last 24 hours
                for submission in subreddit.hot(limit=20):
                    title = submission.title.upper()
                    text = submission.selftext.upper() if submission.selftext else ""
                    
                    # Look for stock tickers (1-5 capital letters)
                    import re
                    tickers = re.findall(r'\b[A-Z]{1,5}\b', title + " " + text)
                    
                    for ticker in tickers:
                        # Filter out common words that aren't tickers
                        if ticker not in ['THE', 'AND', 'FOR', 'YOU', 'ARE', 'WAS', 'HAS', 'HAD', 'NOT', 'BUT', 'ALL', 'CAN', 'HER', 'WERE', 'SHE', 'HIS', 'ONE', 'SAID', 'THEY', 'EACH', 'WHICH', 'SHE', 'DO', 'HOW', 'THEIR', 'IF', 'WILL', 'UP', 'OTHER', 'ABOUT', 'OUT', 'MANY', 'THEN', 'THEM', 'THESE', 'SO', 'SOME', 'HER', 'WOULD', 'MAKE', 'LIKE', 'INTO', 'HIM', 'TIME', 'HAS', 'TWO', 'MORE', 'GO', 'NO', 'WAY', 'COULD', 'MY', 'THAN', 'FIRST', 'BEEN', 'CALL', 'WHO', 'ITS', 'NOW', 'FIND', 'LONG', 'DOWN', 'DAY', 'DID', 'GET', 'COME', 'MADE', 'MAY', 'PART']:
                            reddit_stocks.add(ticker)
                            
            except Exception as e:
                print(f"  Error scraping r/{subreddit_name}: {e}")
                continue
                
    except Exception as e:
        print(f"  Error initializing Reddit client: {e}")
        print("  Note: Check your Reddit API credentials in .env file")
    
    return list(reddit_stocks)

# =============================================================================
# DISPLAY AND REPORTING FUNCTIONS
# =============================================================================

def post_results_to_reddit(results_df, options_found, portfolio_allocation=None, confidence_allocation=None, medallion_allocation=None, thresholds=None):
    """Post screener results to Reddit"""
    try:
        # Check if Reddit credentials are available
        if  not all([REDDIT_CLIENT_ID, REDDIT_CLIENT_SECRET, REDDIT_USERNAME, REDDIT_PASSWORD]):
            print("❌ Reddit credentials not found in environment variables. Cannot post to Reddit.")
            return False
        if not all([REDDIT_CLIENT_ID, REDDIT_CLIENT_SECRET, REDDIT_USERNAME, REDDIT_PASSWORD]):
            print("❌ Reddit credentials not found in environment variables. Cannot post to Reddit.")
            return False
            
        # Initialize Reddit client
        reddit = praw.Reddit(
            client_id=REDDIT_CLIENT_ID,
            client_secret=REDDIT_CLIENT_SECRET,
            user_agent="PennyStockScreener/1.0", 
            username=REDDIT_USERNAME,
            password=REDDIT_PASSWORD,
        )
        
        # Create post content
        title = f"🚀 Penny Stock Screener Results - {datetime.now().strftime('%Y-%m-%d')}"
        
        text = "## 🔥 TOP DOUBLING CANDIDATES\n\n"
        
        # Add top 5 stocks
        for i, (_, row) in enumerate(results_df.head(5).iterrows(), 1):
            text += f"**#{i}: {row['Ticker']}** - Score: {row['Doubling_Score']}\n"
            text += f"- Price: ${row['Current_Price']:.2f} | Market Cap: ${row['Market_Cap']:,.0f}\n"
            text += f"- Daily Change: {row['Price_Change_1D']:+.1f}% | Volume Spike: {row['Volume_Spike']:.1f}x\n"
            
            # Add valuation metrics with dynamic thresholds
            valuation_info = []
            if row['PE_Ratio'] > 0:
                # Dynamic PE status based on thresholds passed from main function
                pe_status = "Low" if row['PE_Ratio'] < thresholds.get('pe_low', 20) else "Medium" if row['PE_Ratio'] < thresholds.get('pe_medium', 30) else "High"
                valuation_info.append(f"PE: {row['PE_Ratio']:.1f} ({pe_status})")
            
            if row['PEG_Ratio'] > 0:
                # Dynamic PEG status based on thresholds passed from main function
                peg_status = "Undervalued" if row['PEG_Ratio'] < thresholds.get('peg_undervalued', 1) else "Fair" if row['PEG_Ratio'] < thresholds.get('peg_fair', 2) else "Overvalued"
                valuation_info.append(f"PEG: {row['PEG_Ratio']:.1f} ({peg_status})")
            
            if row['Price_to_Sales'] > 0:
                # Dynamic P/S status based on thresholds passed from main function
                ps_status = "Low" if row['Price_to_Sales'] < thresholds.get('ps_low', 1) else "Medium" if row['Price_to_Sales'] < thresholds.get('ps_medium', 3) else "High"
                valuation_info.append(f"P/S: {row['Price_to_Sales']:.1f} ({ps_status})")
            
            if valuation_info:
                text += f"- Valuation: {' | '.join(valuation_info)}\n"
            
            if row['Analyst_Target'] > 0:
                target_potential = ((row['Analyst_Target'] - row['Current_Price']) / row['Current_Price']) * 100
                text += f"- Analyst Target: ${row['Analyst_Target']:.2f} ({target_potential:+.1f}% potential)\n"
            text += f"- Reasons: {row['Reasons']}\n\n"
        
        # Add Valuation Summary
        text += "## 📊 VALUATION SUMMARY\n\n"
        
        # Calculate valuation statistics
        pe_ratios = results_df[results_df['PE_Ratio'] > 0]['PE_Ratio']
        peg_ratios = results_df[results_df['PEG_Ratio'] > 0]['PEG_Ratio']
        ps_ratios = results_df[results_df['Price_to_Sales'] > 0]['Price_to_Sales']
        
        if not pe_ratios.empty:
            pe_low_threshold = thresholds.get('pe_low', 20) if thresholds else 20
            pe_high_threshold = thresholds.get('pe_high', 30) if thresholds else 30
            text += f"**PE Ratio Stats:**\n"
            text += f"- Average: {pe_ratios.mean():.1f} | Median: {pe_ratios.median():.1f}\n"
            text += f"- Range: {pe_ratios.min():.1f} - {pe_ratios.max():.1f}\n"
            text += f"- Low PE (<{pe_low_threshold:.1f}): {len(pe_ratios[pe_ratios < pe_low_threshold])} stocks\n"
            text += f"- High PE (>{pe_high_threshold:.1f}): {len(pe_ratios[pe_ratios > pe_high_threshold])} stocks\n\n"
        
        if not peg_ratios.empty:
            peg_undervalued_threshold = thresholds.get('peg_undervalued', 1) if thresholds else 1
            peg_overvalued_threshold = thresholds.get('peg_overvalued', 2) if thresholds else 2
            text += f"**PEG Ratio Stats:**\n"
            text += f"- Average: {peg_ratios.mean():.1f} | Median: {peg_ratios.median():.1f}\n"
            text += f"- Range: {peg_ratios.min():.1f} - {peg_ratios.max():.1f}\n"
            text += f"- Undervalued (<{peg_undervalued_threshold:.1f}): {len(peg_ratios[peg_ratios < peg_undervalued_threshold])} stocks\n"
            text += f"- Overvalued (>{peg_overvalued_threshold:.1f}): {len(peg_ratios[peg_ratios > peg_overvalued_threshold])} stocks\n\n"
        
        if not ps_ratios.empty:
            ps_low_threshold = thresholds.get('ps_low', 1) if thresholds else 1
            ps_high_threshold = thresholds.get('ps_high', 3) if thresholds else 3
            text += f"**P/S Ratio Stats:**\n"
            text += f"- Average: {ps_ratios.mean():.1f} | Median: {ps_ratios.median():.1f}\n"
            text += f"- Range: {ps_ratios.min():.1f} - {ps_ratios.max():.1f}\n"
            text += f"- Low P/S (<{ps_low_threshold:.1f}): {len(ps_ratios[ps_ratios < ps_low_threshold])} stocks\n"
            text += f"- High P/S (>{ps_high_threshold:.1f}): {len(ps_ratios[ps_ratios > ps_high_threshold])} stocks\n\n"
        
        # Add Kelly Criterion Portfolio Allocation
        if portfolio_allocation and portfolio_allocation['allocations']:
            text += "## 🎯 KELLY CRITERION PORTFOLIO ALLOCATION ($1,000)\n\n"
            text += f"**Portfolio Summary:**\n"
            text += f"- Total Allocated: ${portfolio_allocation['total_allocated']:.0f}\n"
            text += f"- Cash Remaining: ${portfolio_allocation['cash_remaining']:.0f}\n"
            text += f"- Allocation %: {portfolio_allocation['allocation_percentage']:.1f}%\n"
            text += f"- Number of Positions: {len(portfolio_allocation['allocations'])}\n\n"
            
            text += "**Top Kelly Allocations:**\n"
            for i, alloc in enumerate(portfolio_allocation['allocations'][:5], 1):
                # Get valuation data from results_df
                stock_data = results_df[results_df['Ticker'] == alloc['ticker']].iloc[0] if len(results_df[results_df['Ticker'] == alloc['ticker']]) > 0 else None
                
                text += f"{i}. **{alloc['ticker']}** - ${alloc['dollar_allocation']:.0f} ({alloc['scaled_kelly']:.1%})\n"
                text += f"   - Shares: {alloc['shares_to_buy']} | Win Rate: {alloc['win_probability']:.1%}\n"
                text += f"   - Kelly Score: {alloc['kelly_fraction']:.1%} | Doubling Score: {alloc['doubling_score']}\n"
                
                # Add valuation metrics if available
                if stock_data is not None:
                    valuation_info = []
                    if stock_data['PE_Ratio'] > 0:
                        pe_status = "Low" if stock_data['PE_Ratio'] < thresholds.get('pe_low', 20) else "Medium" if stock_data['PE_Ratio'] < thresholds.get('pe_medium', 30) else "High"
                        valuation_info.append(f"PE: {stock_data['PE_Ratio']:.1f} ({pe_status})")
                    
                    if stock_data['PEG_Ratio'] > 0:
                        peg_status = "Undervalued" if stock_data['PEG_Ratio'] < thresholds.get('peg_undervalued', 1) else "Fair" if stock_data['PEG_Ratio'] < thresholds.get('peg_fair', 2) else "Overvalued"
                        valuation_info.append(f"PEG: {stock_data['PEG_Ratio']:.1f} ({peg_status})")
                    
                    if stock_data['Price_to_Sales'] > 0:
                        ps_status = "Low" if stock_data['Price_to_Sales'] < thresholds.get('ps_low', 1) else "Medium" if stock_data['Price_to_Sales'] < thresholds.get('ps_medium', 3) else "High"
                        valuation_info.append(f"P/S: {stock_data['Price_to_Sales']:.1f} ({ps_status})")
                    
                    if valuation_info:
                        text += f"   - Valuation: {' | '.join(valuation_info)}\n"
                
                text += "\n"
        
        # Add Confidence-Weighted Kelly Comparison
        if confidence_allocation and confidence_allocation['allocations']:
            text += "## 📊 CONFIDENCE-WEIGHTED KELLY ANALYSIS\n\n"
            text += f"**Confidence-Weighted Summary:**\n"
            text += f"- Total Allocated: ${confidence_allocation['total_allocated']:.0f}\n"
            text += f"- Cash Remaining: ${confidence_allocation['cash_remaining']:.0f}\n"
            text += f"- Allocation %: {confidence_allocation['allocation_percentage']:.1f}%\n"
            text += f"- Number of Positions: {len(confidence_allocation['allocations'])}\n\n"
            
            if portfolio_allocation:
                standard_allocated = portfolio_allocation['total_allocated']
                confidence_allocated = confidence_allocation['total_allocated']
                difference = confidence_allocated - standard_allocated
                percent_change = ((confidence_allocated/standard_allocated - 1)*100) if standard_allocated > 0 else 0
                
                text += f"**Comparison:**\n"
                text += f"- Standard Kelly: ${standard_allocated:.0f}\n"
                text += f"- Confidence-Weighted: ${confidence_allocated:.0f}\n"
                text += f"- Difference: ${difference:+.0f} ({percent_change:+.1f}%)\n\n"
            
            text += "**Top Confidence-Weighted Allocations:**\n"
            for i, alloc in enumerate(confidence_allocation['allocations'][:3], 1):
                # Get valuation data from results_df
                stock_data = results_df[results_df['Ticker'] == alloc['ticker']].iloc[0] if len(results_df[results_df['Ticker'] == alloc['ticker']]) > 0 else None
                
                text += f"{i}. **{alloc['ticker']}** - ${alloc['dollar_allocation']:.0f} ({alloc['scaled_kelly']:.1%})\n"
                
                # Check if this is a confidence-weighted allocation or fallback equal-weight allocation
                if 'confidence_factor' in alloc:
                    text += f"   - Confidence: {alloc['confidence_factor']:.1%} | Sample Size: {alloc['sample_size']} days\n"
                else:
                    text += f"   - Confidence: N/A (Equal-weight allocation) | Sample Size: N/A\n"
                
                text += f"   - Win Rate: {alloc['win_probability']:.1%} | Volatility: {alloc['volatility']:.1%}\n"
                
                # Add valuation metrics if available
                if stock_data is not None:
                    valuation_info = []
                    if stock_data['PE_Ratio'] > 0:
                        pe_status = "Low" if stock_data['PE_Ratio'] < thresholds.get('pe_low', 20) else "Medium" if stock_data['PE_Ratio'] < thresholds.get('pe_medium', 30) else "High"
                        valuation_info.append(f"PE: {stock_data['PE_Ratio']:.1f} ({pe_status})")
                    
                    if stock_data['PEG_Ratio'] > 0:
                        peg_status = "Undervalued" if stock_data['PEG_Ratio'] < thresholds.get('peg_undervalued', 1) else "Fair" if stock_data['PEG_Ratio'] < thresholds.get('peg_fair', 2) else "Overvalued"
                        valuation_info.append(f"PEG: {stock_data['PEG_Ratio']:.1f} ({peg_status})")
                    
                    if stock_data['Price_to_Sales'] > 0:
                        ps_status = "Low" if stock_data['Price_to_Sales'] < thresholds.get('ps_low', 1) else "Medium" if stock_data['Price_to_Sales'] < thresholds.get('ps_medium', 3) else "High"
                        valuation_info.append(f"P/S: {stock_data['Price_to_Sales']:.1f} ({ps_status})")
                    
                    if valuation_info:
                        text += f"   - Valuation: {' | '.join(valuation_info)}\n"
                
                text += "\n"
    
        # Add Medallion-Style Analysis
        if medallion_allocation and medallion_allocation['allocations']:
            text += "## 🏆 MEDALLION-STYLE UNIFIED RISK-REWARD ANALYSIS\n\n"
            text += f"**Medallion-Style Summary:**\n"
            text += f"- Total Allocated: ${medallion_allocation['total_allocated']:.0f}\n"
            text += f"- Cash Remaining: ${medallion_allocation['cash_remaining']:.0f}\n"
            text += f"- Allocation %: {medallion_allocation['allocation_percentage']:.1f}%\n"
            text += f"- Number of Positions: {len(medallion_allocation['allocations'])}\n\n"
        
            text += "**Top Medallion-Style Allocations:**\n"
            for i, alloc in enumerate(medallion_allocation['allocations'][:3], 1):
                # Check if this is a Medallion-style allocation or fallback equal-weight allocation
                if 'unified_score' in alloc:
                    # Calculate holding timeframe for this allocation
                    holding_info = calculate_dynamic_holding_timeframe(
                        unified_score=alloc['unified_score'],
                        volatility=alloc['volatility'],
                        max_drawdown=alloc['max_drawdown'],
                        calmar_ratio=alloc['calmar_ratio'],
                        sortino_ratio=alloc['sortino_ratio']
                    )
                    
                    # Get valuation data from results_df
                    stock_data = results_df[results_df['Ticker'] == alloc['ticker']].iloc[0] if len(results_df[results_df['Ticker'] == alloc['ticker']]) > 0 else None
                    
                    text += f"{i}. **{alloc['ticker']}** - ${alloc['dollar_allocation']:.0f} ({alloc['scaled_kelly']:.1%})\n"
                    text += f"   - Unified Score: {alloc['unified_score']:.1%} | Kelly: {alloc['kelly_score']:.1%} | Sortino: {alloc['sortino_score']:.1%} | Calmar: {alloc['calmar_score']:.1%}\n"
                    text += f"   - Kelly Ratio: {alloc['kelly_ratio']:.1%} | Sortino Ratio: {alloc['sortino_ratio']:.2f} | Calmar Ratio: {alloc['calmar_ratio']:.2f}\n"
                    text += f"   - Holding Period: {holding_info['holding_days']} days | Risk: {holding_info['risk_level']} | Rebalance: {holding_info['rebalancing_frequency']}\n"
                    
                    # Add valuation metrics if available
                    if stock_data is not None:
                        valuation_info = []
                        if stock_data['PE_Ratio'] > 0:
                            pe_status = "Low" if stock_data['PE_Ratio'] < thresholds.get('pe_low', 20) else "Medium" if stock_data['PE_Ratio'] < thresholds.get('pe_medium', 30) else "High"
                            valuation_info.append(f"PE: {stock_data['PE_Ratio']:.1f} ({pe_status})")
                        
                        if stock_data['PEG_Ratio'] > 0:
                            peg_status = "Undervalued" if stock_data['PEG_Ratio'] < thresholds.get('peg_undervalued', 1) else "Fair" if stock_data['PEG_Ratio'] < thresholds.get('peg_fair', 2) else "Overvalued"
                            valuation_info.append(f"PEG: {stock_data['PEG_Ratio']:.1f} ({peg_status})")
                        
                        if stock_data['Price_to_Sales'] > 0:
                            ps_status = "Low" if stock_data['Price_to_Sales'] < thresholds.get('ps_low', 1) else "Medium" if stock_data['Price_to_Sales'] < thresholds.get('ps_medium', 3) else "High"
                            valuation_info.append(f"P/S: {stock_data['Price_to_Sales']:.1f} ({ps_status})")
                        
                        if valuation_info:
                            text += f"   - Valuation: {' | '.join(valuation_info)}\n"
                    
                    text += "\n"
                else:
                    # Fallback equal-weight allocation
                    # Get valuation data from results_df
                    stock_data = results_df[results_df['Ticker'] == alloc['ticker']].iloc[0] if len(results_df[results_df['Ticker'] == alloc['ticker']]) > 0 else None
                    
                    text += f"{i}. **{alloc['ticker']}** - ${alloc['dollar_allocation']:.0f} ({alloc['scaled_kelly']:.1%})\n"
                    text += f"   - Equal Weight Allocation | Doubling Score: {alloc['doubling_score']}\n"
                    text += f"   - Win Probability: {alloc['win_probability']:.1%} | Volatility: {alloc['volatility']:.1%}\n"
                    text += f"   - Conservative Approach | Rebalance Monthly\n"
                    
                    # Add valuation metrics if available
                    if stock_data is not None:
                        valuation_info = []
                        if stock_data['PE_Ratio'] > 0:
                            pe_status = "Low" if stock_data['PE_Ratio'] < thresholds.get('pe_low', 20) else "Medium" if stock_data['PE_Ratio'] < thresholds.get('pe_medium', 30) else "High"
                            valuation_info.append(f"PE: {stock_data['PE_Ratio']:.1f} ({pe_status})")
                        
                        if stock_data['PEG_Ratio'] > 0:
                            peg_status = "Undervalued" if stock_data['PEG_Ratio'] < thresholds.get('peg_undervalued', 1) else "Fair" if stock_data['PEG_Ratio'] < thresholds.get('peg_fair', 2) else "Overvalued"
                            valuation_info.append(f"PEG: {stock_data['PEG_Ratio']:.1f} ({peg_status})")
                        
                        if stock_data['Price_to_Sales'] > 0:
                            ps_status = "Low" if stock_data['Price_to_Sales'] < thresholds.get('ps_low', 1) else "Medium" if stock_data['Price_to_Sales'] < thresholds.get('ps_medium', 3) else "High"
                            valuation_info.append(f"P/S: {stock_data['Price_to_Sales']:.1f} ({ps_status})")
                        
                        if valuation_info:
                            text += f"   - Valuation: {' | '.join(valuation_info)}\n"
                    
                    text += "\n"
    
        # Add Kelly Criterion Insights
        text += "## 💡 KELLY CRITERION INSIGHTS\n\n"
        text += "• **Kelly Criterion** maximizes long-term geometric growth\n"
        text += "• **Confidence-weighted Kelly** adjusts for uncertainty in estimates\n"
        text += "• **Higher confidence** = larger position sizes\n"
        text += "• **Lower confidence** = smaller position sizes (more conservative)\n"
        text += "• **Half-Kelly (50%)** provides ~90% of growth with half the volatility\n"
        text += "• **Rebalance monthly** based on updated Kelly calculations\n\n"
    
        # Add Medallion-Style Insights
        if medallion_allocation:
            text += "## 🏆 MEDALLION-STYLE PRINCIPLES\n\n"
            text += "• **Unified Risk-Reward Metric**: Combines Kelly, Sortino, and Calmar ratios\n"
            text += "• **Kelly Criterion (40%)**: Optimal position sizing for growth\n"
            text += "• **Sortino Ratio (30%)**: Focus on downside risk only\n"
            text += "• **Calmar Ratio (30%)**: Drawdown control and capital preservation\n"
            text += "• **Risk-Adjusted Kelly**: Kelly allocation weighted by unified score\n"
            text += "• **Short-term focus**: Designed for active trading strategies\n\n"
        
        # Add Valuation Analysis Section
        text += "## 📊 STOCK VALUATION FRAMEWORK\n\n"
        
        text += "### 💰 Valuing Profitable Businesses\n\n"
        text += "• **PE Ratio** = Share Price ÷ Earnings Per Share\n"
        pe_low_display = thresholds.get('pe_low', 20) if thresholds else 20
        pe_high_display = thresholds.get('pe_high', 30) if thresholds else 30
        text += f"• **Low PE**: Below {pe_low_display:.1f} (potentially undervalued)\n"
        text += f"• **Medium PE**: {pe_low_display:.1f}-{pe_high_display:.1f} (fairly valued)\n"
        text += f"• **High PE**: Above {pe_high_display:.1f} (potentially overvalued)\n"
        text += "• **Warning**: PE alone doesn't work for high-growth stocks\n\n"
        
        text += "### 🚀 Valuing High-Growth Businesses\n\n"
        text += "• **PEG Ratio** = PE Ratio ÷ Expected Annual Earnings Growth Rate\n"
        peg_undervalued_display = thresholds.get('peg_undervalued', 1) if thresholds else 1
        peg_overvalued_display = thresholds.get('peg_overvalued', 2) if thresholds else 2
        text += f"• **PEG < {peg_undervalued_display:.1f}**: Undervalued (growth justifies high PE)\n"
        text += f"• **PEG {peg_undervalued_display:.1f}-{peg_overvalued_display:.1f}**: Fairly valued\n"
        text += f"• **PEG > {peg_overvalued_display:.1f}**: Overvalued (growth doesn't justify PE)\n"
        text += "• **Key Insight**: High PE is acceptable if PEG is low\n\n"
        
        text += "### 📈 Price-to-Growth Ratio Usage\n\n"
        text += "• **Growth-Adjusted Valuation**: PEG accounts for future earnings potential\n"
        text += "• **Growth Rate Quality**: Use analyst estimates or historical growth\n"
        text += "• **Industry Comparison**: Compare PEG within the same sector\n"
        text += "• **Risk Assessment**: Higher growth rates may be less sustainable\n"
        text += "• **Combined Analysis**: Use with PE ratio for complete picture\n\n"
        
        text += "### 💸 Valuing Loss-Making Businesses\n\n"
        text += "• **Price-to-Sales Ratio** = Market Cap ÷ Annual Revenue\n"
        text += "• **Alternative Metric**: When PE and PEG don't apply\n"
        ps_low_display = thresholds.get('ps_low', 1) if thresholds else 1
        ps_high_display = thresholds.get('ps_high', 3) if thresholds else 3
        text += f"• **Low P/S**: Below {ps_low_display:.1f} (potentially undervalued)\n"
        text += f"• **Medium P/S**: {ps_low_display:.1f}-{ps_high_display:.1f} (fairly valued)\n"
        text += f"• **High P/S**: Above {ps_high_display:.1f} (potentially overvalued)\n\n"
        
        text += "### 📊 Price-to-Sales Ratio Usage\n\n"
        text += "• **Revenue Focus**: Values company based on sales, not earnings\n"
        text += "• **Loss-Making Companies**: Essential for unprofitable businesses\n"
        text += "• **Growth Stage**: Useful for companies reinvesting profits\n"
        text += "• **Industry Variations**: Tech companies often have higher P/S ratios\n"
        text += "• **Profit Margin Context**: Consider if high sales lead to future profits\n\n"
        
        # Add detailed options analysis
        if options_found:
            text += "## 🎯 OPTIONS OPPORTUNITIES\n\n"
            text += f"Found {len(options_found)} high-reward option opportunities:\n\n"
            
            # Sort options by score
            options_found.sort(key=lambda x: x.get('score', 0), reverse=True)
            
            # Add top 3 options with key details
            for i, opt in enumerate(options_found[:3], 1):
                text += f"**#{i}: {opt['ticker']}** - Score: {opt.get('score', 'N/A')}\n"
                text += f"- Strike: ${opt['strike']:.2f} | Cost: ${opt['ask']:.2f} | Expiry: {opt['expiry']}\n"
                text += f"- Current Price: ${opt['current_price']:.2f}\n"
                
                # Add key returns
                if opt.get('return_25', 0) > 0:
                    text += f"- 25% move: {opt['return_25']:.0f}% return\n"
                if opt.get('return_50', 0) > 0:
                    text += f"- 50% move: {opt['return_50']:.0f}% return\n"
                if opt.get('return_100', 0) > 0:
                    text += f"- 100% move: {opt['return_100']:.0f}% return\n"
                
                # Add key reasons
                if opt.get('reasons'):
                    text += f"- Reasons: {', '.join(opt['reasons'][:3])}\n"  # Limit to top 3 reasons
                
                text += "\n"
        
        text += "---\n"
        text += "*Generated by Penny Stock Screener with Kelly Criterion Analysis*\n"
        text += "*Not financial advice - Always do your own research!*\n"
        text += "*Kelly Criterion optimizes position sizing for long-term growth*"
        
        # Reddit posting removed
        print("Reddit posting removed. Returning content only.")
        return {"title": title, "text": text}
        
    except Exception as e:
        print(f"❌ Error posting to Reddit: {e}")
        print("  Note: Check your Reddit API credentials in .env file")
        return False

def main():
    print("🚀 POTENTIAL DOUBLING STOCKS SCREENER")
    print("=" * 60)
    print("Finding stocks with potential for significant gains...")
    
    screener = Technical()
    screener.set_filter(filters_dict=get_screener_filters())
    df = screener.screener_view(order='Change', limit=50, ascend=False)
    
    if df is None or df.empty:
        print("❌ No stocks found matching criteria")
        return
    
    print(f"\n✅ Found {len(df)} initial candidates")
    
    print("\n🔍 Analyzing top candidates...")
    detailed_analysis = []
    
    tickers_to_analyze = df['Ticker'].head(10)
    total_tickers = len(tickers_to_analyze)
    
    for i, ticker in enumerate(tickers_to_analyze, 1):
        print(f"  [{i}/{total_tickers}] Analyzing {ticker}...")
        analysis = analyze_stock_potential(ticker)
        if analysis:
            detailed_analysis.append(analysis)
        time.sleep(0.3)
    
    if not detailed_analysis:
        print("❌ No stocks could be analyzed")
        return
    
    results_df = pd.DataFrame(detailed_analysis)
    
    # Calculate dynamic valuation thresholds
    thresholds = calculate_dynamic_valuation_thresholds(results_df)
    
    # Apply dynamic valuation scoring
    for idx, row in results_df.iterrows():
        doubling_score = row['Doubling_Score']
        reasons = row['Reasons'].split(', ')
        
        # Dynamic valuation scoring
        if row['PE_Ratio'] > 0 and row['PE_Ratio'] < thresholds['pe_low']:
            doubling_score += 5
            reasons.append("Low PE ratio")
        elif row['PE_Ratio'] > 0 and row['PE_Ratio'] > thresholds['pe_high']:
            doubling_score -= 5
            reasons.append("High PE ratio")
        
        if row['PEG_Ratio'] > 0 and row['PEG_Ratio'] < thresholds['peg_undervalued']:
            doubling_score += 10
            reasons.append("Undervalued PEG")
        elif row['PEG_Ratio'] > 0 and row['PEG_Ratio'] > thresholds['peg_overvalued']:
            doubling_score -= 5
            reasons.append("Overvalued PEG")
        
        if row['Price_to_Sales'] > 0 and row['Price_to_Sales'] < thresholds['ps_low']:
            doubling_score += 5
            reasons.append("Low P/S ratio")
        elif row['Price_to_Sales'] > 0 and row['Price_to_Sales'] > thresholds['ps_high']:
            doubling_score -= 5
            reasons.append("High P/S ratio")
        
        # Update the row
        results_df.at[idx, 'Doubling_Score'] = doubling_score
        results_df.at[idx, 'Reasons'] = ', '.join(reasons)
    
    results_df = results_df.sort_values('Doubling_Score', ascending=False)
    
    print(f"\n🔥 TOP DOUBLING CANDIDATES (Found {len(results_df)} stocks)")
    print(f"📊 Dynamic Valuation Thresholds:")
    print(f"   PE: Low < {thresholds['pe_low']:.1f}, High > {thresholds['pe_high']:.1f}")
    print(f"   PEG: Undervalued < {thresholds['peg_undervalued']:.1f}, Overvalued > {thresholds['peg_overvalued']:.1f}")
    print(f"   P/S: Low < {thresholds['ps_low']:.1f}, High > {thresholds['ps_high']:.1f}")
    print("=" * 100)
    
    display_columns = ['Ticker', 'Current_Price', 'Market_Cap', 'Doubling_Score', 
                      'Price_Change_1D', 'Volume_Spike', 'Float_Short', 'Volatility']
    
    available_columns = [col for col in display_columns if col in results_df.columns]
    display_df = results_df[available_columns].head(10)
    
    pd.set_option('display.max_columns', None)
    pd.set_option('display.width', None)
    print(display_df.to_string(index=False, float_format='%.2f'))
    
    print(f"\n📋 DETAILED BREAKDOWN (TOP 3)")
    print("=" * 100)
    
    for i, (_, row) in enumerate(results_df.head(3).iterrows(), 1):
        print(f"\n🥇 #{i}: {row['Ticker']} | Score: {row['Doubling_Score']}")
        print(f"   Price: ${row['Current_Price']:.2f}, Market Cap: ${row['Market_Cap']:,.0f}")
        print(f"   Daily Change: {row['Price_Change_1D']:+.1f}%, Vol Spike: {row['Volume_Spike']:.1f}x")
        print(f"   Short Float: {row['Float_Short']:.1f}%, Volatility: {row['Volatility']:.1f}%")
        print(f"   Reasons: {row['Reasons']}")
        if row['Analyst_Target'] > 0:
            target_potential = ((row['Analyst_Target'] - row['Current_Price']) / row['Current_Price']) * 100
            print(f"   Analyst Target: ${row['Analyst_Target']:.2f} ({target_potential:+.1f}% potential)")
    
    # PORTFOLIO ALLOCATION ANALYSIS
    print(f"\n🎯 PORTFOLIO ALLOCATION ANALYSIS")
    print("=" * 100)
    
    stocks_data = results_df.head(10).to_dict('records')
    
    # Standard Kelly allocation
    portfolio_allocation = calculate_portfolio_allocation(stocks_data, portfolio_value=1000, scaling_factor=0.5)
    if not portfolio_allocation['allocations']:
        portfolio_allocation = create_equal_weight_allocation(stocks_data, 1000, max_positions=5)
    
    risk_metrics = calculate_risk_metrics(portfolio_allocation)
    display_portfolio_allocation(portfolio_allocation, 1000, "Kelly", risk_metrics)
    
    # Confidence-weighted Kelly allocation
    confidence_portfolio_allocation = calculate_confidence_weighted_portfolio_allocation(stocks_data, portfolio_value=1000, scaling_factor=0.5, risk_aversion=1.0)
    if not confidence_portfolio_allocation['allocations']:
        confidence_portfolio_allocation = create_equal_weight_allocation(stocks_data, 1000, max_positions=5)
    
    display_portfolio_allocation(confidence_portfolio_allocation, 1000, "Confidence-Weighted Kelly")
    
    # Compare standard vs confidence-weighted approaches
    print(f"\n📊 STANDARD vs CONFIDENCE-WEIGHTED KELLY COMPARISON")
    print("-" * 80)
    
    standard_allocated = portfolio_allocation['total_allocated']
    confidence_allocated = confidence_portfolio_allocation['total_allocated']
    standard_positions = len(portfolio_allocation['allocations'])
    confidence_positions = len(confidence_portfolio_allocation['allocations'])
    
    print(f"  Standard Kelly: ${standard_allocated:.0f} allocated, {standard_positions} positions")
    print(f"  Confidence-Weighted: ${confidence_allocated:.0f} allocated, {confidence_positions} positions")
    if standard_allocated > 0:
        percent_change = ((confidence_allocated/standard_allocated - 1)*100)
        print(f"  Difference: ${confidence_allocated - standard_allocated:+.0f} ({percent_change:+.1f}%)")
    else:
        print(f"  Difference: ${confidence_allocated - standard_allocated:+.0f} (N/A - no standard allocation)")
    
    if confidence_positions > 0:
        # Check if this is a confidence-weighted allocation or fallback equal-weight allocation
        if 'confidence_factor' in confidence_portfolio_allocation['allocations'][0]:
            avg_confidence = sum(alloc['confidence_factor'] for alloc in confidence_portfolio_allocation['allocations']) / confidence_positions
            print(f"  Average Confidence Factor: {avg_confidence:.1%}")
        else:
            print(f"  Average Confidence Factor: N/A (using equal-weight allocation)")
    
    # Additional Kelly analysis for different scaling factors
    print(f"\n📊 KELLY SCALING COMPARISON")
    print("-" * 50)
    
    scaling_factors = [0.25, 0.5, 0.75, 1.0]  # Quarter, Half, Three-quarter, Full Kelly
    
    for scaling in scaling_factors:
        test_allocation = calculate_portfolio_allocation(stocks_data, 1000, scaling)
        if test_allocation['allocations']:
            print(f"  {scaling*100:.0f}% Kelly: ${test_allocation['total_allocated']:.0f} allocated, "
                  f"{len(test_allocation['allocations'])} positions, "
                  f"{test_allocation['cash_remaining']:.0f} cash remaining")
        else:
            print(f"  {scaling*100:.0f}% Kelly: No suitable allocations")
    
    print(f"\n💡 KELLY CRITERION INSIGHTS:")
    print("-" * 50)
    print("• Kelly Criterion maximizes long-term geometric growth")
    print("• Confidence-weighted Kelly adjusts for uncertainty in estimates")
    print("• Higher confidence = larger position sizes")
    print("• Lower confidence = smaller position sizes")
    print("• Full Kelly can be volatile - consider scaling down")
    print("• Half-Kelly (50%) provides ~90% of growth with half the volatility")
    print("• Quarter-Kelly (25%) is very conservative but still growth-optimal")
    print("• Recalculate Kelly fractions monthly as probabilities change")
    print("• Use remaining cash for new opportunities or risk management")
    
    # Medallion-style allocation
    medallion_allocation = calculate_medallion_style_portfolio_allocation(stocks_data, portfolio_value=1000, scaling_factor=0.5, risk_aversion=1.0)
    if not medallion_allocation['allocations']:
        medallion_allocation = create_equal_weight_allocation(stocks_data, 1000, max_positions=5)
    
    display_portfolio_allocation(medallion_allocation, 1000, "Medallion-Style")
    
    # Compare all three approaches
    print(f"\n📊 COMPREHENSIVE ALLOCATION COMPARISON")
    print("-" * 80)
    
    standard_allocated = portfolio_allocation['total_allocated']
    confidence_allocated = confidence_portfolio_allocation['total_allocated']
    medallion_allocated = medallion_allocation['total_allocated']
    
    print(f"  Standard Kelly: ${standard_allocated:.0f} allocated, {len(portfolio_allocation['allocations'])} positions")
    print(f"  Confidence-Weighted: ${confidence_allocated:.0f} allocated, {len(confidence_portfolio_allocation['allocations'])} positions")
    print(f"  Medallion-Style: ${medallion_allocated:.0f} allocated, {len(medallion_allocation['allocations'])} positions")
    
    # Calculate differences
    conf_diff = confidence_allocated - standard_allocated
    medallion_diff = medallion_allocated - standard_allocated
    
    if standard_allocated > 0:
        conf_percent = ((confidence_allocated/standard_allocated - 1)*100)
        medallion_percent = ((medallion_allocated/standard_allocated - 1)*100)
        print(f"  Confidence vs Standard: ${conf_diff:+.0f} ({conf_percent:+.1f}%)")
        print(f"  Medallion vs Standard: ${medallion_diff:+.0f} ({medallion_percent:+.1f}%)")
    else:
        print(f"  Confidence vs Standard: ${conf_diff:+.0f} (N/A - no standard allocation)")
        print(f"  Medallion vs Standard: ${medallion_diff:+.0f} (N/A - no standard allocation)")
    
    # Show which approach is most conservative
    approaches = [
        ("Standard Kelly", standard_allocated),
        ("Confidence-Weighted", confidence_allocated),
        ("Medallion-Style", medallion_allocated)
    ]
    
    most_conservative = min(approaches, key=lambda x: x[1])
    most_aggressive = max(approaches, key=lambda x: x[1])
    
    print(f"\n  Most Conservative: {most_conservative[0]} (${most_conservative[1]:.0f})")
    print(f"  Most Aggressive: {most_aggressive[0]} (${most_aggressive[1]:.0f})")
    
    print(f"\n💡 MEDALLION-STYLE INSIGHTS:")
    print("-" * 50)
    print("• **Unified Risk-Reward Metric**: Combines Kelly, Sortino, and Calmar ratios")
    print("• **Kelly Criterion (40%)**: Optimal position sizing for growth")
    print("• **Sortino Ratio (30%)**: Focus on downside risk only")
    print("• **Calmar Ratio (30%)**: Drawdown control and capital preservation")
    print("• **Risk-Adjusted Kelly**: Kelly allocation weighted by unified score")
    print("• **Short-term focus**: Designed for active trading strategies")
    print("• **Medallion-inspired**: Based on principles of successful quantitative funds")
    
    # Options Analysis
    print(f"\n🚀 HIGH-REWARD OPTIONS OPPORTUNITIES")
    print("=" * 100)
    
    all_options = []
    for ticker in results_df['Ticker'].head(10):
        opportunities = analyze_options(ticker, with_greeks=True)
        if opportunities:
            for opt in opportunities:
                opt['ticker'] = ticker
                all_options.append(opt)
    
    if all_options:
        all_options.sort(key=lambda x: x['score'], reverse=True)
        print(f"\n✅ Found {len(all_options)} high-reward option opportunities:")
        for i, opt in enumerate(all_options[:5], 1):
            print(f"🥇 #{i}: {opt['ticker']} - Score: {opt['score']}")
            print(f"   Strike: ${opt['strike']:.2f} | Cost: ${opt['ask']:.2f} | Expiry: {opt['expiry']}")
            print(f"   If Stock +25%: {opt['return_25']:.0f}% return | If Stock +50%: {opt['return_50']:.0f}% return")
            print()
    else:
        print("No high-reward options found in screened stocks.")
    
    # Options Kelly allocation
    if all_options:
        options_allocation = calculate_options_kelly_allocation(all_options, portfolio_value=1000, scaling_factor=0.25)
        display_options_kelly_allocation(options_allocation, portfolio_value=1000)
    
    # Portfolio summary
    print(f"\n🎯 PORTFOLIO SUMMARY (${1000:,})")
    print("=" * 100)
    stock_allocation = portfolio_allocation['total_allocated']
    options_allocation_total = options_allocation['total_allocated'] if 'options_allocation' in locals() else 0
    total_combined = stock_allocation + options_allocation_total
    print(f"📊 Combined Allocation: Stocks ${stock_allocation:.2f} | Options ${options_allocation_total:.2f} | Total ${total_combined:.2f}")
    
    # Reddit posting removed

# =============================================================================
# MAIN EXECUTION FUNCTION
# =============================================================================

if __name__ == "__main__":
    main()

