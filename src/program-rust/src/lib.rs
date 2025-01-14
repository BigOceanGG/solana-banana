pub mod account_state;
pub mod error;
pub mod shared;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    system_instruction,
    program::invoke,
};
use std::collections::HashMap;
use crate::{
    account_state::ProgramAccountState,
};

entrypoint!(process_instruction);

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("Solana split transfer contract called.");

    if instruction_data.is_empty() {
        return Err(ProgramError::InvalidInstructionData);
    }

    let mut amount_bytes = [0u8; 8];
    amount_bytes.copy_from_slice(&instruction_data[1..9]);
    let amount = u64::from_le_bytes(amount_bytes);

    match instruction_data[0] {
        0 => deposit(accounts, amount),
        1 => withdraw(accounts, amount),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}

fn deposit(accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let depositor_account = next_account_info(account_info_iter)?;
    let contract_account = next_account_info(account_info_iter)?;

    let deposit_instruction1 = system_instruction::transfer(
        &depositor_account.key,
        &contract_account.key,
        amount/2,
    );
    invoke(
        &deposit_instruction1,
        &[depositor_account.clone(), contract_account.clone()],
    )?;

    let deposit_instruction2 = system_instruction::transfer(
        &depositor_account.key,
        &contract_account.key,
        amount/2,
    );

    invoke(
        &deposit_instruction2,
        &[depositor_account.clone(), contract_account.clone()],
    )?;

    let mut account_data = contract_account.data.borrow_mut();
    let mut account_state = ProgramAccountState::unpack_unchecked(&account_data)?;
    if account_state.is_initialized() {
    } else {
        account_state.set_initialized();
    }
    account_state.deposit(depositor_account.key.to_string(), amount)?;
    ProgramAccountState::pack(account_state, &mut account_data)?;

    Ok(())
}

fn withdraw(accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let contract_account = next_account_info(account_info_iter)?;
    let withdrawer_account = next_account_info(account_info_iter)?;

    if contract_account.lamports() < amount {
        msg!("Insufficient balance. {}", contract_account.lamports());
        return Err(ProgramError::InsufficientFunds);
    }
    **contract_account.lamports.borrow_mut() -= amount;
    **withdrawer_account.lamports.borrow_mut() += amount/2;
    **withdrawer_account.lamports.borrow_mut() += amount/2;

    let mut account_data = contract_account.data.borrow_mut();
    let mut account_state = ProgramAccountState::unpack_unchecked(&account_data)?;
    if account_state.is_initialized() {
    } else {
        account_state.set_initialized();
    }
    account_state.withdraw(withdrawer_account.key.to_string(), amount)?;
    ProgramAccountState::pack(account_state, &mut account_data)?;

    Ok(())
}
