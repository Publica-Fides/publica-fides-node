use super::*;
use crate::{helper, mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn it_stores_contents() {
	new_test_ext().execute_with(|| {
		assert_ok!(PublicaFides::store_content(Origin::signed(1), vec![1, 2]));
		assert_eq!(PublicaFides::next_class_id(), 1)
	});
}

#[test]
fn it_stores_claims() {
	new_test_ext().execute_with(|| {
		assert_ok!(PublicaFides::store_content(Origin::signed(1), vec![1, 2]));

		assert_ok!(PublicaFides::store_claim_for_content(Origin::signed(1), vec![1, 2], 0, false));
		assert_eq!(PublicaFides::next_claim_id(), 1);
		assert_eq!(
			PublicaFides::get_claims(0, 0),
			Claim { claim_text_cid: [1, 2].to_vec(), is_accepted: false }
		);
	});
}

#[test]
fn it_fails_if_content_nonexistent() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			PublicaFides::store_claim_for_content(Origin::signed(1), vec![1, 2], 0, false),
			Error::<Test>::NonExistentContent
		);
	});
}

#[test]
fn it_calculates_content_claims_score() {
	new_test_ext().execute_with(|| {
		let claim_is_accepted = |is_accepted| Claim { claim_text_cid: [].to_vec(), is_accepted };
		let claims = ResolvedClaims {
			claims: [
				claim_is_accepted(true),
				claim_is_accepted(true),
				claim_is_accepted(false),
				claim_is_accepted(true),
			]
			.to_vec(),
		};
		assert_eq!(helper::score_claims(claims), 0.75);
	});
}
