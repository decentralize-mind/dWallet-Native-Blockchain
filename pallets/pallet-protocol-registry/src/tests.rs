use crate::{mock::*, Error, LayerRegistry, GenesisPhase, PendingUpdates, TIMELOCK_PERIOD, GENESIS_DURATION};
use frame_support::{assert_ok, assert_noop};

#[test]
fn test_register_layer_during_genesis() {
    new_test_ext().execute_with(|| {
        // Register layer 1
        assert_ok!(ProtocolRegistry::register_layer(
            RuntimeOrigin::root(),
            1,
            1001
        ));

        // Verify layer was registered
        let layer_info = ProtocolRegistry::get_layer_info(1).unwrap();
        assert_eq!(layer_info.layer_id, 1);
        assert_eq!(layer_info.address, 1001);
        
        // Check event was emitted
        System::assert_last_event(
            RuntimeEvent::ProtocolRegistry(crate::Event::LayerRegistered {
                layer_id: 1,
                address: 1001,
            })
        );
    });
}

#[test]
fn test_cannot_register_duplicate_layer() {
    new_test_ext().execute_with(|| {
        // Register layer 1
        assert_ok!(ProtocolRegistry::register_layer(
            RuntimeOrigin::root(),
            1,
            1001
        ));

        // Try to register again - should fail
        assert_noop!(
            ProtocolRegistry::register_layer(RuntimeOrigin::root(), 1, 1002),
            Error::<Test>::LayerAlreadyRegistered
        );
    });
}

#[test]
fn test_cannot_register_invalid_layer_id() {
    new_test_ext().execute_with(|| {
        // Try to register layer 11 (invalid)
        assert_noop!(
            ProtocolRegistry::register_layer(RuntimeOrigin::root(), 11, 1001),
            Error::<Test>::InvalidLayerId
        );
    });
}

#[test]
fn test_initiate_layer_update_after_genesis() {
    new_test_ext().execute_with(|| {
        // Register layer first
        assert_ok!(ProtocolRegistry::register_layer(
            RuntimeOrigin::root(),
            1,
            1001
        ));

        // End genesis phase
        GenesisPhase::<Test>::put(false);

        // Initiate update
        assert_ok!(ProtocolRegistry::initiate_layer_update(
            RuntimeOrigin::root(),
            1,
            2001
        ));

        // Verify pending update exists
        let pending = ProtocolRegistry::pending_update(1).unwrap();
        assert_eq!(pending.0, 2001);
    });
}

#[test]
fn test_cannot_update_during_genesis() {
    new_test_ext().execute_with(|| {
        // Register layer
        assert_ok!(ProtocolRegistry::register_layer(
            RuntimeOrigin::root(),
            1,
            1001
        ));

        // Try to update during genesis - should fail
        assert_noop!(
            ProtocolRegistry::initiate_layer_update(RuntimeOrigin::root(), 1, 2001),
            Error::<Test>::GenesisPhaseActive
        );
    });
}

#[test]
fn test_execute_layer_update_after_timelock() {
    new_test_ext().execute_with(|| {
        // Register layer
        assert_ok!(ProtocolRegistry::register_layer(
            RuntimeOrigin::root(),
            1,
            1001
        ));

        // End genesis
        GenesisPhase::<Test>::put(false);

        // Initiate update
        assert_ok!(ProtocolRegistry::initiate_layer_update(
            RuntimeOrigin::root(),
            1,
            2001
        ));

        // Try to execute immediately - should fail
        assert_noop!(
            ProtocolRegistry::execute_layer_update(RuntimeOrigin::root(), 1),
            Error::<Test>::TimelockNotExpired
        );

        // Advance blocks past timelock
        System::set_block_number(TIMELOCK_PERIOD as u64 + 100);

        // Now execute should work
        assert_ok!(ProtocolRegistry::execute_layer_update(RuntimeOrigin::root(), 1));

        // Verify layer was updated
        let layer_info = ProtocolRegistry::get_layer_info(1).unwrap();
        assert_eq!(layer_info.address, 2001);

        // Verify pending update was removed
        assert!(ProtocolRegistry::pending_update(1).is_none());
    });
}

#[test]
fn test_end_genesis_phase() {
    new_test_ext().execute_with(|| {
        // Genesis should be active initially (if set in genesis config)
        GenesisPhase::<Test>::put(true);

        // Try to end before duration - should fail
        System::set_block_number(GENESIS_DURATION as u64 - 1);
        assert_noop!(
            ProtocolRegistry::end_genesis_phase(RuntimeOrigin::root()),
            Error::<Test>::GenesisPhaseActive
        );

        // Advance past genesis duration
        System::set_block_number(GENESIS_DURATION as u64 + 1);

        // End genesis
        assert_ok!(ProtocolRegistry::end_genesis_phase(RuntimeOrigin::root()));

        // Verify genesis ended
        assert!(!ProtocolRegistry::is_genesis_phase());
    });
}

#[test]
fn test_helper_functions() {
    new_test_ext().execute_with(|| {
        // Register multiple layers
        assert_ok!(ProtocolRegistry::register_layer(RuntimeOrigin::root(), 1, 1001));
        assert_ok!(ProtocolRegistry::register_layer(RuntimeOrigin::root(), 2, 1002));
        assert_ok!(ProtocolRegistry::register_layer(RuntimeOrigin::root(), 3, 1003));

        // Test get_layer_address
        assert_eq!(ProtocolRegistry::get_layer_address(1), Some(1001));
        assert_eq!(ProtocolRegistry::get_layer_address(2), Some(1002));
        assert_eq!(ProtocolRegistry::get_layer_address(99), None);

        // Test is_layer_registered
        assert!(ProtocolRegistry::is_layer_registered(1));
        assert!(!ProtocolRegistry::is_layer_registered(99));

        // Test get_registered_layers
        let registered = ProtocolRegistry::get_registered_layers();
        assert_eq!(registered.len(), 3);
        assert!(registered.contains(&1));
        assert!(registered.contains(&2));
        assert!(registered.contains(&3));
    });
}

#[test]
fn test_non_root_cannot_register() {
    new_test_ext().execute_with(|| {
        // Non-root origin should fail
        assert_noop!(
            ProtocolRegistry::register_layer(RuntimeOrigin::signed(1), 1, 1001),
            sp_runtime::traits::BadOrigin
        );
    });
}

#[test]
fn test_execute_nonexistent_update() {
    new_test_ext().execute_with(|| {
        // Register layer
        assert_ok!(ProtocolRegistry::register_layer(
            RuntimeOrigin::root(),
            1,
            1001
        ));

        GenesisPhase::<Test>::put(false);

        // Try to execute update that doesn't exist
        assert_noop!(
            ProtocolRegistry::execute_layer_update(RuntimeOrigin::root(), 1),
            Error::<Test>::NoPendingUpdate
        );
    });
}

#[test]
fn test_update_nonexistent_layer() {
    new_test_ext().execute_with(|| {
        GenesisPhase::<Test>::put(false);

        // Try to update layer that doesn't exist
        assert_noop!(
            ProtocolRegistry::initiate_layer_update(RuntimeOrigin::root(), 99, 1001),
            Error::<Test>::LayerNotRegistered
        );
    });
}
