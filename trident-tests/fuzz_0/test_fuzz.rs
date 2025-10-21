use fuzz_accounts::*;
use trident_fuzz::fuzzing::*;
mod fuzz_accounts;
mod instructions;
mod transactions;
mod types;
pub use transactions::*;
use anyhow::Context;
use spl_token::state::Account as SplTokenAccount;
use types::Proposal;
use borsh::BorshDeserialize;

#[derive(FuzzTestMethods)]
struct FuzzTest {
    /// for fuzzing
    trident: Trident,
    /// for storing fuzzing accounts
    fuzz_accounts: FuzzAccounts,
}

#[flow_executor]
impl FuzzTest {
    fn new() -> Self {
        Self {
            trident: Trident::default(),
            fuzz_accounts: FuzzAccounts::default(),
        }
    }

    #[init]
    fn start(&mut self) {
        // perform any initialization here, this method will be executed
        // at start of each iteration
    }

    #[flow]
    fn flow1(&mut self) {
        // perform logic which is meant to be fuzzed
        // this flow is selected randomly from other flows
    }

    #[flow]
    fn flow2(&mut self) {
        // perform logic which is meant to be fuzzed
        // this flow is selected randomly from other flows
    }

    #[end]
    fn end(&mut self) -> anyhow::Result<()> {
        // helper: read SPL token amount
        let read_token_amount = |pk: trident_fuzz::fuzzing::TridentPubkey| -> anyhow::Result<u64> {
            let data = self.trident.rpc()
                .get_account_data(&pk.into())
                .context("read ATA")?;
            let ata = SplTokenAccount::unpack(&data).context("unpack ATA")?;
            Ok(ata.amount)
        };
    
        // helper: read Proposal (skip 8-byte Anchor discriminator)
        let read_proposal = |pk: trident_fuzz::fuzzing::TridentPubkey| -> anyhow::Result<types::Proposal> {
            let data = self.trident.rpc()
                .get_account_data(&pk.into())
                .context("read proposal")?;
            types::Proposal::try_from_slice(&data[8..]).context("decode proposal")
        };
    
        // 1) Soulbound: backer token amount ≤ 1
        if let Ok(bal) = read_token_amount(self.fuzz_accounts.backer_token_account) {
            anyhow::ensure!(bal <= 1, "soulbound > 1");
        }
    
        // 2) Proposal cannot be both launched and rejected
        if let Ok(p) = read_proposal(self.fuzz_accounts.proposal) {
            anyhow::ensure!(!(p.is_pool_launched && p.is_rejected), "rejected proposal launched");
        }
    
        Ok(())
    }
}

fn main() {
    FuzzTest::fuzz(1000, 100);
}
