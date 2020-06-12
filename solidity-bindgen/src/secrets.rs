use secp256k1::key::{SecretKey, ONE_KEY};

use std::fmt;
use std::ops::Deref;
use std::pin::Pin;
use zeroize::{DefaultIsZeroes, Zeroize};

/// Securely stores a secret key in memory that is zeroized on drop. Care is
/// taken so that when this struct is constructed or moved that additional
/// copies of the secret are not made in memory.
/// https://github.com/veorq/cryptocoding#clean-memory-of-secret-data
///
/// Unfortunately the SafeSecretKey is not magic, there are some things to be
/// aware of when using it...
///   * The memory used when constructing the secret key must also be zeroized,
///     but this is left as an exercise to the caller.
///   * If you mem::forget the SafeSecretKey or otherwise don't drop it, the
///     secret will not be zeroized.
///   * When the caller lends out a reference to the SecretKey (available for
///     example via Deref) it is the responsibility of the caller to not Clone
///     the SecretKey or otherwise make a copy of it's memory
pub struct SafeSecretKey {
    safe: Pin<Box<ZeroizedSecretKey>>,
}

impl Clone for SafeSecretKey {
    fn clone(&self) -> Self {
        Self::new(&self)
    }
}

impl fmt::Debug for SafeSecretKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SecretKey").finish()
    }
}

#[derive(Copy, Clone)]
struct ZeroizedSecretKey(SecretKey);

impl DefaultIsZeroes for ZeroizedSecretKey {}
impl Default for ZeroizedSecretKey {
    fn default() -> Self {
        Self(ONE_KEY)
    }
}

impl SafeSecretKey {
    fn new(secret: &SecretKey) -> Self {
        // Allocate to a fixed location in memory
        let mut safe = Pin::new(Box::<ZeroizedSecretKey>::default());

        // Copy to that location without moving secret onto the stack
        safe.0 = *secret;

        // Now the only copy of the memory managed by Self is committed to be zeroized later.
        // This operation (and subsequent) don't make copies because it's the Pin that moves,
        // not the underlying allocation.
        Self { safe }
    }
}

impl<'a> From<&'a SecretKey> for SafeSecretKey {
    fn from(secret: &'a SecretKey) -> Self {
        Self::new(secret)
    }
}

impl Drop for SafeSecretKey {
    fn drop(&mut self) {
        self.safe.zeroize()
    }
}

impl Deref for SafeSecretKey {
    type Target = SecretKey;
    fn deref(&self) -> &Self::Target {
        &self.safe.0
    }
}
