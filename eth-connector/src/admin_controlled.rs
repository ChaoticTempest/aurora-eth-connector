pub type PausedMask = u8;

/// Admin control flow flag indicates that all control flow unpause (unblocked).
pub const UNPAUSE_ALL: PausedMask = 0;
/// Admin control flow flag indicates that the deposit is paused.
pub const PAUSE_DEPOSIT: PausedMask = 1 << 0;
/// Admin control flow flag indicates that withdrawal is paused.
pub const PAUSE_WITHDRAW: PausedMask = 1 << 1;

pub const ERR_PAUSED: &str = "ERR_PAUSED";

pub trait AdminControlled {
    /// Return the current mask representing all paused events.
    fn get_paused_flags(&self) -> PausedMask;

    /// Update mask with all paused events.
    /// Implementor is responsible for guaranteeing that this function can only be
    /// called by owner of the contract.
    fn set_paused_flags(&mut self, paused: PausedMask);

    /// Return if the contract is paused for the current flag and user
    fn is_paused(&self, flag: PausedMask, is_owner: bool) -> bool {
        (self.get_paused_flags() & flag) != 0 && !is_owner
    }

    /// Asserts the passed paused flag is not set. Returns `PausedError` if paused.
    fn assert_not_paused(&self, flag: PausedMask, is_owner: bool) -> Result<(), PausedError> {
        if self.is_paused(flag, is_owner) {
            Err(PausedError)
        } else {
            Ok(())
        }
    }
}

pub struct PausedError;

impl AsRef<[u8]> for PausedError {
    fn as_ref(&self) -> &[u8] {
        ERR_PAUSED.as_bytes()
    }
}
