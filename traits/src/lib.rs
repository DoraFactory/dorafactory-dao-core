#![cfg_attr(not(feature = "std"), no_std)]

pub trait MemberSet<AccountId> {
    fn is_member(member: AccountId) -> bool;
}