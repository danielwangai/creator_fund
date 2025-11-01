use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer as TokenTransfer};

use crate::states::CreatorWallet;
use crate::{
    constants::{VOTE_SEED, TARGET_NUMBER_OF_UPVOTES, CREATOR_FUND_REWARD}, 
    errors::AppError, 
    states::{Post, Vote, VoteType}
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

    // Reward creator if they exceed the target number of upvotes and haven't been rewarded yet
    if post_account.up_votes >= TARGET_NUMBER_OF_UPVOTES && !post_account.rewarded {
        // Verify creator matches post author
        require!(
            ctx.accounts.creator.key() == post_account.author,
            AppError::InvalidCreator
        );

        // Validate and deserialize token accounts
        let fund_token_account_data = TokenAccount::try_deserialize(&mut &ctx.accounts.fund_token_account.data.borrow()[..])?;
        let creator_vault_account_data = TokenAccount::try_deserialize(&mut &ctx.accounts.creator_vault_token_account.data.borrow()[..])?;

        // Verify fund token account belongs to deployer
        require!(
            fund_token_account_data.owner == ctx.accounts.deployer.key(),
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

        // Calculate reward amount (CREATOR_FUND_REWARD is in base units)
        let reward_amount = CREATOR_FUND_REWARD;

        // Transfer tokens from deployer's fund to creator's vault
        let cpi_accounts = TokenTransfer {
            from: ctx.accounts.fund_token_account.to_account_info(),
            to: ctx.accounts.creator_vault_token_account.to_account_info(),
            authority: ctx.accounts.deployer.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        
        token::transfer(cpi_ctx, reward_amount)?;

        // Mark post as rewarded
        post_account.rewarded = true;
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
    
    /// CHECK: Creator of the post (validated when reward threshold is met)
    pub creator: AccountInfo<'info>,
    
    /// CHECK: Deployer who remits the reward (signer when reward threshold is met)
    pub deployer: Signer<'info>,
    
    /// CHECK: Fund token account owned by deployer (validated manually when reward threshold is met)
    #[account(mut)]
    pub fund_token_account: UncheckedAccount<'info>,
    
    /// CHECK: Creator's wallet state account (validated when reward threshold is met)
    #[account(
        seeds = [b"state", creator.key().as_ref()],
        bump = creator_wallet.state_bump,
    )]
    pub creator_wallet: Account<'info, CreatorWallet>,
    
    /// CHECK: Creator's vault token account (validated manually when reward threshold is met)
    #[account(mut)]
    pub creator_vault_token_account: UncheckedAccount<'info>,
    
    /// CHECK: Vault authority PDA (required when reward threshold is met)
    #[account(
        seeds = [b"vault", creator_wallet.key().as_ref()],
        bump = creator_wallet.wallet_bump,
    )]
    pub vault_authority: AccountInfo<'info>,
    
    /// CHECK: Token program (required when reward threshold is met)
    pub token_program: Program<'info, Token>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PayCreator<'info> {
    #[account(mut)]
    pub post: Account<'info, Post>,

    /// The creator receiving the reward
    /// CHECK: Verified via constraint that creator matches post.author
    #[account(
        constraint = creator.key() == post.author @ AppError::InvalidCreator
    )]
    pub creator: AccountInfo<'info>,

    /// The deployer who remits the reward to the creator (must sign)
    #[account(mut)]
    pub deployer: Signer<'info>,

    /// CHECK: The fund token account owned by deployer (tokens transferred from here, validated manually)
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
    pub token_program: Program<'info, Token>,
}

impl<'info> PayCreator<'info> {
    pub fn pay_creator(ctx: Context<PayCreator>, amount: u64) -> Result<()> {
        // Transfer tokens from deployer's fund to creator's vault
        let cpi_accounts = TokenTransfer {
            from: ctx.accounts.fund_token_account.to_account_info(),
            to: ctx.accounts.creator_vault_token_account.to_account_info(),
            authority: ctx.accounts.deployer.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        
        token::transfer(cpi_ctx, amount)?;

        Ok(())
    }
}
