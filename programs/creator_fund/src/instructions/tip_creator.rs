use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenInterface, TokenAccount,
    TransferChecked,
};

use crate::states::Post;
use crate::errors::AppError;

pub fn tip_creator_instruction(ctx: Context<TipCreator>, amount: u64) -> Result<()> {
    // Validate token accounts
    let from_token_account = TokenAccount::try_deserialize(
        &mut &ctx.accounts.from.data.borrow()[..]
    )?;
    let to_token_account = TokenAccount::try_deserialize(
        &mut &ctx.accounts.to.data.borrow()[..]
    )?;

    // Verify from account belongs to authority
    require!(
        from_token_account.owner == ctx.accounts.authority.key(),
        anchor_lang::error::ErrorCode::ConstraintOwner
    );

    // Verify mint matches for both token accounts
    let mint_key = ctx.accounts.mint.key();
    require!(
        from_token_account.mint == mint_key,
        anchor_lang::error::ErrorCode::ConstraintTokenMint
    );
    require!(
        to_token_account.mint == mint_key,
        anchor_lang::error::ErrorCode::ConstraintTokenMint
    );

    // Verify creator (owner of "to" token account) has at least 1 post
    let creator = to_token_account.owner;
    let post = &ctx.accounts.creator_post;
    require!(
        post.author == creator,
        AppError::CreatorHasNoPosts
    );

    // Perform the transfer using the helper function
    TipCreator::tip_creator(
        &ctx.accounts.from.to_account_info(),
        &ctx.accounts.to.to_account_info(),
        amount,
        &ctx.accounts.mint.to_account_info(),
        &ctx.accounts.authority.to_account_info(),
        &ctx.accounts.token_program,
        None,
    )
}

#[derive(Accounts)]
pub struct TipCreator<'info> {
    /// CHECK: Token account to transfer from (validated manually)
    #[account(mut)]
    pub from: UncheckedAccount<'info>,
    /// CHECK: Token account to transfer to (validated manually)
    #[account(mut)]
    pub to: UncheckedAccount<'info>,
    /// CHECK: Token mint (validated manually to get decimals)
    pub mint: UncheckedAccount<'info>,
    pub authority: Signer<'info>,
    /// A post by the creator (proves creator has at least 1 post)
    /// The creator is identified as the owner of the "to" token account
    pub creator_post: Account<'info, Post>,
    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> TipCreator<'info> {
    pub fn tip_creator(
        from: &AccountInfo<'info>, // token account to transfer from
        to: &AccountInfo<'info>, // token account to transfer to
        amount: u64, // amount of tokens to transfer
        mint: &AccountInfo<'info>, // mint of the tokens to transfer
        authority: &AccountInfo<'info>, // authority of the tokens to transfer
        token_program: &Interface<'info, TokenInterface>, // token program to use for the transfer.
        owning_pda_seeds: Option<&[&[u8]]>,
    ) -> Result<()> {
        let mint_data = Mint::try_deserialize(&mut &mint.data.borrow()[..])?;
        
        let transfer_accounts = TransferChecked {
            from: from.to_account_info(),
            mint: mint.to_account_info(),
            to: to.to_account_info(),
            authority: authority.to_account_info(),
        };
    
        let signers_seeds = owning_pda_seeds.map(|seeds| [seeds]);
    
        transfer_checked(
            if let Some(seeds_arr) = signers_seeds.as_ref() {
                CpiContext::new_with_signer(
                    token_program.to_account_info(),
                    transfer_accounts,
                    seeds_arr,
                )
            } else {
                // used when user's keypair is already a signer of the transaction
                CpiContext::new(token_program.to_account_info(), transfer_accounts)
            },
            amount,
            mint_data.decimals,
        )
    }    
}
