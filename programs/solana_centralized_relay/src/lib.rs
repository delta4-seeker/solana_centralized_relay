use anchor_lang::prelude::*;
use std::mem::size_of;
declare_id!("35XcNkLUyBHetw7CVZL6Gg9R1AhZgzS1ektLy1Yeiv8e");

//Instructions
#[program]
pub mod solana_centralized_relay {
    use super::*;

    pub fn initialize(
        _ctx: Context<InitializeCtx>,
        _relayer: Pubkey,
        _xcall: Pubkey,
    ) -> Result<()> {
        require_keys_eq!(
            _ctx.accounts.centralized_connection_state.admin_address,
            _ctx.accounts.user.key()
        );
        _ctx.accounts.centralized_connection_state.admin_address = _relayer;
        _ctx.accounts.centralized_connection_state.xcall_address = _xcall;
        Ok(())
    }

    pub fn set_admin(_ctx: Context<SetAdminCtx>, _admin: Pubkey) -> Result<()> {
        require_keys_eq!(
            _ctx.accounts.centralized_connection_state.admin_address,
            _ctx.accounts.user.key()
        );
        _ctx.accounts.centralized_connection_state.admin_address = _admin;
        Ok(())
    }

    pub fn get_admin(_ctx: Context<GetAdminCtx>) -> Result<Pubkey> {
        Ok(_ctx.accounts.centralized_connection_state.admin_address)
    }

    pub fn set_fee(
        _ctx: Context<SetFeeCtx>,
        _message_fee: u64,
        _response_fee: u64,
        _network: String,
    ) -> Result<()> {
        require_keys_eq!(
            _ctx.accounts.user.key(),
            _ctx.accounts.centralized_connection_state.admin_address
        );
        _ctx.accounts.fees.message_fees = _message_fee;
        _ctx.accounts.fees.response_fees = _response_fee;
        Ok(())
    }
    pub fn get_fee(_ctx: Context<GetFeeCtx>, _network: String, response: bool) -> Result<u64> {
        let message_fee = &mut _ctx.accounts.fees.message_fees.clone();

        if response == true {
            *message_fee += _ctx.accounts.fees.response_fees;
        }
        Ok(*message_fee)
    }

    pub fn send_message(
        _ctx: Context<SendMessageCtx>,
        _to: String,
        _svc: String,
        _sn: i128,
        _msg: Vec<i8>,
    ) -> Result<()> {
        require_keys_eq!(
            _ctx.accounts.user.key(),
            _ctx.accounts.centralized_connection_state.xcall_address
        );
        let mut fee: u64 = 0;
        if _sn > 0 {
            fee = _ctx.accounts.fees.total_fees(true);
        } else if _sn == 0 {
            fee = _ctx.accounts.fees.total_fees(false);
        }
        if _ctx.remaining_accounts[0].lamports() < fee {
            return Err(ErrorCode::InsufficientFee.into());
        }
        _ctx.accounts.centralized_connection_state.conn_sn += 1;
        // todo!("transfer funds from user to program")
        // todo!("emit message event");
        Ok(())
    }

    pub fn recv_receipt(
        _ctx: Context<RecvReceiptCtx>,
        _src_network: String,
        _conn_sn: u128,
        _msg: Vec<i8>,
    ) -> Result<()> {
        require_keys_eq!(
            _ctx.accounts.centralized_connection_state.admin_address,
            _ctx.accounts.user.key()
        );
        if _ctx.accounts.receipt.receive_status == true {
            return Err(ErrorCode::DuplicateMessage.into());
        }
        _ctx.accounts.receipt.receive_status = true;
        // todo!("ICallService(xCall).handleMessage(srcNetwork, _msg);");

        Ok(())
    }

    pub fn get_receipt(
        _ctx: Context<GetReceiptCtx>,
        _src_network: String,
        _conn_sn: u128,
    ) -> Result<bool> {
        return Ok(_ctx.accounts.receipt.receive_status);
    }
}

