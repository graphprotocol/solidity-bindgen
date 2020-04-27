pub use anyhow::{anyhow, Result};
use ethabi::Token;
use futures::compat::Future01CompatExt as _;
use web3::contract::tokens::{Detokenize, Tokenizable, Tokenize};
use web3::contract::{Contract, Error};
use web3::transports::{EventLoopHandle, Http};
use web3::types::Address;
use web3::Web3;

pub enum Unimplemented {}

impl Tokenizable for Unimplemented {
    fn from_token(_: Token) -> Result<Self, Error>
    where
        Self: Sized,
    {
        unsafe { std::hint::unreachable_unchecked() }
    }
    fn into_token(self) -> Token {
        unsafe { std::hint::unreachable_unchecked() }
    }
}

pub struct Empty;
impl Detokenize for Empty {
    fn from_tokens(tokens: Vec<Token>) -> std::result::Result<Self, Error>
    where
        Self: Sized,
    {
        if tokens.is_empty() {
            Ok(Empty)
        } else {
            Err(Error::InvalidOutputType("Expected no tokens".to_owned()))
        }
    }
}

pub struct ContractWrapper {
    // Keeping a reference to this because if we drop it then interaction with Ethereum ceases.
    //
    // We could get really fancy and use a MaybeOwned here, or maybe just an Arc
    // to manage EventLoopHandles for multiple contracts, but it's not likely worth the effort
    _event_loop_handle: EventLoopHandle,
    contract: Contract<Http>,
}

impl ContractWrapper {
    /// Mostly exists to map to the new futures.
    /// This is the "untyped" API which the generated types will use
    pub async fn query<T: Detokenize>(
        &self,
        name: &'static str,
        params: impl Tokenize,
    ) -> Result<T> {
        self.contract
            .query(name, params, None, Default::default(), None)
            .compat()
            .await
            .map_err(|e| anyhow!("{}", e))
    }

    pub async fn non_pure_todo<T: Detokenize>(
        &self,
        _name: &'static str,
        _params: impl Tokenize,
    ) -> Result<T> {
        todo!()
    }

    pub async fn new(address: Address, json_abi: &[u8], url: &str) -> Result<Self> {
        // We are not expecting to interact with the chain frequently,
        // and the websocket transport has problems with ping.
        // So, the Http transport seems like the best choice.
        let (_event_loop_handle, http) = Http::new(&url)?;
        let web3 = Web3::new(http);

        let contract =
            Contract::from_json(web3.eth(), address, json_abi).map_err(|e| anyhow!("{}", e))?;

        Ok(Self {
            _event_loop_handle,
            contract,
        })
    }
}
