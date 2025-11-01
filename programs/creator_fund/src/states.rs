use anchor_lang::prelude::*;
use crate::constants::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub enum VoteType {
    UpVote,
    DownVote,
}

#[account]
#[derive(InitSpace)]
pub struct Post {
    #[max_len(POST_TITLE_MAX_LEN)]
    pub title: String,
    #[max_len(POST_CONTENT_MAX_LEN)]
    pub content: String,
    pub author: Pubkey,
    pub community: Pubkey,
    pub up_votes: u64,
    pub down_votes: u64,
    pub created_at: u64,
    pub rewarded: bool,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Vote {
    pub voter: Pubkey,
    pub post: Pubkey,
    pub vote_type: VoteType,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct CreatorWallet {
    /// The bump seed for the vault authority PDA
    /// This is used to sign transactions on behalf of the vault
    pub wallet_bump: u8,

    /// The bump seed for the state account PDA
    /// This is used for validation when accessing the state account
    pub state_bump: u8,

    /// The mint address of the token type stored in this vault
    /// This ensures all operations are performed on the correct token type
    pub mint: Pubkey,

    /// The address of the vault's token account
    /// This is where the actual tokens are stored
    pub vault_token_account: Pubkey,
}
