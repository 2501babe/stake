#![allow(dead_code)]
#![allow(unused_imports)]

use {
    crate::{feature_set_die, stake_history_die},
    num_traits::cast::ToPrimitive,
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        clock::{Clock, Epoch},
        entrypoint::ProgramResult,
        instruction::{checked_add, InstructionError},
        msg,
        program_error::ProgramError,
        program_utils::limited_deserialize,
        pubkey::Pubkey,
        rent::Rent,
        stake::state::*,
        stake::{
            instruction::{LockupArgs, StakeError, StakeInstruction},
            program::id,
            stake_flags::StakeFlags,
            state::{Authorized, Lockup},
            tools::{acceptable_reference_epoch_credits, eligible_for_deactivate_delinquent},
        },
        stake_history::{StakeHistory, StakeHistoryEntry},
        sysvar::Sysvar,
        vote::program as solana_vote_program,
        vote::state::{VoteState, VoteStateVersions},
    },
    std::{cmp::Ordering, collections::HashSet, convert::TryFrom},
};

// XXX note to self. InstructionError is actually a superset of ProgramError
// there is a TryFrom instance, but thats why theres no From instance
// there are ProgramError conversions between u64 tho, and From<T> for InstructionError where T: FromPrimitive
// very unusual. i guess i can look more into this but for now using ProgramError is fine seems safe

// XXX a nice change would be to pop an account off the queue and discard if its a gettable sysvar
// ie, allow people to omit them from the accounts list without breaking compat

/// XXX THIS SECTION is new utility functions and stuff like that

// XXX errors changed from GenericError
fn set_stake_state(
    stake_account_info: &AccountInfo,
    new_state: &StakeStateV2,
) -> Result<(), ProgramError> {
    let serialized_size =
        bincode::serialized_size(new_state).map_err(|_| ProgramError::InvalidAccountData)?;
    if serialized_size > stake_account_info.data_len() as u64 {
        return Err(ProgramError::AccountDataTooSmall);
    }

    bincode::serialize_into(&mut stake_account_info.data.borrow_mut()[..], new_state)
        .map_err(|_| ProgramError::InvalidAccountData)
}

// XXX impl from<StakeError> for ProgramError. also idk if this is correct
// i just want to keep the same errors in-place and then clean up later, instead of needing to hunt down the right ones
pub trait TurnInto {
    fn turn_into(&self) -> ProgramError;
}
impl TurnInto for StakeError {
    fn turn_into(&self) -> ProgramError {
        ProgramError::Custom(self.to_u32().unwrap())
    }
}

/// XXX THIS SECTION is mostly copy-pasted from stake_state.rs

/// After calling `validate_delegated_amount()`, this struct contains calculated values that are used
/// by the caller.
struct ValidatedDelegatedInfo {
    stake_amount: u64,
}

pub(crate) fn new_stake(
    stake: u64,
    voter_pubkey: &Pubkey,
    vote_state: &VoteState,
    activation_epoch: Epoch,
) -> Stake {
    Stake {
        delegation: Delegation::new(voter_pubkey, stake, activation_epoch),
        credits_observed: vote_state.credits(),
    }
}

/// Ensure the stake delegation amount is valid.  This checks that the account meets the minimum
/// balance requirements of delegated stake.  If not, return an error.
fn validate_delegated_amount(
    account: &AccountInfo,
    meta: &Meta,
) -> Result<ValidatedDelegatedInfo, ProgramError> {
    let stake_amount = account.lamports().saturating_sub(meta.rent_exempt_reserve); // can't stake the rent

    // Stake accounts may be initialized with a stake amount below the minimum delegation so check
    // that the minimum is met before delegation.
    if stake_amount < crate::get_minimum_delegation() {
        return Err(StakeError::InsufficientDelegation.turn_into());
    }
    Ok(ValidatedDelegatedInfo { stake_amount })
}

/// XXX THIS SECTION is the new processor

