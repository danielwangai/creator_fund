use anchor_lang::error_code;

#[error_code]
pub enum AppError {
    PostTitleRequired,
    #[msg("Post title is too long")]
    PostTitleTooLong,
    #[msg("Post content is required")]
    PostContentRequired,
    #[msg("Post content is too long")]
    PostContentTooLong,
    #[msg("Already voted on this post")]
    AlreadyVoted,
    #[msg("Vote count overflow")]
    VoteOverflow,
}
