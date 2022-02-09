#![cfg_attr(not(feature = "std"), no_std)]

use core_services::{DoraPay, DoraUserOrigin};
use frame_support::{
    codec::{Decode, Encode},
    traits::{
        Currency, ExistenceRequirement::KeepAlive, Get, OnUnbalanced, ReservableCurrency,
        UnfilteredDispatchable,
    },
    weights::GetDispatchInfo,
    PalletId,
};
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use pallet::*;
use scale_info::TypeInfo;
use sp_runtime::traits::AccountIdConversion;
use sp_std::boxed::Box;
use sp_std::{convert::TryInto, vec::Vec};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[derive(Encode, Decode, Default, Clone, PartialEq, TypeInfo)]
pub struct Orgnization<AccountId> {
    pub org_type: u32,
    pub description: Vec<u8>,
    pub owner: AccountId,
    pub members: Vec<AccountId>,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, TypeInfo)]
pub struct AppInfo<AccountId, Balance> {
    pub title: Vec<u8>,
    pub owner: AccountId,
    pub description: Vec<u8>,
    pub charge_type: u32,
    pub price: Balance,
}

type OrgnizationOf<T> = Orgnization<<T as frame_system::Config>::AccountId>;
type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
type AppInfoOf<T, P> = AppInfo<<T as frame_system::Config>::AccountId, BalanceOf<P>>;
pub const MAX_MEMBERS: usize = 16;

