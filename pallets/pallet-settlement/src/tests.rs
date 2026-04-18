use crate::{mock::*, Error, Placeholder};
use frame_support::{assert_ok, assert_noop};

#[test]
fn it_works_for_default_value() {
    new_test_ext().execute_with(|| {
        // Just a placeholder test
        assert!(true);
    });
}
