use anchor_lang::prelude::*;
use std::collections::HashMap;


declare_id!("35XcNkLUyBHetw7CVZL6Gg9R1AhZgzS1ektLy1Yeiv8e");

#[program]
mod centralized_connection {
    use super::*;

    pub struct CentralizedConnection {
        pub message_fees: HashMap<String, u64>,
        pub response_fees: HashMap<String, u64>,
        pub receipts: HashMap<String, HashMap<u64, bool>>,
        pub x_call: Pubkey,
        pub admin_address: Pubkey,
        pub conn_sn: u64,
    }

    impl CentralizedConnection {

        pub fn initialize(
            &mut self,
            ctx: Context<Initialize>,
            relayer: Pubkey,
            x_call: Pubkey,
        ) -> Result<()> {
            self.x_call = x_call;
            self.admin_address = relayer;
            Ok(())
        }

        pub fn set_fee(
            &mut self,
            ctx: Context<SetFee>,
            network_id: String,
            message_fee: u64,
            response_fee: u64,
        ) -> Result<()> {
            self.message_fees.insert(network_id.clone(), message_fee);
            self.response_fees.insert(network_id, response_fee);
            Ok(())
        }

        pub fn get_fee(
            &self,
            ctx: Context<GetFee>,
            to: String,
            response: bool,
        ) -> Result<()> {
            let message_fee = self.message_fees.get(&to).ok_or(ErrorCode::MessageNotFound)?;
            let response_fee = if response {
                *self.response_fees.get(&to).unwrap_or(&0)
            } else {
                0
            };
            let fee = message_fee + response_fee;
            msg!("Fee for {}: {}", to, fee);
            Ok(())
        }

        pub fn send_message(
            &mut self,
            ctx: Context<SendMessage>,
            to: String,
            svc: String,
            sn: i64,
            _msg: Vec<u8>,
        ) -> Result<()> {
            if ctx.accounts.x_call.key() != &self.x_call {
                return Err(ErrorCode::OnlyXcallCanCallSendMessage.into());
            }
            let fee = if sn > 0 {
                self.get_fee(ctx.accounts.to.clone(), to, true)?;
            } else if sn == 0 {
                self.get_fee(ctx.accounts.to.clone(), to, false)?;
            } else {
                return Err(ErrorCode::InvalidSn.into());
            };
            let fee = fee as u64;
            if ctx.accounts.system_program.remaining_accounts == 0 {
                let ix = anchor_lang::solana_program::system_instruction::transfer(
                    &ctx.accounts.to.key(),
                    &ctx.accounts.system_program.key(),
                    fee,
                );
                anchor_lang::solana_program::program::invoke(&ix, &[ctx.accounts.to.clone(), ctx.accounts.system_program.clone()])?;
            }
            self.conn_sn += 1;
            emit!(Message {
                target_network: ctx.accounts.to.clone(),
                sn: self.conn_sn,
                msg: _msg,
            });
            Ok(())
        }

        pub fn recv_message(
            &mut self,
            ctx: Context<RecvMessage>,
            src_network: String,
            conn_sn: u64,
            _msg: Vec<u8>,
        ) -> Result<()> {
            if let Some(receipts) = self.receipts.get_mut(&src_network) {
                if receipts.contains_key(&conn_sn) {
                    return Err(ErrorCode::DuplicateMessage.into());
                }
                receipts.insert(conn_sn, true);
            }
            let cpi_ctx = CpiContext::new(self.x_call.clone(), crate::handlers::HandleMessage {
                src_network,
                msg: _msg,
            });
            ICallService::handle_message(&mut cpi_ctx)?;
            Ok(())
        }

        pub fn claim_fees(&mut self, ctx: Context<ClaimFees>) -> Result<()> {
            if ctx.accounts.admin_address.key() != &self.admin_address {
                return Err(ErrorCode::OnlyAdmin.into());
            }
            let ix = anchor_lang::solana_program::system_instruction::transfer(
                ctx.accounts.admin_address.key(),
                ctx.accounts.to.key(),
                ctx.accounts.to.lamports(),
            );
            anchor_lang::solana_program::program::invoke(&ix, &[ctx.accounts.to.clone(), ctx.accounts.admin_address.clone()])?;
            Ok(())
        }

        pub fn revert_message(&mut self, ctx: Context<RevertMessage>, sn: u64) -> Result<()> {
            if ctx.accounts.x_call.key() != &self.x_call {
                return Err(ErrorCode::OnlyXcallCanRevertMessage.into());
            }
            let cpi_ctx = CpiContext::new(self.x_call.clone(), crate::handlers::HandleError { sn });
            ICallService::handle_error(cpi_ctx)?;
            Ok(())
        }

