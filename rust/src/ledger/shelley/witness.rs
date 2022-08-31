use crate::crypto::{TransactionHash, PrivateKey, Vkeywitness, Vkey};


pub fn make_vkey_witness(
    tx_body_hash: &TransactionHash,
    sk: &PrivateKey
) -> Vkeywitness {
    let sig = sk.sign(tx_body_hash.0.as_ref());
    Vkeywitness::new(&Vkey::new(&sk.to_public()), &sig)
}
