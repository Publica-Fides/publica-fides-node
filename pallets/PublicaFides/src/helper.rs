use crate::pallet;
use substrate_fixed::types::U32F32;

pub fn score_claims(claims: pallet::ResolvedClaims) -> U32F32 {
    let mut true_count: U32F32 = U32F32::from_num(0);
    let iter_true_claims = claims.claims.iter();
    // claims should be max 10
    for claim in iter_true_claims {
        if claim.is_accepted == true {
            true_count = true_count + U32F32::from_num(1);
        };
    };
    let total_claims = claims.claims.len() as u32;
    let total = U32F32::from_num(total_claims);
    true_count / total
}
