use std::{collections::BTreeMap, fmt::Debug};
use crate::{*};

use super::{input_builder::InputBuilderResult, mint_builder::MintBuilderResult, withdrawal_builder::WithdrawalBuilderResult, certificate_builder::CertificateBuilderResult};


#[derive(Clone, Copy, PartialOrd, Ord, Debug, PartialEq, Eq, Hash)]
pub struct RedeemerWitnessKey {
    tag: RedeemerTag,
    index: BigNum,
}


impl RedeemerWitnessKey {

    pub fn tag(&self) -> RedeemerTag {
        self.tag
    }

    pub fn index(&self) -> BigNum {
        self.index
    }

    pub fn new(tag: &RedeemerTag, index: &BigNum) -> Self {
        Self {
            tag: *tag,
            index: *index,
        }
    }
}

/// Redeemer without the tag of index
/// This allows builder code to return partial redeemers
/// and then later have them placed in the right context

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct UntaggedRedeemer {
    data: PlutusData,
    ex_units: ExUnits,
}


impl UntaggedRedeemer {

    pub fn datum(&self) -> PlutusData {
        self.data.clone()
    }

    pub fn ex_units(&self) -> ExUnits {
        self.ex_units.clone()
    }

    pub fn new(data: &PlutusData, ex_units: &ExUnits) -> Self {
        Self {
            data: data.clone(),
            ex_units: ex_units.clone(),
        }
    }
}

#[derive(Clone, Debug)]
enum UntaggedRedeemerPlaceholder {
    JustData(PlutusData),
    Full(UntaggedRedeemer)
}

/// Possible errors during conversion from bytes
#[derive(Debug)]
pub enum MissingExunitError {
    Key((RedeemerTag, usize, String)),
}

impl std::fmt::Display for MissingExunitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            MissingExunitError::Key((tag, index, key)) => write!(f, "Missing exunit for {:?} with <key, index> values of <{:?}, {}>", tag, index, key)
        }
    }
}

/// In order to calculate the index from the sorted set, "add_*" methods in this builder
/// must be called along with the "add_*" methods in transaction builder.
#[derive(Clone, Default, Debug)]
pub struct RedeemerSetBuilder {
    // the set of inputs is an ordered set (according to the order defined on the type TxIn) -
    // this also is the order in which the elements of the set are indexed (lex order on the pair of TxId and Ix).
    // All inputs of a transaction are included in the set being indexed (not just the ones that point to a Plutus script UTxO)
    spend: BTreeMap<TransactionInput, UntaggedRedeemerPlaceholder>,

    // the set of policy IDs is ordered according to the order defined on PolicyID (lex).
    // The index of a PolicyID in this set of policy IDs is computed according to this order.
    // Note that at the use site, the set of policy IDs passed to indexof is the (unfiltered)
    // domain of the Value map in the mint field of the transaction.
    mint: BTreeMap<PolicyID, UntaggedRedeemerPlaceholder>,

    // the index of a reward account ract in the reward withdrawals map is the index of ract as a key in the (unfiltered) map.
    // The keys of the Wdrl map are arranged in the order defined on the RewardAcnt type, which is a lexicographical (abbrv. lex)
    // order on the pair of the Network and the Credential.
    reward: BTreeMap<RewardAddress, UntaggedRedeemerPlaceholder>,

    // certificates in the DCert list are indexed in the order in which they arranged in the (full, unfiltered)
    // list of certificates inside the transaction
    cert: Vec<UntaggedRedeemerPlaceholder>,
}

