// Updated DexRouter with pre-execution checks, path validation, price impact guard, and fallback logic.
use soroban_sdk::{contract, contractimpl, token, Address, Env, Vec, Symbol};

/// DEX Router interface for Soroswap-style swaps.
/// This provides a generic interface for atomic token swaps.
#[contract]
pub struct DexRouter;

#[cfg_attr(
    any(not(target_arch = "wasm32"), feature = "contract-dex-router"),
    contractimpl
)]
impl DexRouter {
    /// Get the router's factory address.
    pub fn factory(env: Env) -> Address {
        // In a real implementation, this would call the router's factory() method
        // For now, we return a placeholder that can be configured
        env.current_contract_address()
    }

    /// Get the path length for a swap.
    pub fn get_amounts_out(env: Env, amount_in: i128, path: Vec<Address>) -> Vec<i128> {
        // Returns cumulative output per hop: amounts[0] = amount_in, amounts[i] = output after hop i.
        let mut amounts = Vec::new(&env);
        if path.is_empty() {
            return amounts;
        }

        amounts.push_back(amount_in);
        let mut current = amount_in;
        for i in 1..path.len() {
            let _token_out = path.get(i).unwrap();
            // Simulate per-hop slippage for quote estimation (real impl delegates to router).
            current = current.saturating_mul(99).saturating_div(100);
            amounts.push_back(current);
        }
        amounts
    }

    /// Internal: Validate that the provided path is non-empty and has at least two hops.
    fn validate_path(path: &Vec<Address>) -> Result<(), &'static str> {
        if path.len() < 2 {
            return Err("Path must contain at least two token addresses");
        }
        // Ensure no consecutive duplicate addresses.
        for i in 1..path.len() {
            if path.get(i) == path.get(i - 1) {
                return Err("Path contains duplicate consecutive token addresses");
            }
        }
        Ok(())
    }

    /// Internal: Placeholder liquidity check. In a real implementation this would query reserves.
    fn check_liquidity(_env: &Env, _path: &Vec<Address>) -> bool {
        // Assume sufficient liquidity for now.
        true
    }

    /// Internal: Simple price impact guard – reject swaps that would lose >5% of input value.
    fn price_impact_guard(input: i128, output: i128) -> Result<(), &'static str> {
        // Compute price impact as (input - output) / input.
        if input <= 0 {
            return Err("Invalid input amount");
        }
        let impact_basis_points = ((input - output) * 10_000) / input; // basis points
        if impact_basis_points > 500 {
            // >5% impact
            return Err("Price impact exceeds 5% guardrail");
        }
        Ok(())
    }

    /// Swap exact tokens for tokens with pre‑execution checks and fallback.
    /// amount_in: exact amount of input tokens to spend
    /// amount_out_min: minimum amount of output tokens required
    /// path: array of token addresses [token_in, token_out]
    /// to: address to receive output tokens
    /// deadline: Unix timestamp after which the swap reverts
    pub fn swap_exact_tokens_for_tokens(
        env: Env,
        amount_in: i128,
        amount_out_min: i128,
        mut path: Vec<Address>,
        to: Address,
        deadline: u64,
    ) -> Result<Vec<i128>, &'static str> {
        // ----- Pre‑execution checks -----
        Self::validate_path(&path)?;
        if !Self::check_liquidity(&env, &path) {
            return Err("Insufficient liquidity for the requested path");
        }

        // Attempt primary swap.
        let primary_result = Self::execute_swap(&env, amount_in, amount_out_min, &path, to.clone(), deadline);
        if primary_result.is_ok() {
            return primary_result;
        }

        // ----- Fallback logic -----
        // Simple fallback: reverse the path and try again.
        let mut fallback_path = path.clone();
        fallback_path.reverse();
        // If the reversed path is identical (e.g., length 2 with same tokens) we skip to refund.
        if fallback_path == path {
            // Refund the caller as the swap failed entirely.
            Self::refund_caller(&env, to.clone(), amount_in)?;
            return Err("Swap failed; fallback not available, refunded caller");
        }
        // Try the fallback path.
        let fallback_result = Self::execute_swap(&env, amount_in, amount_out_min, &fallback_path, to.clone(), deadline);
        if fallback_result.is_ok() {
            // Emit a fallback event.
            env.events().publish(
                (Symbol::new(&env, "SWAP"), Symbol::new(&env, "FALLBACK")),
                (amount_in, to.clone()),
            );
            return fallback_result;
        }

        // If fallback also fails, refund caller.
        Self::refund_caller(&env, to.clone(), amount_in)?;
        Err("Swap and fallback both failed; caller refunded")
    }

    /// Internal helper that performs the core swap logic and enforces price‑impact guard.
    fn execute_swap(
        env: &Env,
        amount_in: i128,
        amount_out_min: i128,
        path: &Vec<Address>,
        to: Address,
        deadline: u64,
    ) -> Result<Vec<i128>, &'static str> {
        // Get the expected output amounts.
        let amounts = Self::get_amounts_out(env.clone(), amount_in, path.clone());
        if amounts.is_empty() {
            return Err("No output amounts calculated");
        }
        let final_output = *amounts.get(amounts.len() - 1).unwrap_or(&0);
        // Guard against slippage and price impact.
        if final_output < amount_out_min {
            return Err("Slippage protection triggered: output is below minimum required");
        }
        Self::price_impact_guard(amount_in, final_output)?;
        // Emit successful swap event.
        env.events().publish(
            (Symbol::new(&env, "SWAP"), Symbol::new(&env, "EXECUTED")),
            (amount_in, final_output, to, deadline),
        );
        Ok(amounts)
    }

    /// Internal: Refund the caller by transferring the input tokens back.
    fn refund_caller(env: &Env, recipient: Address, amount: i128) -> Result<(), &'static str> {
        // In a real implementation this would invoke the token contract's transfer.
        // Here we simply emit a Refund event for visibility.
        env.events().publish(
            (Symbol::new(&env, "REFUND"), Symbol::new(&env, "CALLER")),
            (recipient, amount),
        );
        Ok(())
    }

    /// Swap tokens for exact tokens.
    /// amount_out: exact amount of output tokens required
    /// amount_in_max: maximum amount of input tokens to spend
    /// path: array of token addresses [token_in, token_out]
    /// to: address to receive output tokens
    /// deadline: Unix timestamp after which the swap reverts
    pub fn swap_tokens_for_exact_tokens(
        env: Env,
        amount_out: i128,
        _amount_in_max: i128,
        path: Vec<Address>,
        _to: Address,
        _deadline: u64,
    ) -> Vec<i128> {
        // Similar to swap_exact_tokens_for_tokens but for exact output
        Symbol::new(&env, "SWAP");
        Symbol::new(&env, "EXECUTED");

        let mut amounts = Vec::new(&env);
        for _ in 0..path.len() {
            amounts.push_back(amount_out);
        }
        amounts
    }
}


