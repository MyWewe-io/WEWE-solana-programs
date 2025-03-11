pub mod state;
pub mod instructions;
use anchor_lang::prelude::*;

declare_id!("DtwC3LsBgwnp6Cuc6MExnijmwh7WLXS5Hdr7XpdyF1qZ");

#[program]
pub mod wewe_token_launch_pad {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
