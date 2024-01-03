use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
    program_error::ProgramError,
    program_pack::Pack,
    system_instruction,
    program::invoke,
};

entrypoint!(process_instruction);

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("Solana split transfer contract called.");

    if instruction_data.len() < 8 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let mut amount_bytes = [0u8; 8];
    amount_bytes.copy_from_slice(&instruction_data[0..8]);
    let amount = u64::from_le_bytes(amount_bytes);

    let account_info_iter = &mut accounts.iter();
    let sender_account = next_account_info(account_info_iter)?;
    let receiver1_account = next_account_info(account_info_iter)?;
    let receiver2_account = next_account_info(account_info_iter)?;
    let system_program_account = next_account_info(account_info_iter)?;

    if !sender_account.is_signer {
        msg!("Error: Sender account is not a signer");
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !receiver1_account.is_writable || !receiver2_account.is_writable{
        msg!("Error: Receiver account is not writable");
        return Err(ProgramError::InvalidAccountData);
    }

    if sender_account.lamports() < amount {
        msg!("Insufficient balance.");
        return Err(ProgramError::InsufficientFunds);
    }

    invoke(
        &system_instruction::transfer(
                 &sender_account.key,
                 &receiver1_account.key,
                 amount/20,
             ),
        &[
            sender_account.clone(),
            receiver1_account.clone(),
            system_program_account.clone(),
        ],
    )?;

    invoke(
       &system_instruction::transfer(
               &sender_account.key,
               &receiver2_account.key,
               19*amount/20,
            ),
       &[
            sender_account.clone(),
            receiver2_account.clone(),
            system_program_account.clone(),
       ],
    )?;

    Ok(())
}