// Instructure context structures
#[derive(Accounts)]
pub struct InitializeCtx<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(init, payer= user, space = 8 + size_of::<CentralizedConnectionState>(), seeds = [b"centralized_state"],  bump)]
    pub centralized_connection_state: Account<'info, CentralizedConnectionState>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetAdminCtx<'info> {
    #[account(mut, seeds = [b"centralized_state"],  bump)]
    pub user: Signer<'info>,
    pub centralized_connection_state: Account<'info, CentralizedConnectionState>,
}

#[derive(Accounts)]
pub struct GetAdminCtx<'info> {
    #[account( seeds = [b"centralized_state"],  bump)]
    pub centralized_connection_state: Account<'info, CentralizedConnectionState>,
}

#[derive(Accounts)]
#[instruction(_network: String)]
pub struct SetFeeCtx<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(init, payer= user, space = 8 + size_of::<FeesState>(), seeds = [b"fees", _network.as_bytes()], bump)]
    pub fees: Account<'info, FeesState>,
    pub centralized_connection_state: Account<'info, CentralizedConnectionState>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(_network: String)]
pub struct GetFeeCtx<'info> {
    #[account(seeds = [b"fees", _network.as_bytes()] , bump)]
    pub fees: Account<'info, FeesState>,
}

#[derive(Accounts)]
#[instruction(_src_network: String, _conn_sn: u64)]

pub struct RecvReceiptCtx<'info> {
    #[account(init, space = 8 + size_of::<ReceiptState>() ,  payer = user , seeds = [b"receipt" , _src_network.as_bytes(), &_conn_sn.to_le_bytes()] , bump)]
    pub receipt: Account<'info, ReceiptState>,
    pub system_program: Program<'info, System>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account( seeds = [b"centralized_state"],  bump)]
    pub centralized_connection_state: Account<'info, CentralizedConnectionState>,
}

#[derive(Accounts)]
#[instruction(_network: String, _conn_sn: u64)]
pub struct GetReceiptCtx<'info> {
    #[account(seeds = [b"receipt" , _network.as_bytes(), &_conn_sn.to_le_bytes()] , bump)]
    pub receipt: Account<'info, ReceiptState>,
}

#[derive(Accounts)]
#[instruction(_sn: u128 , _to: String)]
pub struct SendMessageCtx<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,

    #[account( seeds = [b"centralized_state"],  bump)]
    pub centralized_connection_state: Account<'info, CentralizedConnectionState>,

    #[account(seeds = [b"fees", _to.as_bytes()] , bump)]
    pub fees: Account<'info, FeesState>,

    #[account(init, payer=user, space = 8 + size_of::<MessageEvent>(), seeds = [b"messageEvent" , _sn.to_le_bytes().as_ref()] , bump)]
    pub message_event: Account<'info, MessageEvent>,
}

#[account]
#[derive(Default)]
pub struct MessageEvent {
    pub target_network: String,
    pub sn: u128,
    pub _msg: Vec<u8>,
}

// data structures
#[account]
#[derive(Default)]
pub struct CentralizedConnectionState {
    pub xcall_address: Pubkey,
    pub admin_address: Pubkey,
    pub conn_sn: u128,
}

#[account]
#[derive(Default)]

pub struct ReceiptState {
    src_network: String,
    conn_sn: u64,
    receive_status: bool,
}

#[account]
#[derive(Default)]
pub struct FeesState {
    network: String,
    message_fees: u64,
    response_fees: u64,
}

impl FeesState {
    pub fn new(network: String, message_fees: u64, response_fees: u64) -> Self {
        FeesState {
            network,
            message_fees,
            response_fees,
        }
    }

    pub fn total_fees(&self, response: bool) -> u64 {
        if response {
            self.message_fees + self.response_fees
        } else {
            self.message_fees
        }
    }
}

#[error_code]
pub enum ErrorCode {
    #[msg("Only Xcall can call sendMessage")]
    OnlyXcallCanCallSendMessage,
    #[msg("Fee is not sufficient")]
    InsufficientFee,
    #[msg("Invalid serial number")]
    DuplicateMessage,
    #[msg("Duplicate Message")]
    InvalidSerialNumber,
}
