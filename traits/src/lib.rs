#![cfg_attr(not(feature = "std"), no_std)]
use frame_support::dispatch::DispatchResultWithPostInfo;

/// traits for Org users 
pub trait DoraUserOrigin {
    type AccountId;
    type AppId;
    type OrgId;
    fn ensure_valid(who: Self::AccountId, org: Self::OrgId, app: Self::AppId) -> DispatchResultWithPostInfo;
}

/// traits for Developers
pub trait DoraDeveloperOrigin<AccountId, AppId> {
    fn ensure_signed(who: AccountId, app: AppId) -> DispatchResultWithPostInfo;
}

/// traits for payment
pub trait DoraPay {
    type AccountId;
    type Balance;
    type AppId;

    fn charge(source: Self::AccountId, value: Self::Balance, app_id: Self::AppId) -> DispatchResultWithPostInfo;
    fn withdraw(dest: Self::AccountId, value: Self::Balance, app_id: Self::AppId) -> DispatchResultWithPostInfo;
}