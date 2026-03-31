use crate::*;
use anchor_client::anchor_lang::InstructionData;
use anchor_client::anchor_lang::ToAccountMetas;
use anchor_client::solana_client::rpc_client::RpcClient;
use anchor_client::solana_sdk::transaction::Transaction;
use anchor_client::solana_sdk::{
    compute_budget::ComputeBudgetInstruction, instruction::Instruction, pubkey::Pubkey,
    signature::Signature, signer::Signer,
};
// use anchor_spl::token;
use anchor_spl::token_2022;
use anyhow::{Ok, Result};
use locker::CreateVestingEscrowParameters;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::str::FromStr;

/// Represents a single entry in a CSV
#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
struct CsvEntry2 {
    /// token mint
    pub token_mint: String,
    /// address
    pub address: String,
    /// cliff_unlock_amount
    pub cliff_unlock_amount: u64,
    /// amount_per_period
    pub amount_per_period: u64,
}

pub fn process_initialize_lock_escrow_from_file2(
    args: &Args,
    sub_args: &InitializeLockEscrowFromFileArgs,
) {
    println!("initialize lock escrow from file2: {sub_args:#?}");
    create_lock_escrow2(args, sub_args).unwrap();
}

fn create_lock_escrow2(args: &Args, sub_args: &InitializeLockEscrowFromFileArgs) -> Result<()> {
    let &InitializeLockEscrowFromFileArgs {
        wallet_path: _,
        token_mint,
        vesting_start_time,
        cliff_time,
        frequency,
        number_of_period,
        update_recipient_mode,
        cancel_mode,
    } = sub_args;
    let file = File::open(sub_args.wallet_path.clone())?;
    let mut rdr = csv::Reader::from_reader(file);

    let mut entries = Vec::new();
    for result in rdr.deserialize() {
        let record: CsvEntry2 = result.unwrap();
        entries.push(record);
    }

    // save signature back to csv file
    for entry in entries.iter() {
        let signature = create_lock_escrow_for_an_user2(
            args,
            &LockEscrowForAnUserParam2{
                wallet: Pubkey::from_str(&entry.address).unwrap(), // panic for invalid wallet
                token_mint: Pubkey::from_str(&entry.token_mint).unwrap(),
                vesting_start_time,
                cliff_time,
                frequency,
                cliff_unlock_amount: entry.cliff_unlock_amount,
                amount_per_period: entry.amount_per_period,
                number_of_period,
                update_recipient_mode,
                cancel_mode,
            },
        )?;
        println!(
            "successfully create vesting escrow for address {:?} with signature {signature:#?}",
            entry.address
        );
    }

    Ok(())
}

pub struct LockEscrowForAnUserParam2 {
    pub wallet: Pubkey,
    pub token_mint: Pubkey,
    pub vesting_start_time: u64,
    pub cliff_time: u64,
    pub frequency: u64,
    pub cliff_unlock_amount: u64,
    pub amount_per_period: u64,
    pub number_of_period: u64,
    pub update_recipient_mode: u8,
    pub cancel_mode: u8,
}

fn create_lock_escrow_for_an_user2(
    args: &Args,
    sub_args: &LockEscrowForAnUserParam2,
) -> Result<Signature> {
    let &LockEscrowForAnUserParam2 {
        wallet,
        token_mint,
        vesting_start_time,
        cliff_time,
        frequency,
        cliff_unlock_amount,
        amount_per_period,
        number_of_period,
        update_recipient_mode,
        cancel_mode,
    } = sub_args;
    let client =
        RpcClient::new_with_commitment(args.rpc_url.clone(), CommitmentConfig::finalized());
    let keypair = read_keypair_file(&args.keypair_path.clone().unwrap()).unwrap();
    let mut ixs = vec![];
    // check priority fee
    if let Some(priority_fee) = args.priority_fee {
        ixs.push(ComputeBudgetInstruction::set_compute_unit_price(
            priority_fee,
        ));
    }

    let base_kp: Keypair = Keypair::new();
    let (escrow, _bump) = Pubkey::find_program_address(
        &[b"escrow".as_ref(), base_kp.pubkey().as_ref()],
        &locker::ID,
    );

    ixs.push(
        spl_associated_token_account::instruction::create_associated_token_account_idempotent(
            &keypair.pubkey(),
            &escrow,
            &token_mint,
            &token_2022::ID,
        ),
    );

    let (event_authority, _bump) =
        Pubkey::find_program_address(&[b"__event_authority"], &locker::ID);
    ixs.push(Instruction {
        program_id: locker::ID,
        accounts: locker::accounts::CreateVestingEscrow2Ctx {
            base: base_kp.pubkey(),
            escrow,
            token_mint, //TESTING UNSURE IF THIS IS CORRECT
            escrow_token: spl_associated_token_account::get_associated_token_address_with_program_id(   //TESTING UNSURE IF THIS IS CORRECT
                &escrow,
                &token_mint,
                &token_2022::ID
            ),
            sender: keypair.pubkey(),
            sender_token: spl_associated_token_account::get_associated_token_address_with_program_id(    //TESTING UNSURE IF THIS IS CORRECT
                &keypair.pubkey(),
                &token_mint,
                &token_2022::ID
            ),
            recipient: wallet,
            token_program: token_2022::ID,
            system_program: anchor_lang::solana_program::system_program::id(),
            event_authority,
            program: locker::ID,
        }
            .to_account_metas(None),
        data: locker::instruction::CreateVestingEscrowV2 {
            params: CreateVestingEscrowParameters {
                vesting_start_time,
                cliff_time,
                frequency,
                cliff_unlock_amount,
                amount_per_period,
                number_of_period,
                update_recipient_mode,
                cancel_mode,
            },
            remaining_accounts_info: None,
        }
            .data(),
    });

    let blockhash = client.get_latest_blockhash().unwrap();
    let tx = Transaction::new_signed_with_payer(
        &ixs,
        Some(&keypair.pubkey()),
        &[&keypair, &base_kp],
        blockhash,
    );

    let signature = client
        .send_and_confirm_transaction_with_spinner(&tx)
        .unwrap();

    println!(
        "Created lock escrow: {:?} for token mint: {:?}",
            &escrow.to_string(),
            &token_mint.to_string()
    );

    Ok(signature)
}
