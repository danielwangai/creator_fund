use anchor_lang::prelude::*;

use crate::{
    constants::VOTE_SEED, errors::AppError, states::{Post, Vote, VoteType}
};


pub fn vote_on_post(ctx: Context<VoteOnPost>, vote_type: VoteType) -> Result<()> {
    // Initialize the vote account
    // The init constraint ensures the account doesn't exist (hasn't been voted before)
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
        constraint = vote.to_account_info().lamports() == 0 @ AppError::AlreadyVoted,
    )]
    pub vote: Account<'info, Vote>,
    #[account(mut)]
    pub post: Account<'info, Post>,
    pub system_program: Program<'info, System>,
}