use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use starknet::{
    core::types::{Felt, StarknetError},
    providers::Provider,
};

use super::{jediswap::JediswapPool, types::Reserves};

#[async_trait]
pub trait AutomatedMarketMaker {
    /// Returns the address of the AMM.
    fn address(&self) -> Felt;

    /// Returns a vector of tokens in the AMM.
    fn tokens(&self) -> Vec<Felt>;

    /// Calculates a f64 representation of base token price in the AMM.
    fn calculate_price(&self, base_token: Felt, quote_token: Felt) -> Result<f64, StarknetError>;

    /// Locally simulates a swap in the AMM.
    ///
    /// Returns the amount received for `amount_in` of `token_in`.
    async fn simulate_swap<P>(
        &self,
        base_token: Felt,
        quote_token: Felt,
        amount_in: Felt,
        provider: Arc<P>,
    ) -> Result<Felt, StarknetError>
    where
        P: Provider + Send + Sync;

    /// Locally simulates a swap in the AMM.
    /// Mutates the AMM state to the state of the AMM after swapping.
    /// Returns the amount received for `amount_in` of `token_in`.
    fn simulate_swap_mut(
        &mut self,
        base_token: Felt,
        quote_token: Felt,
        amount_in: Felt,
    ) -> Result<Felt, StarknetError>;

    async fn get_reserves<P>(&mut self, provider: Arc<P>) -> Result<Reserves, StarknetError>
    where
        P: Provider + Sync + Send;
}

macro_rules! amm {
    ($($pool_type:ident),+ $(,)?) => {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub enum AMM {
            $($pool_type($pool_type),)+
        }

        #[async_trait]
        impl AutomatedMarketMaker for AMM {
            fn address(&self) -> Felt{
                match self {
                    $(AMM::$pool_type(pool) => pool.address(),)+
                }
            }


            async fn simulate_swap<P>(&self, base_token: Felt, quote_token: Felt, amount_in: Felt, provider: Arc<P>) -> Result<Felt, StarknetError> where P: Provider + Send + Sync {
                match self {
                    $(AMM::$pool_type(pool) => pool.simulate_swap(base_token, quote_token, amount_in, provider).await)+
                }
            }

            fn simulate_swap_mut(&mut self, base_token: Felt, quote_token: Felt, amount_in: Felt) -> Result<Felt, StarknetError> {
                match self {
                    $(AMM::$pool_type(pool) => pool.simulate_swap_mut(base_token, quote_token, amount_in),)+
                }
            }

            fn tokens(&self) -> Vec<Felt> {
                match self {
                    $(AMM::$pool_type(pool) => pool.tokens(),)+
                }
            }

            fn calculate_price(&self, base_token: Felt, quote_token: Felt) -> Result<f64, StarknetError> {
                match self {
                    $(AMM::$pool_type(pool) => pool.calculate_price(base_token, quote_token),)+
                }
            }


            async fn get_reserves<P>(&mut self, provider: Arc<P>) -> Result<Reserves, StarknetError>
            where
            P: Provider + Sync + Send
            {
                match self {

                        $(AMM::$pool_type(pool) => pool.get_reserves(provider).await)+
                }
            }
        }


        impl PartialEq for AMM {
            fn eq(&self, other: &Self) -> bool {
                self.address() == other.address()
            }
        }

        impl Eq for AMM {}
    };
}

amm!(JediswapPool);