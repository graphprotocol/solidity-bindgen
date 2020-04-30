pub use anyhow::{anyhow, Result};
use ethabi::Token;
use futures::compat::Future01CompatExt as _;
use std::sync::Arc;
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
        unimplemented!()
    }
    #[inline(always)]
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
    // Keeping a reference to this because if we drop it then interaction with
    // Ethereum ceases. This goes in an Arc because if we create too many of
    // these my macbook says there are too many open files.
    _event_loop_handle: Arc<EventLoopHandle>,
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

    pub fn new(
        address: Address,
        json_abi: &[u8],
        url: &str,
        event_loop_handle: Arc<EventLoopHandle>,
    ) -> Result<Self> {
        // We are not expecting to interact with the chain frequently,
        // and the websocket transport has problems with ping.
        // So, the Http transport seems like the best choice.
        let handle = event_loop_handle
            .remote()
            .handle()
            .expect("Handle for event loop should be alive");
        let transport = Http::with_event_loop(&url, &handle, 64)?;
        let web3 = Web3::new(transport);

        let contract =
            Contract::from_json(web3.eth(), address, json_abi).map_err(|e| anyhow!("{}", e))?;

        Ok(Self {
            _event_loop_handle: event_loop_handle,
            contract,
        })
    }
}
