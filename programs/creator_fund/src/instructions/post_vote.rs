use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Transfer as TokenTransfer};

use crate::states::CreatorWallet;
use crate::{
    constants::{VOTE_SEED, TARGET_NUMBER_OF_UPVOTES, CREATOR_FUND_REWARD}, 
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
    
    /// The authority for the fund token account (must sign to authorize transfer)
    /// This is typically the deployer who owns the fund
    pub fund_authority: Signer<'info>,

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

pub fn claim_creator_reward(ctx: Context<ClaimCreatorReward>) -> Result<()> {
    let post_account = &mut ctx.accounts.post;
    
    // Verify threshold is met
    require!(
        post_account.up_votes >= TARGET_NUMBER_OF_UPVOTES,
        AppError::InvalidCreator
    );
    
    // Verify creator matches post author
    require!(
        ctx.accounts.creator.key() == post_account.author,
        AppError::InvalidCreator
    );

    // Verify not already rewarded
    require!(
        !post_account.rewarded,
        AppError::InvalidCreator
    );
    
    // Validate token accounts
    let fund_token_account_data = TokenAccount::try_deserialize(
        &mut &ctx.accounts.fund_token_account.data.borrow()[..]
    )?;
    let creator_vault_account_data = TokenAccount::try_deserialize(
        &mut &ctx.accounts.creator_vault_token_account.data.borrow()[..]
    )?;
    
    // Verify fund token account authority matches provided authority
    require!(
        fund_token_account_data.owner == ctx.accounts.fund_authority.key(),
        AppError::InvalidCreator
    );
    
    // Verify vault token account matches creator wallet state
    require!(
        ctx.accounts.creator_vault_token_account.key() == ctx.accounts.creator_wallet.vault_token_account,
        AppError::InvalidCreator
    );
    
    // Verify mint matches for both accounts
    require!(
        fund_token_account_data.mint == ctx.accounts.creator_wallet.mint,
        AppError::InvalidCreator
    );
    require!(
        creator_vault_account_data.mint == ctx.accounts.creator_wallet.mint,
        AppError::InvalidCreator
    );
    
    // Transfer tokens from fund to creator's vault
    let reward_amount = CREATOR_FUND_REWARD;
    let cpi_accounts = TokenTransfer {
        from: ctx.accounts.fund_token_account.to_account_info(),
        to: ctx.accounts.creator_vault_token_account.to_account_info(),
        authority: ctx.accounts.fund_authority.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    
    token::transfer(cpi_ctx, reward_amount)?;
    
    // Mark post as rewarded
    post_account.rewarded = true;
    
    Ok(())
}
