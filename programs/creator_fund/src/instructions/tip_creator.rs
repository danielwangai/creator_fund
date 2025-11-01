use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenInterface,
    TransferChecked,
};

pub fn tip_creator<'info>(
    from: &AccountInfo<'info>, // token account to transfer from
    to: &AccountInfo<'info>, // token account to transfer to
    amount: u64, // amount of tokens to transfer
    mint: &AccountInfo<'info>, // mint of the tokens to transfer
    authority: &AccountInfo<'info>, // authority of the tokens to transfer
    token_program: &Interface<'info, TokenInterface>, // token program to use for the transfer.
    owning_pda_seeds: Option<&[&[u8]]>,
) -> Result<()> {
    // Deserialize mint to get decimals
    let mint_data = Mint::try_deserialize(&mut &mint.data.borrow()[..])?;
    
    let transfer_accounts = TransferChecked {
        from: from.to_account_info(),
        mint: mint.to_account_info(),
        to: to.to_account_info(),
        authority: authority.to_account_info(),
    };

    let signers_seeds = owning_pda_seeds.map(|seeds| [seeds]);

    // Do the transfer, by calling transfer_checked - providing a different CPI context
    // depending on whether we're sending tokens from a PDA or not
    transfer_checked(
        if let Some(seeds_arr) = signers_seeds.as_ref() {
            // used when the PDA is the authority doing the transfer
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

pub fn tip_creator_instruction(ctx: Context<TipCreator>, amount: u64) -> Result<()> {
    tip_creator(
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
    pub token_program: Interface<'info, TokenInterface>,
}
