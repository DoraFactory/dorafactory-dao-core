#![cfg_attr(not(feature = "std"), no_std)]
use frame_support::dispatch::DispatchResultWithPostInfo;

/// traits for Org users 
pub trait DoraUserOrigin {
    type AccountId;
    type AppId;
    type OrgId;
    fn ensure_signed(who: Self::AccountId, org: Self::OrgId, app: Self::AppId) -> DispatchResultWithPostInfo;
}

/// traits for Developers
pub trait DoraDeveloperOrigin<AccountId, AppId> {
    fn ensure_signed(who: AccountId, app: AppId) -> DispatchResultWithPostInfo;
}