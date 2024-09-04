use anchor_lang::prelude::*;
declare_id!("BZ3ek43wGJHWnmNp4snCpmd165L3wZkjobA4WJFkjRdm");

#[program]
pub mod unsecure_program {
    use super::*;

    pub fn initialize_id_generator(ctx: Context<InitializeIdGenerator>) -> Result<()> {
        ctx.accounts.id_generator.last_timestamp = 0;
        ctx.accounts.id_generator.counter = 0;
        Ok(())
    }

    pub fn initialize(ctx: Context<CreateUser>, name: String) -> Result<()> {
        let id_generator = &mut ctx.accounts.id_generator;
        let user = &mut ctx.accounts.user;

        if name.len() == 0 || name.len() > 10 {
            return err!(MyError::InvalidNameLength);
        }

        // Generate a new unique ID
        let current_timestamp = Clock::get()?.unix_timestamp as u64;
        if current_timestamp > id_generator.last_timestamp {
            id_generator.last_timestamp = current_timestamp;
            id_generator.counter = 0;
        } else {
            id_generator.counter += 1;
        }
        let new_id = (current_timestamp << 32) | (id_generator.counter as u64);

        // Initialize the user account
        user.id = new_id;
        user.owner = *ctx.accounts.signer.key;
        user.name = name;
        user.points = 1000;

        msg!("Created new user with ID: {} and 1000 points", new_id);
        Ok(())
    }

    pub fn transfer_points(ctx: Context<TransferPoints>, amount: u16) -> Result<()> {
        let sender = &mut ctx.accounts.sender;
        let receiver = &mut ctx.accounts.receiver;

        // Check if sender has enough points
        if sender.points < amount {
            return err!(MyError::NotEnoughPoints);
        }

        // Check for overflow
        let new_receiver_points = receiver
            .points
            .checked_add(amount)
            .ok_or(MyError::OverflowError)?;

        // Perform the transfer
        sender.points -= amount;
        receiver.points = new_receiver_points;

        msg!(
            "Transferred {} points from {} to {}",
            amount,
            sender.id,
            receiver.id
        );
        Ok(())
    }

    pub fn remove_user(ctx: Context<RemoveUser>) -> Result<()> {
        let user = ctx.accounts.user.clone();
        let user_id = user.id;

        // Clear the account data
        ctx.accounts
            .user
            .close(ctx.accounts.owner.to_account_info())?;

        msg!("Account closed for user with id: {}", user_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeIdGenerator<'info> {
    #[account(
        init,
        payer = signer,
        space = DISCRIMINATOR + IdGenerator::INIT_SPACE,
        seeds = [b"id_generator"],
        bump
    )]
    pub id_generator: Account<'info, IdGenerator>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateUser<'info> {
    #[account(
        init,
        payer = signer,
        space = DISCRIMINATOR + User::INIT_SPACE,
        seeds = [b"user", &[id_generator.last_timestamp.to_le_bytes()][0][0..4], &id_generator.counter.to_le_bytes()],
        bump
    )]
    pub user: Account<'info, User>,
    #[account(mut)]
    pub id_generator: Account<'info, IdGenerator>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct TransferPoints<'info> {
    #[account(
        mut,
        seeds = [b"user", &sender.id.to_le_bytes()],
        bump,
        has_one = owner @ MyError::UnauthorizedTransfer,
    )]
    pub sender: Account<'info, User>,
    #[account(
        mut,
        seeds = [b"user", &receiver.id.to_le_bytes()],
        bump
    )]
    pub receiver: Account<'info, User>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RemoveUser<'info> {
    #[account(
        mut,
        close = owner,
        seeds = [b"user", &user.id.to_le_bytes()],
        bump,
        has_one = owner @ MyError::UnauthorizedRemoval
    )]
    pub user: Account<'info, User>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(InitSpace)]
pub struct IdGenerator {
    pub last_timestamp: u64,
    pub counter: u32,
}

#[account]
#[derive(Default, InitSpace)]
pub struct User {
    pub id: u64,
    pub owner: Pubkey,
    #[max_len(10)]
    pub name: String,
    pub points: u16,
}

const DISCRIMINATOR: usize = 8;

#[error_code]
pub enum MyError {
    #[msg("Invalid name length. Should be >0 and =<10.")]
    InvalidNameLength,
    #[msg("Not enough points to transfer")]
    NotEnoughPoints,
    #[msg("Arithmetic overflow occurred")]
    OverflowError,
    #[msg("Unauthorized transfer attempt")]
    UnauthorizedTransfer,
    #[msg("Unauthorized attempt to remove user")]
    UnauthorizedRemoval,
}
