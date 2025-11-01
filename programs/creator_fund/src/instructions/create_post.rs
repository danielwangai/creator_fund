use anchor_lang::prelude::*;
use anchor_lang::solana_program::hash::hash;

use crate::{
    constants::{POST_SEED, POST_TITLE_MAX_LEN, POST_CONTENT_MAX_LEN},
    states::{ Post},
    errors::AppError
};

pub fn create_post(ctx: Context<CreatePost>, title: String, content: String) -> Result<()> {
    if title == "" {
        return Err(AppError::PostTitleRequired.into());
    }
    if title.chars().count() > POST_TITLE_MAX_LEN {
        return Err(AppError::PostTitleTooLong.into());
    }
    if content == "" {
        return Err(AppError::PostContentRequired.into());
    }
    if content.chars().count() > POST_CONTENT_MAX_LEN {
        return Err(AppError::PostContentTooLong.into());
    }

    let post: &mut Account<Post> = &mut ctx.accounts.post;
    post.title = title;
    post.content = content;
    post.author = ctx.accounts.author.key();
    post.up_votes = 0;
    post.down_votes = 0;
    post.created_at = Clock::get()?.unix_timestamp as u64;
    post.rewarded = false;
    post.bump = ctx.bumps.post;

    Ok(())
}

#[derive(Accounts)]
#[instruction(title: String)]
pub struct CreatePost<'info> {
    #[account(mut)]
    pub author: Signer<'info>,
    #[account(
        init,
        payer = author,
        space = 8 + Post::INIT_SPACE,
        seeds = [
            POST_SEED.as_bytes(),
            {hash(title.as_bytes()).to_bytes().as_ref()},
            author.key().as_ref(),
        ],
        bump,
    )]
    pub post: Account<'info, Post>,
    pub system_program: Program<'info, System>,
}
