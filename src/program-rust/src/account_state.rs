//! @brief account_state manages account data
use crate::error::SampleError;
use crate::shared::ACCOUNT_STATE_SPACE;
use solana_program::{
    msg,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
};
use std::collections::BTreeMap;

/// Maintains global accumulator
#[derive(Debug, Default, PartialEq)]
pub struct ProgramAccountState {
    is_initialized: bool,
    btree_storage: BTreeMap<String, u64>,
}

impl ProgramAccountState {
    ///
    pub fn set_initialized(&mut self) {
        self.is_initialized = true;
    }
    /// Adds a new key/value pair to the account
    pub fn deposit(&mut self, key: String, value: u64) -> ProgramResult {
        match self.btree_storage.contains_key(&key) {
            true => {
                 if let Some(balance) = self.btree_storage.get_mut(&key) {
                    *balance += value;
                 }
                 Ok(())
            },
            false => {
                self.btree_storage.insert(key, value);
                Ok(())
            }
        }
    }
    pub fn withdraw(&mut self, key: String, value: u64) -> ProgramResult {
        if let Some(balance) = self.btree_storage.get_mut(&key) {
            if *balance < value {
                return Err(ProgramError::InsufficientFunds);
            }
            *balance -= value;
            Ok(())
        } else {
            Err(ProgramError::UninitializedAccount)
        }
    }
    /// Removes a key from account and returns the keys value
    pub fn remove(&mut self, key: &str) -> Result<u64, SampleError> {
        match self.btree_storage.contains_key(key) {
            true => Ok(self.btree_storage.remove(key).unwrap()),
            false => Err(SampleError::KeyNotFoundInAccount),
        }
    }
}

impl Sealed for ProgramAccountState {}

impl IsInitialized for ProgramAccountState {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for ProgramAccountState {
    const LEN: usize = ACCOUNT_STATE_SPACE;

    /// Store 'state' of account to its data area
    fn pack_into_slice(&self, dst: &mut [u8]) {
        crate::shared::pack_into_slice(self.is_initialized, &self.btree_storage, dst);
    }

    /// Retrieve 'state' of account from account data area
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        match crate::shared::unpack_from_slice(src) {
            Ok((is_initialized, btree_map)) => Ok(ProgramAccountState {
                is_initialized,
                btree_storage: btree_map,
            }),
            Err(_) => Err(ProgramError::InvalidAccountData),
        }
    }
}