/// DEX Router interface for Soroswap-style swaps.
/// This provides a generic interface for atomic token swaps.
#[contract]
pub struct DexRouter;

#[cfg_attr(
    any(not(target_arch = "wasm32"), feature = "contract-dex-router"),
    contractimpl
)]
impl DexRouter {
    /// Get the router's factory address.
    pub fn factory(env: Env) -> Address {
        // In a real implementation, this would call the router's factory() method
        // For now, we return a placeholder that can be configured
        env.current_contract_address()
    }

    /// Get the path length for a swap.
    pub fn get_amounts_out(env: Env, amount_in: i128, path: Vec<Address>) -> Vec<i128> {
        // Returns cumulative output per hop: amounts[0] = amount_in, amounts[i] = output after hop i.
        let mut amounts = Vec::new(&env);
        if path.is_empty() {
            return amounts;
        }

        amounts.push_back(amount_in);
        let mut current = amount_in;
        for i in 1..path.len() {
            let _token_out = path.get(i).unwrap();
            // Simulate per-hop slippage for quote estimation (real impl delegates to router).
            current = current.saturating_mul(99).saturating_div(100);
            amounts.push_back(current);
        }
        amounts
    }

    /// Swap exact tokens for tokens.
    /// amount_in: exact amount of input tokens to spend
    /// amount_out_min: minimum amount of output tokens required
    /// path: array of token addresses [token_in, token_out]
    /// to: address to receive output tokens
    /// deadline: Unix timestamp after which the swap reverts
    /// Swap exact tokens for tokens.
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `amount_in` - Exact amount of input tokens to spend
    /// * `amount_out_min` - Minimum amount of output tokens required
    /// * `path` - Array of token addresses [token_in, token_out]
    /// * `to` - Address to receive output tokens
    /// * `deadline` - Unix timestamp after which the swap reverts
    /// 
    /// # Returns
    /// Vector of amounts representing the output at each hop
    /// 
    /// # Errors
    /// Returns an error if the actual output falls below `amount_out_min`
    pub fn swap_exact_tokens_for_tokens(
        env: Env,
        amount_in: i128,
        amount_out_min: i128,
        path: Vec<Address>,
        to: Address,
        deadline: u64,
    ) -> Result<Vec<i128>, &'static str> {
        // In a real implementation, this would:
        // 1. Transfer input tokens from caller to router
        // 2. Call router's swapExactTokensForTokens
        // 3. Transfer output tokens to 'to' address
        // 4. Return the amounts swapped

        // Get the expected output amounts
        let amounts = Self::get_amounts_out(env.clone(), amount_in, path.clone());
        
        // Check if we have at least one output amount
        if amounts.len() == 0 {
            return Err("No output amounts calculated");
        }
        
        // Get the final output amount (last element in the amounts vector)
        let final_output = amounts.get(amounts.len() - 1).unwrap_or(&0);
        
        // Revert if output falls below amount_out_min (issue #217)
        if *final_output < amount_out_min {
            return Err("Slippage protection triggered: output is below minimum required");
        }

        // Emit SWAP/EXECUTED event
        env.events().publish(
            (soroban_sdk::Symbol::new(&env, "SWAP"), soroban_sdk::Symbol::new(&env, "EXECUTED")),
            (amount_in, *final_output, to, deadline),
        );

        Ok(amounts)
    }

    /// Swap tokens for exact tokens.
    /// amount_out: exact amount of output tokens required
    /// amount_in_max: maximum amount of input tokens to spend
    /// path: array of token addresses [token_in, token_out]
    /// to: address to receive output tokens
    /// deadline: Unix timestamp after which the swap reverts
    pub fn swap_tokens_for_exact_tokens(
        env: Env,
        amount_out: i128,
        _amount_in_max: i128,
        path: Vec<Address>,
        _to: Address,
        _deadline: u64,
    ) -> Vec<i128> {
        // Similar to swap_exact_tokens_for_tokens but for exact output
        soroban_sdk::Symbol::new(&env, "SWAP");
        soroban_sdk::Symbol::new(&env, "EXECUTED");

        let mut amounts = Vec::new(&env);
        for _ in 0..path.len() {
            amounts.push_back(amount_out);
        }
        amounts
    }
}
