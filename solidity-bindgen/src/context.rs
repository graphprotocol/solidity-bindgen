use crate::SafeSecretKey;
use secp256k1::key::SecretKey;
use std::convert::TryInto as _;
use std::sync::Arc;
use web3::api::Eth;
use web3::transports::Http;
use web3::types::Address;
use web3::Web3;

/// Common data associated with multiple contracts.
#[derive(Clone)]
pub struct Context(Arc<ContextInner>);

struct ContextInner {
    from: Address,
    secret_key: SafeSecretKey,
    // We are not expecting to interact with the chain frequently,
    // and the websocket transport has problems with ping.
    // So, the Http transport seems like the best choice.
    eth: Eth<Http>,
}

impl Context {
    pub fn new(
        url: &str,
        from: Address,
        secret_key: &SecretKey,
    ) -> Result<Self, web3::error::Error> {
        let transport = Http::new(url)?;
        let web3 = Web3::new(transport);
        let eth = web3.eth();
        let inner = ContextInner {
            eth,
            from,
            secret_key: secret_key.try_into().unwrap(),
        };
        Ok(Self(Arc::new(inner)))
    }

    pub fn from(&self) -> Address {
        self.0.from
    }

    pub(crate) fn secret_key(&self) -> &SecretKey {
        &self.0.secret_key
    }

    pub(crate) fn eth(&self) -> Eth<Http> {
        self.0.eth.clone()
    }
}
