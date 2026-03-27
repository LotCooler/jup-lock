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
use anchor_spl::memo;
use anyhow::{Ok, Result};
use locker::{handle_cancel_vesting_escrow, handle_close_vesting_escrow, CancelVestingEscrowCtx, CloseVestingEscrowCtx, CreateVestingEscrowParameters};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::str::FromStr;
use anchor_lang::context::Context;
use locker::util::RemainingAccountsInfo;

/// Represents a single entry in a CSV
#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
struct CsvEntry3 {
    /// unknown what we need yet, just making placeholder\
    pub address: String,
    pub escrow: String,
    pub token_mint: String,
    pub max_amount: u64,
}

pub fn process_initialize_claim_escrow_from_file2(
    args: &Args,
    sub_args: &InitializeClaimEscrowFromFile2Args,
) {
    println!("initialize claim escrow from file2: {sub_args:#?}");
    crate::instructions::process_initialize_claim_escrow_from_file2::claim_escrow2(args, sub_args).unwrap();
}

fn claim_escrow2(args: &Args, sub_args: &InitializeClaimEscrowFromFile2Args) -> Result<()> {
    let &InitializeClaimEscrowFromFile2Args {
        wallet_path: _,
    } = sub_args;
    let file = File::open(sub_args.wallet_path.clone())?;
    let mut rdr = csv::Reader::from_reader(file);

    let mut entries = Vec::new();
    for result in rdr.deserialize() {
        let record: crate::instructions::process_initialize_claim_escrow_from_file2::CsvEntry3 = result.unwrap();
        entries.push(record);
    }

    // save signature back to csv file
    for entry in entries.iter() {
        let signature = crate::instructions::process_initialize_claim_escrow_from_file2::claim_escrow_for_an_user2(
            args,
            &ClaimEscrowForAnUserParam2{
                wallet: Pubkey::from_str(&entry.address).unwrap(), // panic for invalid wallet
                escrow: Pubkey::from_str(&entry.escrow).unwrap(),
                token_mint: Pubkey::from_str(&entry.token_mint).unwrap(),
                max_amount: entry.max_amount,
            },
        )?;
        println!(
            "successfully claim vesting escrow for escrow {:?} with signature {signature:#?}",
            entry.escrow
        );
    }

    Ok(())
}

pub struct ClaimEscrowForAnUserParam2 {
    pub wallet: Pubkey,
    pub escrow: Pubkey,
    pub token_mint: Pubkey,
    pub max_amount: u64,
}

fn claim_escrow_for_an_user2(
    args: &Args,
    sub_args: &ClaimEscrowForAnUserParam2,
) -> Result<Signature> {
    let &ClaimEscrowForAnUserParam2 {
        wallet,
        escrow,
        token_mint,
        max_amount,
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

    // Need to call this function of the main contract (locker)
    //    pub fn claim_v2<'c: 'info, 'info>(
    //         ctx: Context<'_, '_, 'c, 'info, Claim2Ctx<'info>>,
    //         max_amount: u64,
    //         remaining_accounts_info: Option<RemainingAccountsInfo>,
    //     ) -> Result<()> {handle_claim2(ctx, max_amount, remaining_accounts_info) }

    // After testing, add this function to get all our rent back
    //    pub fn close_vesting_escrow<'c: 'info, 'info>(
    //        ctx: Context<'_, '_, 'c, 'info, CloseVestingEscrowCtx<'info>>,
    //        remaining_accounts_info: Option<RemainingAccountsInfo>,
    //    ) -> anchor_lang::Result<()> {handle_close_vesting_escrow(ctx, remaining_accounts_info)}

    let (event_authority, _bump) =
        Pubkey::find_program_address(&[b"__event_authority"], &locker::ID);
    ixs.push(Instruction {
        program_id: locker::ID,
        accounts: locker::accounts::Claim2Ctx {
            escrow: sub_args.escrow,
            token_mint: sub_args.token_mint,
            escrow_token: spl_associated_token_account::get_associated_token_address_with_program_id(   //TESTING UNSURE IF THIS IS CORRECT
                                                                 &escrow,
                                                                 &token_mint,
                                                                 &token_2022::ID),
            recipient: wallet,
            recipient_token: spl_associated_token_account::get_associated_token_address_with_program_id(    //TESTING UNSURE IF THIS IS CORRECT
                                                                 &keypair.pubkey(),
                                                                 &token_mint,
                                                                 &token_2022::ID),
            memo_program: memo::ID,
            token_program: token_2022::ID,
            event_authority,
            program: locker::ID,
        }
            .to_account_metas(None),
        data: locker::instruction::ClaimV2 {
            max_amount: sub_args.max_amount,
            remaining_accounts_info: None,
        }
            .data(),
    });

    let blockhash = client.get_latest_blockhash().unwrap();
    let tx = Transaction::new_signed_with_payer(
        &ixs,
        Some(&keypair.pubkey()),
        &[&keypair],
        blockhash,
    );

    let signature = client
        .send_and_confirm_transaction_with_spinner(&tx)
        .unwrap();

    Ok(signature)
}