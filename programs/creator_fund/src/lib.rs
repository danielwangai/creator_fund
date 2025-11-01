use anchor_lang::prelude::*;

declare_id!("5fHuxe7VZB3APbJ5AV4jbcFea2gTBHVS8VVQfSu42jdS");

#[program]
pub mod creator_fund {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
