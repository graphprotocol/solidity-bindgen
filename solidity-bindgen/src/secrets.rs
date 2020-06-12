use secp256k1::key::{SecretKey, ONE_KEY};

use sodiumoxide::utils::{mlock, munlock};
use std::convert::TryFrom;
use std::fmt;
use std::ops::Deref;
use std::pin::Pin;
use std::slice;
use zeroize::{DefaultIsZeroes, Zeroize};

/// Securely stores a secret key in memory that is zeroized on drop. Care is
/// taken so that when this struct is constructed or moved that additional
/// copies of the secret are not made in memory or disk.
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

impl ZeroizedSecretKey {
    fn as_mut_bytes(&mut self) -> &mut [u8] {
        unsafe {
            let ptr = self.0.as_mut_ptr();
            slice::from_raw_parts_mut(ptr, self.0.len())
        }
    }
}

impl SafeSecretKey {
    fn new(secret: &SecretKey) -> Result<Self, ()> {
        // Allocate to a fixed location in memory
        let mut safe = Pin::new(Box::<ZeroizedSecretKey>::default());

        // Tell the OS that it's not ok to page this memory to disk
        let mem = safe.as_mut_bytes();
        mlock(mem)?;

        // Copy to that location without moving secret onto the stack
        safe.0 = *secret;

        // Now the only copy of the memory managed by Self is committed to be zeroized later.
        // This operation (and subsequent) don't make copies because it's the Pin that moves,
        // not the underlying allocation.
        Ok(Self { safe })
    }
}

impl<'a> TryFrom<&'a SecretKey> for SafeSecretKey {
    type Error = ();
    fn try_from(secret: &'a SecretKey) -> Result<Self, ()> {
        Self::new(secret)
    }
}

impl Drop for SafeSecretKey {
    fn drop(&mut self) {
        self.safe.zeroize();
        let mem = self.safe.as_mut_bytes();
        // If the OS cannot unlock this memory, it's not a problem. There is no
        // reasonable way to propagate an error so just ignore it.
        let _ignore = munlock(mem);
    }
}

impl Deref for SafeSecretKey {
    type Target = SecretKey;
    fn deref(&self) -> &Self::Target {
        &self.safe.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secp256k1::key::ONE_KEY;

    /// It's a bit hard to verify the inner workings like zeroing without relying on undefined behavior.
    /// But, we can check that this at least runs
    #[test]
    pub fn no_panic() {
        let key = ONE_KEY;
        let safe = SafeSecretKey::new(&key).unwrap();
        drop(safe);
    }
}
