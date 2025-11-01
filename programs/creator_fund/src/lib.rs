use anchor_lang::prelude::*;

pub mod states;
pub mod instructions;
pub mod errors;
pub mod constants;

use crate::instructions::*;
use crate::states::VoteType;

declare_id!("5fHuxe7VZB3APbJ5AV4jbcFea2gTBHVS8VVQfSu42jdS");

#[program]
pub mod creator_fund {
    use super::*;

    pub fn create_post(ctx: Context<CreatePost>, title: String, content: String) -> Result<()> {
        instructions::create_post(ctx, title, content)?;
        Ok(())
    }

    pub fn vote_on_post(ctx: Context<VoteOnPost>, vote_type: VoteType) -> Result<()> {
        instructions::vote_on_post(ctx, vote_type)?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
