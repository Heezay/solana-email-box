use anchor_lang::{prelude::*, InstructionData};
use solana_program::instruction::Instruction;

const DIRECT_CHAT_SIZE: usize = 1 + 32;
const MESSAGE_MAX_SIZE: usize = 32 + 1 + 32 + 4 + MAX_STRING_BYTES;
const MAX_STRING_BYTES: usize = 255;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod solana_email_box {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(message_seed: Vec<u8>)]
pub struct SendDirectMessage<'info> {
    #[account(mut)]
    pub from: Signer<'info>,
    pub to: AccountInfo<'info>,
    #[account(
    init_if_needed,
    payer = from,
    owner = *program_id,
    seeds = [to.key().as_ref()],
    bump,
    space = 8 + DIRECT_CHAT_SIZE
  )]
    pub mailbox: Account<'info, Mailbox>,
    #[account(
    init_if_needed,
    payer = from,
    owner = *program_id,
    seeds = [message_seed.as_ref()],
    bump,
    space = 8 + MESSAGE_MAX_SIZE
  )]
    pub message: Account<'info, Message>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Mailbox {
    pub inbox: Option<Pubkey>,
}

#[account]
pub struct Message {
    pub from: Pubkey,
    pub inbox: Option<Pubkey>,
    pub ciphertext: Vec<u8>,
}