impl RedeemerSetBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn is_empty(&self) -> bool {
        self.spend.is_empty() && self.mint.is_empty() && self.reward.is_empty() && self.cert.is_empty()
    }

    /// note: will override existing value if called twice with the same key
    pub fn update_ex_units(&mut self, key: &RedeemerWitnessKey, ex_units: &ExUnits) {
        let replace_placeholder = |entry: &mut UntaggedRedeemerPlaceholder| match entry {
            UntaggedRedeemerPlaceholder::JustData(data) => UntaggedRedeemerPlaceholder::Full(UntaggedRedeemer::new(data, ex_units)),
            UntaggedRedeemerPlaceholder::Full(untagged_redeemer) => UntaggedRedeemerPlaceholder::Full(UntaggedRedeemer::new(&untagged_redeemer.data, ex_units)),
        };
        match key.tag().kind() {
            RedeemerTagKind::Spend => {
                let entry = self.spend.iter_mut().nth(u64::from(key.index()) as usize).unwrap();
                *entry.1 = replace_placeholder(entry.1)
            },
            RedeemerTagKind::Mint => {
                let entry = self.mint.iter_mut().nth(u64::from(key.index()) as usize).unwrap();
                *entry.1 = replace_placeholder(entry.1)
            },
            RedeemerTagKind::Cert => {
                let entry = self.cert.iter_mut().nth(u64::from(key.index()) as usize).unwrap();
                *entry = replace_placeholder(entry)
            },
            RedeemerTagKind::Reward => {
                let entry = self.reward.iter_mut().nth(u64::from(key.index()) as usize).unwrap();
                *entry.1 = replace_placeholder(entry.1)
            },
        };
    }

    pub fn add_spend(&mut self, result: &InputBuilderResult) {
        let plutus_data = {
            result.aggregate_witness.as_ref().and_then(|data| data.plutus_data())
        };
        if let Some(data) = plutus_data {
            self.spend.insert(result.input.clone(), UntaggedRedeemerPlaceholder::JustData(data));
        }
    }

    pub fn add_mint(&mut self, result: &MintBuilderResult) {
        let plutus_data = {
            result.aggregate_witness.as_ref().and_then(|data| data.plutus_data())
        };
        if let Some(data) = plutus_data {
            self.mint.insert(result.policy_id.clone(), UntaggedRedeemerPlaceholder::JustData(data));
        }
    }

    pub fn add_reward(&mut self, result: &WithdrawalBuilderResult) {
        let plutus_data = {
            result.aggregate_witness.as_ref().and_then(|data| data.plutus_data())
        };
        if let Some(data) = plutus_data {
            self.reward.insert(result.address.clone(), UntaggedRedeemerPlaceholder::JustData(data));
        }
    }

    pub fn add_cert(&mut self, result: &CertificateBuilderResult) {
        let plutus_data = {
            result.aggregate_witness.as_ref().and_then(|data| data.plutus_data())
        };
        if let Some(data) = plutus_data {
            self.cert.push(UntaggedRedeemerPlaceholder::JustData(data));
        }
    }

    pub fn build(&self, default_to_dummy_exunits: bool) -> Result<Redeemers, MissingExunitError> {
        let mut redeemers = Vec::new();

        self.remove_placeholders_and_tag(
            &mut redeemers,
            &RedeemerTag::new_spend(),
            &mut self.spend.iter(),
            default_to_dummy_exunits
        )?;
        self.remove_placeholders_and_tag(
            &mut redeemers,
            &RedeemerTag::new_mint(),
            &mut self.mint.iter(),
            default_to_dummy_exunits
        )?;
        self.remove_placeholders_and_tag(
            &mut redeemers,
            &RedeemerTag::new_reward(),
            &mut self.reward.iter(),
            default_to_dummy_exunits
        )?;
        self.remove_placeholders_and_tag(
            &mut redeemers,
            &RedeemerTag::new_cert(),
            &mut self.cert.iter().map(|entry| (&(), entry)),
            default_to_dummy_exunits
        )?;

        Ok(Redeemers(redeemers))
    }

    fn remove_placeholders_and_tag<'a, K: Debug + Clone>(
        &self, redeemers: &mut Vec<Redeemer>,
        tag: &RedeemerTag,
        entries: &mut dyn Iterator<Item = (&'a K, &'a UntaggedRedeemerPlaceholder)>,
        default_to_dummy_exunits: bool
    ) -> Result<(), MissingExunitError> {
        let mut result = vec![];
        for (i, entry) in entries.enumerate() {
            let key = (tag, i, entry.0);

            let redeemer = match entry.1 {
                UntaggedRedeemerPlaceholder::JustData(data) => {
                    if !default_to_dummy_exunits {
                        Err(MissingExunitError::Key((key.0.clone(), key.1, format!("{:?}", key.2))))
                    } else {
                        Ok(UntaggedRedeemer::new(data, &ExUnits::dummy()))
                    }
                },
                UntaggedRedeemerPlaceholder::Full(untagged_redeemer) => Ok(untagged_redeemer.clone())
            }?;
            result.push(redeemer);
        }
        redeemers.append(&mut Self::tag_redeemer(
            tag,
            &result
        ));
        Ok(())
    }

    fn tag_redeemer(tag: &RedeemerTag, untagged_redeemers: &[UntaggedRedeemer]) -> Vec<Redeemer> {
        let mut result = Vec::new();

        for (index, value) in untagged_redeemers.iter().enumerate() {
            let redeemer = {
                let index = index as u64;
                Redeemer::new(tag, &index.into(), &value.data, &value.ex_units)
            };
            result.push(redeemer);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use crate::builders::witness_builder::{PartialPlutusWitness, InputAggregateWitnessData, RequiredWitnessSet};

    use super::*;

    fn fake_raw_key_public(id: u8) -> PublicKey {
        PublicKey::from_bytes(
            &[id, 118, 57, 154, 33, 13, 232, 114, 14, 159, 168, 148, 228, 94, 65, 226, 154, 181, 37, 227, 11, 196, 2, 128, 28, 7, 98, 80, 209, 88, 91, 205]
        ).unwrap()
    }

    #[test]
    fn test_redeemer_set_builder() {
        let mut builder = RedeemerSetBuilder::new();

        let data = {
            let witness = {
                let script = PlutusScriptEnum::from_v1(&PlutusV1Script::new(vec![0]));
                PartialPlutusWitness::new(&PlutusScript(script), &PlutusData::new_integer(&0u64.into()))
            };
            let missing_signers = {
                let key = fake_raw_key_public(0);
                let mut missing_signers = Ed25519KeyHashes::new();
                missing_signers.add(&key.hash());
                missing_signers
            };
            InputAggregateWitnessData::PlutusScript(witness, missing_signers, None)
        };

        let address = Address::from_bech32(&"addr1qxeqxcja25k8q05evyngf4f88xn89asl54x2zg3ephgj26ndyt5qk02xmmras5pe9jz2c7tc93wu4c96rqwvg6e2v50qlpmx70").unwrap();

        let input_result = InputBuilderResult {
            input: TransactionInput { transaction_id: TransactionHash([1; 32]), index: 1u64.into() },
            utxo_info: TransactionOutput { address: address.clone(), amount: Value::zero(), datum_option: None, script_ref: None },
            aggregate_witness: None,
            required_wits: RequiredWitnessSet::new(),
        };

        builder.add_spend(&input_result);

        let input_result = InputBuilderResult {
            input: TransactionInput { transaction_id: TransactionHash([1; 32]), index: 0u64.into() },
            utxo_info: TransactionOutput { address: address.clone(), amount: Value::zero(), datum_option: None, script_ref: None },
            aggregate_witness: None,
            required_wits: RequiredWitnessSet::new(),
        };

        builder.add_spend(&input_result);

        let input_result = InputBuilderResult {
            input: TransactionInput { transaction_id: TransactionHash([0; 32]), index: 0u64.into() },
            utxo_info: TransactionOutput { address: address.clone(), amount: Value::zero(), datum_option: None, script_ref: None },
            aggregate_witness: Some(data.clone()),
            required_wits: RequiredWitnessSet::new(),
        };

        builder.add_spend(&input_result);

        builder.update_ex_units(&RedeemerWitnessKey::new(
            &RedeemerTag::new_spend(),
            &BigNum::from(0),
        ), &ExUnits::new(&to_bignum(10), &to_bignum(10)));

        let redeemers = builder.build(false).unwrap();

        assert_eq!(redeemers.len(), 1);

        let spend_redeemer = &redeemers.0[0];

        assert_eq!(spend_redeemer.tag(), RedeemerTag::new_spend());
        assert_eq!(spend_redeemer.index(), BigNum::from(0u64));
    }
}