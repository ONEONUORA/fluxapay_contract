use crate::{
    merchant_registry::{MerchantRegistry, MerchantRegistryClient},
    PaymentProcessor, PaymentProcessorClient, SwapAndPayArgs,
};
use soroban_sdk::{
    contract, contractimpl, contracterror, testutils::Address as _, vec, Address,
    Env, String, Symbol, Vec,
};

#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MockDexError {
    SwapFailed = 1,
    InvalidPath = 2,
    InsufficientOutput = 3,
}

#[contract]
pub struct MockDexRouter;

#[cfg_attr(
    any(not(target_arch = "wasm32"), feature = "contract-payment-processor"),
    contractimpl
)]
impl MockDexRouter {
    pub fn swap_exact_tokens_for_tokens(
        env: Env,
        _amount_in: i128,
        amount_out_min: i128,
        path: Vec<Address>,
        _to: Address,
        _deadline: u64,
    ) -> Result<Vec<i128>, MockDexError> {
        if path.len() < 2 {
            return Err(MockDexError::InvalidPath);
        }

        let stored_output = env.storage().persistent().get::<_, i128>(&Symbol::new(&env, "output"));

        let amounts = match stored_output {
            Some(output) => {
                if output < amount_out_min {
                    return Err(MockDexError::InsufficientOutput);
                }
                let mut result = vec![&env];
                result.push_back(_amount_in);
                result.push_back(output);
                result
            }
            None => return Err(MockDexError::SwapFailed),
        };

        Ok(amounts)
    }
}

fn setup_swap_test_env(
    env: &Env,
) -> (
    Address,
    PaymentProcessorClient<'_>,
    MerchantRegistryClient<'_>,
    Address,
    Address,
) {
    let payment_processor = env.register(PaymentProcessor, ());
    let merchant_registry = env.register(MerchantRegistry, ());

    let payment_client = PaymentProcessorClient::new(env, &payment_processor);
    let merchant_client = MerchantRegistryClient::new(env, &merchant_registry);

    let admin = Address::generate(env);
    payment_client.initialize_payment_processor(&admin);
    merchant_client.initialize(&admin);
    payment_client.set_merchant_registry_address(&admin, &merchant_registry);

    let token_in = Address::generate(env);
    let token_out = Address::generate(env);
    payment_client.allow_token(&admin, &token_out);

    (
        admin,
        payment_client,
        merchant_client,
        token_in,
        token_out,
    )
}

#[test]
fn test_swap_and_pay_happy_path_with_mock_dex() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, payment_client, merchant_client, token_in, token_out) =
        setup_swap_test_env(&env);

    // Register a mock DEX router
    let mock_dex = env.register(MockDexRouter, ());

    // Set up the mock to return a specific amount_out
    env.storage()
        .persistent()
        .set(&Symbol::new(&env, "output"), &10_000i128);

    // Register merchant
    let merchant = Address::generate(&env);
    merchant_client.register_merchant(
        &merchant,
        &String::from_str(&env, "TestShop"),
        &String::from_str(&env, "USD"),
        &None,
        &None,
        &None,
    );
    merchant_client.verify_merchant(&admin, &merchant);
    payment_client.grant_role(&admin, &Symbol::new(&env, "MERCHANT"), &merchant);

    let payer = Address::generate(&env);
    let deposit_address = Address::generate(&env);

    let path = vec![&env, token_in.clone(), token_out.clone()];
    let args = SwapAndPayArgs {
        payer: payer.clone(),
        payment_id: String::from_str(&env, "SWAP_TEST_001"),
        merchant_id: merchant.clone(),
        amount: 9_900,
        currency: Symbol::new(&env, "USDC"),
        deposit_address: deposit_address.clone(),
        token_in: token_in.clone(),
        amount_in: 10_000,
        amount_out_min: 9_900,
        path,
        expires_at: Some(env.ledger().timestamp() + 3600),
        dex_router: mock_dex.clone(),
        fx_oracle: None,
        oracle_pair: None,
        max_deviation_bps: 0,
    };

    let payment = payment_client.swap_and_pay(&args);
    assert_eq!(payment.payment_id, args.payment_id);
    assert_eq!(payment.status, crate::PaymentStatus::Settled);
}

#[test]
#[should_panic(expected = "Error(Contract, #28)")]
fn test_swap_and_pay_failure_with_mock_dex_insufficient_output() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, payment_client, merchant_client, token_in, token_out) =
        setup_swap_test_env(&env);

    // Register a mock DEX router that returns insufficient output
    let mock_dex = env.register(MockDexRouter, ());

    // Set up the mock to return less than minimum required
    env.storage()
        .persistent()
        .set(&Symbol::new(&env, "output"), &8_000i128);

    // Register merchant
    let merchant = Address::generate(&env);
    merchant_client.register_merchant(
        &merchant,
        &String::from_str(&env, "TestShop"),
        &String::from_str(&env, "USD"),
        &None,
        &None,
        &None,
    );
    merchant_client.verify_merchant(&admin, &merchant);
    payment_client.grant_role(&admin, &Symbol::new(&env, "MERCHANT"), &merchant);

    let payer = Address::generate(&env);
    let deposit_address = Address::generate(&env);

    let path = vec![&env, token_in.clone(), token_out.clone()];
    let args = SwapAndPayArgs {
        payer: payer.clone(),
        payment_id: String::from_str(&env, "SWAP_TEST_002"),
        merchant_id: merchant.clone(),
        amount: 9_900,
        currency: Symbol::new(&env, "USDC"),
        deposit_address: deposit_address.clone(),
        token_in: token_in.clone(),
        amount_in: 10_000,
        amount_out_min: 9_900,
        path,
        expires_at: Some(env.ledger().timestamp() + 3600),
        dex_router: mock_dex.clone(),
        fx_oracle: None,
        oracle_pair: None,
        max_deviation_bps: 0,
    };

    payment_client.swap_and_pay(&args);
}
