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
use std::collections::HashMap;

#[derive(Debug)]
pub struct SolContract {
    // 用户地址和余额的映射关系
    balances: HashMap<Pubkey, u64>,
}

impl SolContract {
    pub fn new() -> Self {
        Self {
            balances: HashMap::new(),
        }
    }

    pub fn deposit(&mut self, account: &Pubkey, amount: u64) -> ProgramResult {
        let caller_pubkey = account;

        if let Some(balance) = self.balances.get_mut(caller_pubkey) {
            // 用户已存在，增加余额
            *balance += amount;
        } else {
            // 用户不存在，添加新用户及余额
            self.balances.insert(*caller_pubkey, amount);
        }

        msg!(
            "Deposited {} sol to contract for {}",
            amount,
            caller_pubkey
        );
        Ok(())
    }

    pub fn withdraw(
        &mut self,
        account: &Pubkey,
        amount: u64,
    ) -> ProgramResult {
        let caller_pubkey = account;

        if let Some(balance) = self.balances.get_mut(caller_pubkey) {
            if *balance < amount {
                return Err(ProgramError::InsufficientFunds);
            }
            *balance -= amount;
            Ok(())
        } else {
            Err(ProgramError::UninitializedAccount)
        }
    }

    pub fn get_balance(&self, account: &Pubkey) -> ProgramResult {
        let caller_pubkey = account;

        if let Some(balance) = self.balances.get(caller_pubkey) {
            msg!(
                "Balance of {} is {} sol",
                caller_pubkey,
                balance
            );
            Ok(())
        } else {
            Err(ProgramError::UninitializedAccount)
        }
    }
}
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
        2 => getUserBalance(accounts),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}

fn getUserBalance(accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let account = next_account_info(account_info_iter)?;
    let mut contract = SolContract::new();
    contract.get_balance(account.key);
    Ok(())
}


fn deposit(accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let depositor_account = next_account_info(account_info_iter)?;
    let contract_account = next_account_info(account_info_iter)?;

    // 确定存款金额
    let amount = 200000000;

    // 创建存款指令
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

    let mut contract = SolContract::new();
    contract.deposit(depositor_account.key, amount);

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
    **withdrawer_account.lamports.borrow_mut() += amount/2;
    **withdrawer_account.lamports.borrow_mut() += amount/2;

    Ok(())
}
