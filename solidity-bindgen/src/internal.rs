use ethabi::Token;
use web3::contract::tokens::{Detokenize, Tokenizable};
use web3::contract::Error;

/// For types which might come up in contracts which are not yet implemented in web3
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

/// This type compensates for the fact that web3 doesn't impl Detokenize for ()
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
