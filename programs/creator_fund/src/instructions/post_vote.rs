use anchor_lang::prelude::*;

use crate::states::CreatorWallet;
use crate::{
    constants::VOTE_SEED, 
    errors::AppError, 
    states::{Post, Vote, VoteType}
};


pub fn vote_on_post(ctx: Context<VoteOnPost>, vote_type: VoteType) -> Result<()> {
    let vote = &mut ctx.accounts.vote;
    vote.voter = ctx.accounts.voter.key();
    vote.post = ctx.accounts.post.key();
    vote.vote_type = vote_type.clone();
    vote.bump = ctx.bumps.vote;

    // Update post vote counts
    let post_account = &mut ctx.accounts.post;
    match vote_type {
        VoteType::UpVote => {
            post_account.up_votes = post_account.up_votes
                .checked_add(1)
                .ok_or(AppError::VoteOverflow)?;
        }
        VoteType::DownVote => {
            post_account.down_votes = post_account.down_votes
                .checked_add(1)
                .ok_or(AppError::VoteOverflow)?;
        }
    }

    Ok(())
}

#[derive(Accounts)]
pub struct VoteOnPost<'info> {
    #[account(mut)]
    pub voter: Signer<'info>,
    #[account(
        init,
        payer = voter,
        space = 8 + Vote::INIT_SPACE,
        seeds = [VOTE_SEED.as_bytes(), voter.key().as_ref(), post.key().as_ref()],
        bump,
    )]
    pub vote: Account<'info, Vote>,
    #[account(mut)]
    pub post: Account<'info, Post>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClaimCreatorReward<'info> {
    #[account(mut)]
    pub post: Account<'info, Post>,

    /// The creator claiming the reward (must sign)
    #[account(mut)]
    pub creator: Signer<'info>,

    /// CHECK: The fund token account (tokens transferred from here, validated manually)
    #[account(mut)]
    pub fund_token_account: UncheckedAccount<'info>,

    /// The creator's wallet state account (validates the vault exists)
    #[account(
        seeds = [b"state", creator.key().as_ref()],
        bump = creator_wallet.state_bump,
    )]
    pub creator_wallet: Account<'info, CreatorWallet>,

    /// CHECK: The creator's vault token account (receives the reward tokens, validated manually)
    #[account(mut)]
    pub creator_vault_token_account: UncheckedAccount<'info>,

    /// The vault authority PDA that can sign for the vault token account
    /// CHECK: This is the vault authority PDA
    #[account(
        seeds = [b"vault", creator_wallet.key().as_ref()],
        bump = creator_wallet.wallet_bump,
    )]
    pub vault_authority: AccountInfo<'info>,

    /// The SPL Token program (required for token transfers)
    pub token_program: Program<'info, anchor_spl::token::Token>,
}

pub fn claim_creator_reward(_ctx: Context<ClaimCreatorReward>) -> Result<()> {
    // TODO: Implementation will be added when instructed
    // This will allow the creator to claim their reward when target upvotes are reached
    Ok(())
}