pub struct Processor {}
impl Processor {
    fn process_initialize(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        authorized: Authorized,
        lockup: Lockup,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let stake_account_info = next_account_info(account_info_iter)?;
        let rent_info = next_account_info(account_info_iter)?;
        let rent = &Rent::from_account_info(rent_info)?;

        if stake_account_info.data_len() != StakeStateV2::size_of() {
            return Err(ProgramError::InvalidAccountData);
        }

        if let StakeStateV2::Uninitialized = stake_account_info
            .deserialize_data()
            .map_err(|_| ProgramError::InvalidAccountData)?
        {
            let rent_exempt_reserve = rent.minimum_balance(stake_account_info.data_len());
            if stake_account_info.lamports() >= rent_exempt_reserve {
                let stake_state = StakeStateV2::Initialized(Meta {
                    rent_exempt_reserve,
                    authorized: authorized,
                    lockup: lockup,
                });

                set_stake_state(stake_account_info, &stake_state)?;

                Ok(()) // XXX the above error as-written is InstructionError::GenericError
            } else {
                Err(ProgramError::InsufficientFunds)
            }
        } else {
            Err(ProgramError::InvalidAccountData)
        }?;

        Ok(())
    }

    fn process_delegate(_program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let stake_account_info = next_account_info(account_info_iter)?;
        let vote_account_info = next_account_info(account_info_iter)?;
        let clock_info = next_account_info(account_info_iter)?;
        let clock = &Clock::from_account_info(clock_info)?;
        let _stake_history_info = next_account_info(account_info_iter)?;
        let _stake_config_info = next_account_info(account_info_iter)?;
        let stake_authority_info = next_account_info(account_info_iter)?;

        if *vote_account_info.owner != solana_vote_program::id() {
            return Err(ProgramError::IncorrectProgramId);
        }

        if !stake_authority_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // XXX when im back on a branch with this
        //let mut vote_state = Box::new(VoteState::default());
        //VoteState::deserialize_into(&vote_account_info.data.borrow(), &mut vote_state).unwrap();
        //let vote_state = vote_state;
        let vote_state = VoteState::deserialize(&vote_account_info.data.borrow()).unwrap();

        // XXX parse stake account, branch on enum, new stake or redelegate

        let stake_state = stake_account_info
            .deserialize_data()
            .map_err(|_| ProgramError::InvalidAccountData)?;
        match stake_state {
            StakeStateV2::Initialized(meta) => {
                if meta.authorized.staker != *stake_authority_info.key {
                    return Err(ProgramError::MissingRequiredSignature);
                }

                let ValidatedDelegatedInfo { stake_amount } =
                    validate_delegated_amount(&stake_account_info, &meta)?;

                let new_stake_state = new_stake(
                    stake_amount,
                    vote_account_info.key,
                    &vote_state,
                    clock.epoch,
                );

                set_stake_state(
                    stake_account_info,
                    &StakeStateV2::Stake(meta, new_stake_state, StakeFlags::empty()),
                )
            }
            StakeStateV2::Stake(meta, mut stake, stake_flags) => {
                if meta.authorized.staker != *stake_authority_info.key {
                    return Err(ProgramError::MissingRequiredSignature);
                }

                let ValidatedDelegatedInfo { stake_amount } =
                    validate_delegated_amount(&stake_account_info, &meta)?;

                // TODO redelegate, then set state
                unimplemented!()
            }
            _ => Err(ProgramError::InvalidAccountData),
        }?;

        Ok(())
    }

    /// Processes [Instruction](enum.Instruction.html).
    // XXX the existing program returns InstructionError not ProgramError
    // look into if theres a trait i can impl to not break the interface but modrenize
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
        let instruction = bincode::deserialize(data).unwrap(); // XXX limited_deserialize?

        match instruction {
            StakeInstruction::Initialize(authorized, lockup) => {
                msg!("Instruction: Initialize");
                Self::process_initialize(program_id, accounts, authorized, lockup)
            }
            StakeInstruction::DelegateStake => {
                msg!("Instruction: DelegateStake");

                if !crate::FEATURE_REDUCE_STAKE_WARMUP_COOLDOWN {
                    panic!("we only impl the `reduce_stake_warmup_cooldown` logic");
                }

                Self::process_delegate(program_id, accounts)
            }
            _ => unimplemented!(),
        }
    }
}