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
    pub bump: u8,
}