#[frame_support::pallet]
pub mod pallet {
    pub use super::*;
    pub use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    pub use frame_system::pallet_prelude::*;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Call: Parameter + UnfilteredDispatchable<Origin = Self::Origin> + GetDispatchInfo;
        type Currency: ReservableCurrency<Self::AccountId>;
        #[pallet::constant]
        type PalletId: Get<PalletId>;
        type TaxInPercent: Get<u32>;
        type SupervisorOrigin: EnsureOrigin<Self::Origin>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    // The pallet's runtime storage items.
    // https://substrate.dev/docs/en/knowledgebase/runtime/storage
    #[pallet::storage]
    #[pallet::getter(fn next_org_id)]
    // Learn more about declaring storage items:
    // https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
    pub(super) type NextOrgId<T> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn orgnizations)]
    pub(super) type Orgnizations<T: Config> =
        StorageMap<_, Blake2_128Concat, u32, OrgnizationOf<T>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn next_app_id)]
    pub(super) type NextAppId<T> = StorageValue<_, u8, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn registered_apps)]
    pub(super) type RegisteredApps<T: Config> =
        StorageMap<_, Blake2_128Concat, u8, AppInfoOf<T, T>, ValueQuery>;

    // Pallets use events to inform users when important changes are made.
    // https://substrate.dev/docs/en/knowledgebase/runtime/events
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Event documentation should end with an array that provides descriptive names for event
        /// parameters. [ord_id, owner]
        OrgRegistered(u32, T::AccountId),
        OrgJoined(u32, T::AccountId),
        AppRegistered(u8, T::AccountId),
        AppDeRegistered(u8, T::AccountId),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// Error names should be descriptive.
        NoneValue,
        /// Errors should have helpful documentation associated with them.
        OrgnizationNotExist,
        NotValidOrgMember,
        AppNotExist,
        AppNotEnable,
        AppNotFound,
        BadOrigin,
        NoRepeatJoin,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// An example dispatchable that takes a singles value as a parameter, writes the value to
        /// storage and emits an event. This function must be dispatched by a signed extrinsic.
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn create(origin: OriginFor<T>, description: Vec<u8>) -> DispatchResultWithPostInfo {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://substrate.dev/docs/en/knowledgebase/runtime/origin
            let who = ensure_signed(origin)?;
            let member = super::Orgnization {
                org_type: 1,
                description: description.clone(),
                owner: who.clone(),
                members: [who.clone()].to_vec(),
            };
            let org_id = <NextOrgId<T>>::get().checked_add(1).unwrap();
            Orgnizations::<T>::insert(org_id, member);
            <NextOrgId<T>>::put(org_id);
            Self::deposit_event(Event::OrgRegistered(org_id, who));
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn join(origin: OriginFor<T>, org_id: u32) -> DispatchResultWithPostInfo {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://substrate.dev/docs/en/knowledgebase/runtime/origin
            let who = ensure_signed(origin)?;
            ensure!(
                Orgnizations::<T>::contains_key(&org_id),
                Error::<T>::OrgnizationNotExist
            );
            let members = Orgnizations::<T>::get(org_id).members;
            ensure!(
                members.binary_search(&who).is_err(),
                Error::<T>::NoRepeatJoin
            );
            Orgnizations::<T>::mutate(org_id, |org| {
                org.members.push(who.clone());
            });
            Self::deposit_event(Event::OrgJoined(org_id, who));
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn invoke(
            origin: OriginFor<T>,
            ord_id: u32,
            pallet: Box<<T as Config>::Call>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin.clone())?;
            ensure!(
                Self::validate_member(who.clone(), ord_id),
                Error::<T>::NotValidOrgMember
            );
            let _ = pallet.dispatch_bypass_filter(origin);
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn register_app(
            origin: OriginFor<T>,
            title: Vec<u8>,
            owner: T::AccountId,
            description: Vec<u8>,
            charge_type: u32,
            #[pallet::compact] price: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let _ = T::SupervisorOrigin::ensure_origin(origin)?;
            let app = super::AppInfo {
                charge_type: charge_type,
                title: title.clone(),
                description: description.clone(),
                owner: owner.clone(),
                price: price,
            };
            let app_id = <NextAppId<T>>::get().checked_add(1).unwrap();
            RegisteredApps::<T>::insert(app_id, app);
            <NextAppId<T>>::put(app_id);
            Self::deposit_event(Event::AppRegistered(app_id, owner));
            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn deregister_app(origin: OriginFor<T>, app_id: u8) -> DispatchResultWithPostInfo {
            let _ = T::SupervisorOrigin::ensure_origin(origin)?;
            ensure!(
                RegisteredApps::<T>::contains_key(app_id),
                Error::<T>::AppNotFound
            );
            RegisteredApps::<T>::remove(app_id);
            Ok(().into())
        }
    }
}

impl<T: Config> Pallet<T> {
    fn account_id(app_id: u8) -> T::AccountId {
        if app_id == 0 {
            T::PalletId::get().into_account()
        } else {
            T::PalletId::get().into_sub_account(app_id)
        }
    }

    fn u128_to_balance(cost: u128) -> BalanceOf<T> {
        TryInto::<BalanceOf<T>>::try_into(cost).ok().unwrap()
    }

    fn balance_to_u128(balance: BalanceOf<T>) -> u128 {
        TryInto::<u128>::try_into(balance).ok().unwrap()
    }

    /// refer https://github.com/paritytech/substrate/blob/743accbe3256de2fc615adcaa3ab03ebdbbb4dbd/frame/treasury/src/lib.rs#L351
    ///
    /// This actually does computation. If you need to keep using it, then make sure you cache the
    /// value and only call this once.
    pub fn validate_member(account_id: T::AccountId, ord_id: u32) -> bool {
        if !Orgnizations::<T>::contains_key(ord_id) {
            false
        } else {
            let members = Orgnizations::<T>::get(ord_id).members;
            match members.binary_search(&account_id) {
                Ok(_) => true,
                Err(_) => false,
            }
        }
    }

    pub fn charge(
        source: T::AccountId,
        value: BalanceOf<T>,
        app_id: u8,
    ) -> frame_support::dispatch::DispatchResultWithPostInfo {
        let value_num = Self::balance_to_u128(value);
        let tax_num = value_num
            .checked_mul(T::TaxInPercent::get().into())
            .unwrap()
            .checked_div(100)
            .unwrap();
        let tax = Self::u128_to_balance(tax_num);
        // charge tax
        let _ = T::Currency::transfer(&source, &Self::account_id(0), tax, KeepAlive);
        // process rest to App's escrow account
        let _ = T::Currency::transfer(&source, &Self::account_id(app_id), value - tax, KeepAlive);

        Ok(().into())
    }
}

impl<T: Config> DoraUserOrigin for Pallet<T> {
    type AccountId = T::AccountId;
    type AppId = u8;
    type OrgId = u32;

    fn ensure_valid(
        who: T::AccountId,
        org: u32,
        app: u8,
    ) -> frame_support::dispatch::DispatchResultWithPostInfo {
        if !RegisteredApps::<T>::contains_key(app) {
            Err(Error::<T>::AppNotExist)?
        }
        if !Orgnizations::<T>::contains_key(org) {
            Err(Error::<T>::OrgnizationNotExist)?
        } else {
            let members = Orgnizations::<T>::get(org).members;
            match members.binary_search(&who) {
                Ok(_) => Ok(().into()),
                Err(_) => Err(Error::<T>::NotValidOrgMember)?,
            }
        }
    }
}

impl<T: Config> DoraPay for Pallet<T> {
    type AccountId = T::AccountId;
    type Balance = BalanceOf<T>;
    type AppId = u8;

    fn charge(
        source: T::AccountId,
        value: BalanceOf<T>,
        app_id: u8,
    ) -> frame_support::dispatch::DispatchResultWithPostInfo {
        let value_num = Self::balance_to_u128(value);
        let tax_num = value_num
            .checked_mul(T::TaxInPercent::get().into())
            .unwrap()
            .checked_div(100)
            .unwrap();
        let tax = Self::u128_to_balance(tax_num);
        // charge tax
        let _ = T::Currency::transfer(&source, &Self::account_id(0), tax, KeepAlive);
        // process rest to App's escrow account
        let _ = T::Currency::transfer(&source, &Self::account_id(app_id), value - tax, KeepAlive);

        Ok(().into())
    }

    /// withdraw from escrow account do not tax
    fn withdraw(
        dest: T::AccountId,
        value: BalanceOf<T>,
        app_id: u8,
    ) -> frame_support::dispatch::DispatchResultWithPostInfo {
        let _ = T::Currency::transfer(&Self::account_id(app_id), &dest, value, KeepAlive);
        Ok(().into())
    }
}
