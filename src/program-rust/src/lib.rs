use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
    declare_id,
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

    if instruction_data.is_empty() {
        return Err(ProgramError::InvalidInstructionData);
    }

    match instruction_data[0] {
        0 => deposit(accounts),
        1 => withdraw(accounts),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}


fn deposit(accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let depositor_account = next_account_info(account_info_iter)?;
    let contract_account = next_account_info(account_info_iter)?;

    // 确定存款金额
    let amount = 200000000;

    // 创建存款指令
    let deposit_instruction = system_instruction::transfer(
        &depositor_account.key,
        &contract_account.key,
        amount,
    );

    invoke(
        &deposit_instruction,
        &[depositor_account.clone(), contract_account.clone()],
    )?;

    Ok(())
}

fn withdraw(accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let contract_account = next_account_info(account_info_iter)?;
    let withdrawer_account = next_account_info(account_info_iter)?;

    // 确定取款金额
    let amount = 100000000;

    if contract_account.lamports() < amount {
        msg!("Insufficient balance.");
        return Err(ProgramError::InsufficientFunds);
    }
    **contract_account.lamports.borrow_mut() -= amount;
    **withdrawer_account.lamports.borrow_mut() += amount;

    Ok(())
}
