use {
    crate::state::backers::Backers,
    anchor_lang::prelude::*,
};

#[derive(Accounts)]
pub struct UpdateBacker<'info> {
    pub authority: Signer<'info>,
    #[account(mut)]
    pub backer_account: Account<'info, Backers>,
    pub system_program: Program<'info, System>,
}

impl<'info> UpdateBacker<'info> {
    pub fn update_airdrop_amount(&mut self, amount: u64) -> Result<()> {
        self.backer_account.claim_amount += amount;     

        Ok(())
    }
}
