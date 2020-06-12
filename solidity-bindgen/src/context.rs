use crate::SafeSecretKey;
use secp256k1::key::SecretKey;
use std::sync::Arc;
use web3::transports::EventLoopHandle;
use web3::types::Address;

/// A type needed to instantiate contracts. All contracts instantiated with the
/// same Context will share the same EventLoopHandle. As long as the contracts
/// are not dropped, the EventLoopHandle will not be dropped either. This type
/// is cheap to clone.
#[derive(Clone)]
pub struct Context(Arc<ContextInner>);

struct ContextInner {
    url: String,
    handle: EventLoopHandle,
    from: Address,
    secret_key: SafeSecretKey,
}

impl Context {
    pub fn new(
        url: String,
        from: Address,
        secret_key: &SecretKey,
    ) -> Result<Self, web3::error::Error> {
        let handle = EventLoopHandle::spawn(|_| Ok(()))?.0;
        let inner = ContextInner {
            url,
            handle,
            from,
            secret_key: secret_key.into(),
        };
        Ok(Self(Arc::new(inner)))
    }

    pub fn url(&self) -> &str {
        &self.0.url
    }

    pub fn from(&self) -> Address {
        self.0.from
    }

    pub(crate) fn handle(&self) -> &EventLoopHandle {
        &self.0.handle
    }

    pub(crate) fn secret_key(&self) -> &SecretKey {
        &self.0.secret_key
    }
}