        pub fn get_receipt(
            &self,
            ctx: Context<GetReceipt>,
            src_network: String,
            conn_sn: u64,
        ) -> Result<()> {
            let receipts = self.receipts.get(&src_network).unwrap_or(&HashMap::new());
            let received = receipts.get(&conn_sn).unwrap_or(&false);
            msg!("Receipt for message {} from {}: {}", conn_sn, src_network, received);
            Ok(())
        }

        pub fn set_admin(
            &mut self,
            ctx: Context<SetAdmin>,
            new_admin: Pubkey,
        ) -> Result<()> {
            if ctx.accounts.admin_address.key() != &self.admin_address {
                return Err(ErrorCode::OnlyAdmin.into());
            }
            self.admin_address = new_admin;
            Ok(())
        }

        pub fn admin(&self, ctx: Context<Admin>) -> Result<()> {
            msg!("Admin address: {}", self.admin_address);
            Ok(())
        }
    }



    /// CHECK: realyer is fine
    #[derive(Accounts)]
    pub struct Initialize<'info> {
        #[account(init, payer = user, space = 4096 , seeds=[b"state"] , bump)]
        pub state: Account<'info, CentralizedConnection>,       
        pub relayer: AccountInfo<'info>,
        pub x_call: AccountInfo<'info>,
        pub user: AccountInfo<'info>,
        pub system_program: AccountInfo<'info>,
        pub bump: u8, // Add this field for bump seed

    }

    #[derive(Accounts)]
    pub struct SetFee<'info> {
        #[account(mut)]
        pub state: Loader<'info, CentralizedConnection>,
        pub payer: AccountInfo<'info>,
    }

    #[derive(Accounts)]
    pub struct GetFee<'info> {
        #[account(init, payer = user, space = 64 + 8 * 1024)]
        pub user: AccountInfo<'info>,
        #[account(mut)]
        pub state: Loader<'info, CentralizedConnection>,
    }

    #[derive(Accounts)]
    pub struct SendMessage<'info> {
        #[account(mut)]
        pub to: AccountInfo<'info>,
        #[account(mut)]
        pub x_call: AccountInfo<'info>,
        pub system_program: AccountInfo<'info>,
        #[account(mut)]
        pub state: Loader<'info, CentralizedConnection>,
    }

    #[derive(Accounts)]
    pub struct RecvMessage<'info> {
        #[account(mut)]
        pub x_call: AccountInfo<'info>,
        #[account(mut)]
        pub state: Loader<'info, CentralizedConnection>,
    }

    #[derive(Accounts)]
    pub struct ClaimFees<'info> {
        #[account(mut)]
        pub to: AccountInfo<'info>,
        #[account(mut)]
        pub admin_address: AccountInfo<'info>,
        #[account(mut)]
        pub state: Loader<'info, CentralizedConnection>,
    }

    #[derive(Accounts)]
    pub struct RevertMessage<'info> {
        #[account(mut)]
        pub x_call: AccountInfo<'info>,
        #[account(mut)]
        pub state: Loader<'info, CentralizedConnection>,
    }

    #[derive(Accounts)]
    pub struct GetReceipt<'info> {
        #[account(init, payer = user, space = 64 + 8 * 1024)]
        pub user: AccountInfo<'info>,
        #[account(mut)]
        pub state: Loader<'info, CentralizedConnection>,
    }

    #[derive(Accounts)]
    pub struct SetAdmin<'info> {
        #[account(mut)]
        pub admin_address: Signer<'info>,
        #[account(mut)]
        pub state: Loader<'info, CentralizedConnection>,
    }

    #[event]
    pub struct Message {
        pub target_network: String,
        pub sn: u64,
        pub msg: Vec<u8>,
    }

    #[error]
    pub enum ErrorCode {
        #[msg("Only Xcall can call sendMessage")]
        OnlyXcallCanCallSendMessage,

        #[msg("Fee is not sufficient")]
        FeeNotSufficient,

        #[msg("Invalid sn value")]
        InvalidSn,

        #[msg("Duplicate message")]
        DuplicateMessage,

        #[msg("Only Xcall can revert message")]
        OnlyXcallCanRevertMessage,

        #[msg("Only admin can perform this action")]
        OnlyAdmin,

        #[msg("Message not found")]
        MessageNotFound,
    }
}

#[interface]
pub trait ICallService {
    fn handle_message(&mut self, src_network: String, msg: Vec<u8>) -> Result<()>;
    fn handle_error(&mut self, sn: u64) -> Result<()>;
}
