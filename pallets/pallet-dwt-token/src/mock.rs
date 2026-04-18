use frame_support::traits::{ConstU64, Hooks};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};

use crate as pallet_dwt_token;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
        DWTToken: pallet_dwt_token::{Pallet, Call, Storage, Event<T>},
    }
);

impl system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_dwt_token::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type MintOrigin = frame_system::EnsureRoot<Self::AccountId>;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap()
        .into()
}

pub fn new_test_ext_with_mint() -> sp_io::TestExternalities {
    let mut ext = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    pallet_dwt_token::GenesisConfig::<Test> {
        initial_mint: vec![(1, 1000 * 10u128.pow(18)), (2, 500 * 10u128.pow(18))],
        token_owner: Some(1),
    }
    .assimilate_storage(&mut ext)
    .unwrap();

    ext.into()
}
