use std::io::{BufRead, Write};

use schemars::JsonSchema;
use crate::chain_crypto;
use crate::chain_crypto::Ed25519;
use crate::chain_crypto::Ed25519Bip32;
use crate::JsError;

use crate::crypto::Bip32PublicKey;
use crate::crypto::PublicKey;
use crate::ledger::common::binary::*;
use crate::ledger::common::value::Coin;

// This library was code-generated using an experimental CDDL to rust tool:
// https://github.com/Emurgo/cddl-codegen

use cbor_event::{self, de::Deserializer, se::{Serialize, Serializer}};

use cbor_event::Type as CBORType;

use cbor_event::Special as CBORSpecial;
use wasm_bindgen::JsValue;

use crate::{chain_crypto::hash::Blake2b224, crypto::impl_hash_type};
use crate::error::{DeserializeError, DeserializeFailure};
use bech32::ToBase32;

pub use self::crc32::Crc32;

mod serialization;
mod utils;
mod crc32;
mod base58;


#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash, Copy, serde::Serialize, serde::Deserialize, JsonSchema)]
pub struct ProtocolMagic(pub(crate) u32);


#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, JsonSchema)]
pub struct HDAddressPayload(pub(crate) Vec<u8>);

to_from_bytes!(HDAddressPayload);

impl_hash_type!(StakeholderId, 28);
impl_hash_type!(AddressId, 28);



#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, JsonSchema)]
pub struct AddrAttributes {
    stake_distribution: Option<StakeDistribution>,
    derivation_path: Option<HDAddressPayload>,
    protocol_magic: Option<ProtocolMagic>,
}

to_from_bytes!(AddrAttributes);
to_from_json!(AddrAttributes);



impl AddrAttributes {
    pub fn set_stake_distribution(&mut self, stake_distribution: &StakeDistribution) {
        self.stake_distribution = Some(stake_distribution.clone())
    }

    pub fn stake_distribution(&self) -> Option<StakeDistribution> {
        self.stake_distribution.clone()
    }

    pub fn set_derivation_path(&mut self, derivation_path: HDAddressPayload) {
        self.derivation_path = Some(derivation_path)
    }

    pub fn derivation_path(&self) -> Option<HDAddressPayload> {
        self.derivation_path.clone()
    }

    pub fn set_protocol_magic(&mut self, protocol_magic: ProtocolMagic) {
        self.protocol_magic = Some(protocol_magic)
    }

    pub fn protocol_magic(&self) -> Option<ProtocolMagic> {
        self.protocol_magic.clone()
    }

    pub fn new() -> Self {
        Self {
            stake_distribution: None,
            derivation_path: None,
            protocol_magic: None,
        }
    }
}



#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, JsonSchema)]
pub enum StakeDistributionKind {
    BootstrapEraDistr,
    SingleKeyDistr,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, JsonSchema)]
enum StakeDistributionEnum {
    BootstrapEraDistr(BootstrapEraDistr),
    SingleKeyDistr(SingleKeyDistr),
}



#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, JsonSchema)]
pub struct StakeDistribution(StakeDistributionEnum);

to_from_bytes!(StakeDistribution);
to_from_json!(StakeDistribution);



impl StakeDistribution {
    pub fn new_bootstrap_era_distr() -> Self {
        Self(StakeDistributionEnum::BootstrapEraDistr(BootstrapEraDistr::new()))
    }

    pub fn new_single_key_distr(stakeholder_id: &StakeholderId) -> Self {
        Self(StakeDistributionEnum::SingleKeyDistr(SingleKeyDistr::new(stakeholder_id)))
    }

    pub fn kind(&self) -> StakeDistributionKind {
        match &self.0 {
            StakeDistributionEnum::BootstrapEraDistr(_) => StakeDistributionKind::BootstrapEraDistr,
            StakeDistributionEnum::SingleKeyDistr(_) => StakeDistributionKind::SingleKeyDistr,
        }
    }

    pub fn as_bootstrap_era_distr(&self) -> Option<BootstrapEraDistr> {
        match &self.0 {
            StakeDistributionEnum::BootstrapEraDistr(x) => Some(x.clone()),
            _ => None,
        }
    }

    pub fn as_single_key_distr(&self) -> Option<SingleKeyDistr> {
        match &self.0 {
            StakeDistributionEnum::SingleKeyDistr(x) => Some(x.clone()),
            _ => None,
        }
    }
}



#[derive(Clone, Debug, Eq, Ord, Hash, PartialEq, PartialOrd)]
pub struct ByronAddress {
    addr: Vec<u8>,
    crc32: Crc32,
}

to_from_bytes!(ByronAddress);
to_from_json!(ByronAddress);



impl ByronAddress {
    pub fn addr(&self) -> Vec<u8> {
        self.addr.clone()
    }

    pub fn crc32(&self) -> Crc32 {
        self.crc32.clone()
    }

    pub fn checksum_from_bytes(addr: Vec<u8>) -> ByronAddress {
        let crc32 = crate::byron::crc32::crc32(&addr);
        
        Self {
            addr: addr,
            crc32: crc32.into(),
        }
    }

    pub fn new(addr: Vec<u8>, crc32: &Crc32) -> Result<ByronAddress, JsError> {
        let found_crc = crate::byron::crc32::crc32(&addr);

        if crc32.0 != found_crc as u32 {
            return Err(JsError::from_str(&format!(
                "Invalid CRC32: 0x{:x} but expected 0x{:x}",
                crc32.0, found_crc
            )));
        }
        Ok(Self {
            addr: addr,
            crc32: crc32.clone(),
        })
    }
}

impl JsonSchema for ByronAddress {
    fn schema_name() -> String { String::from("ByronAddress") }
    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema { String::json_schema(gen) }
    fn is_referenceable() -> bool { String::is_referenceable() }
}



#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, JsonSchema)]
pub struct AddressContent {
    address_id: AddressId,
    addr_attr: AddrAttributes,
    addr_type: ByronAddrType,
}

to_from_bytes!(AddressContent);
to_from_json!(AddressContent);



impl AddressContent {
    pub fn address_id(&self) -> AddressId {
        self.address_id.clone()
    }

    pub fn addr_attr(&self) -> AddrAttributes {
        self.addr_attr.clone()
    }

    pub fn addr_type(&self) -> ByronAddrType {
        self.addr_type.clone()
    }

    pub fn new(address_id: &AddressId, addr_attr: &AddrAttributes, addr_type: &ByronAddrType) -> Self {
        Self {
            address_id: address_id.clone(),
            addr_attr: addr_attr.clone(),
            addr_type: addr_type.clone(),
        }
    }
}



#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, JsonSchema)]
pub enum AddrtypeKind {
    ATPubKey,
    ATScript,
    ATRedeem,
}

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, JsonSchema)]
pub enum AddrTypeEnum {
    ATPubKey,
    ATScript,
    ATRedeem,
}



#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, JsonSchema)]
pub struct ByronAddrType(AddrTypeEnum);

to_from_bytes!(ByronAddrType);
to_from_json!(ByronAddrType);



impl ByronAddrType {
    pub fn new_ATPubKey() -> Self {
        Self(AddrTypeEnum::ATPubKey)
    }

    pub fn new_ATScript() -> Self {
        Self(AddrTypeEnum::ATScript)
    }

    pub fn new_ATRedeem() -> Self {
        Self(AddrTypeEnum::ATRedeem)
    }

    pub fn kind(&self) -> AddrtypeKind {
        match &self.0 {
            AddrTypeEnum::ATPubKey => AddrtypeKind::ATPubKey,
            AddrTypeEnum::ATScript => AddrtypeKind::ATScript,
            AddrTypeEnum::ATRedeem => AddrtypeKind::ATRedeem,
        }
    }
}



#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, JsonSchema)]
pub struct BootstrapEraDistr {
}

to_from_bytes!(BootstrapEraDistr);
to_from_json!(BootstrapEraDistr);



impl BootstrapEraDistr {
    pub fn new() -> Self {
        Self {
        }
    }
}



#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, JsonSchema)]
pub struct SingleKeyDistr {
    stakeholder_id: StakeholderId,
}

to_from_bytes!(SingleKeyDistr);
to_from_json!(SingleKeyDistr);



impl SingleKeyDistr {
    pub fn stakeholder_id(&self) -> StakeholderId {
        self.stakeholder_id.clone()
    }

    pub fn new(stakeholder_id: &StakeholderId) -> Self {
        Self {
            stakeholder_id: stakeholder_id.clone(),
        }
    }
}


#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, JsonSchema)]
pub struct ByronScript(pub(crate) [u8; 32]); // TODO: not sure what this type is supposed to represent. Is it a hash?

to_from_bytes!(ByronScript);



#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, JsonSchema)]
pub enum SpendingDataKind {
    SpendingDataPubKeyASD,
    SpendingDataScriptASD,
    SpendingDataRedeemASD,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, JsonSchema)]
enum SpendingDataEnum {
    SpendingDataPubKeyASD(SpendingDataPubKeyASD),
    SpendingDataScriptASD(SpendingDataScriptASD),
    SpendingDataRedeemASD(SpendingDataRedeemASD),
}



#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, JsonSchema)]
pub struct SpendingData(SpendingDataEnum);

to_from_bytes!(SpendingData);
to_from_json!(SpendingData);



impl SpendingData {
    pub fn new_spending_data_pub_key(public_ed25519_bip32: &Bip32PublicKey) -> Self {
        Self(SpendingDataEnum::SpendingDataPubKeyASD(SpendingDataPubKeyASD::new(public_ed25519_bip32)))
    }

    pub fn new_spending_data_script(script: &ByronScript) -> Self {
        Self(SpendingDataEnum::SpendingDataScriptASD(SpendingDataScriptASD::new(script)))
    }

    pub fn new_spending_data_redeem(public_ed25519: &PublicKey) -> Self {
        Self(SpendingDataEnum::SpendingDataRedeemASD(SpendingDataRedeemASD::new(public_ed25519)))
    }

    pub fn kind(&self) -> SpendingDataKind {
        match &self.0 {
            SpendingDataEnum::SpendingDataPubKeyASD(_) => SpendingDataKind::SpendingDataPubKeyASD,
            SpendingDataEnum::SpendingDataScriptASD(_) => SpendingDataKind::SpendingDataScriptASD,
            SpendingDataEnum::SpendingDataRedeemASD(_) => SpendingDataKind::SpendingDataRedeemASD,
        }
    }

    pub fn as_spending_data_pub_key(&self) -> Option<SpendingDataPubKeyASD> {
        match &self.0 {
            SpendingDataEnum::SpendingDataPubKeyASD(x) => Some(x.clone()),
            _ => None,
        }
    }

    pub fn as_spending_data_script(&self) -> Option<SpendingDataScriptASD> {
        match &self.0 {
            SpendingDataEnum::SpendingDataScriptASD(x) => Some(x.clone()),
            _ => None,
        }
    }

    pub fn as_spending_data_redeem(&self) -> Option<SpendingDataRedeemASD> {
        match &self.0 {
            SpendingDataEnum::SpendingDataRedeemASD(x) => Some(x.clone()),
            _ => None,
        }
    }
}



#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, JsonSchema)]
pub struct SpendingDataPubKeyASD {
    public_ed25519_bip32: chain_crypto::PublicKey<Ed25519Bip32>,
}

to_from_bytes!(SpendingDataPubKeyASD);
to_from_json!(SpendingDataPubKeyASD);



impl SpendingDataPubKeyASD {
    pub fn public_ed25519_bip32(&self) -> Bip32PublicKey {
        Bip32PublicKey(self.public_ed25519_bip32.clone())
    }

    pub fn new(public_ed25519_bip32: &Bip32PublicKey) -> Self {
        Self {
            public_ed25519_bip32: public_ed25519_bip32.0.clone(),
        }
    }
}



#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, JsonSchema)]
pub struct SpendingDataRedeemASD {
    public_ed25519: chain_crypto::PublicKey<Ed25519>,
}

to_from_bytes!(SpendingDataRedeemASD);
to_from_json!(SpendingDataRedeemASD);



impl SpendingDataRedeemASD {
    pub fn public_ed25519(&self) -> PublicKey {
        PublicKey(self.public_ed25519.clone())
    }

    pub fn new(public_ed25519: &PublicKey) -> Self {
        Self {
            public_ed25519: public_ed25519.clone().0,
        }
    }
}



#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, JsonSchema)]
pub struct SpendingDataScriptASD {
    script: ByronScript,
}

to_from_bytes!(SpendingDataScriptASD);
to_from_json!(SpendingDataScriptASD);




impl SpendingDataScriptASD {
    pub fn script(&self) -> ByronScript {
        self.script.clone()
    }

    pub fn new(script: &ByronScript) -> Self {
        Self {
            script: script.clone(),
        }
    }
}

to_from_bytes!(ByronTxout);
to_from_json!(ByronTxout);




#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize, JsonSchema)]
pub struct ByronTxout {
    address: ByronAddress,
    amount: Coin,
}


impl ByronTxout {
    pub fn address(&self) -> ByronAddress {
        self.address.clone()
    }

    pub fn amount(&self) -> Coin {
        self.amount.clone()
    }

    pub fn new(address: &ByronAddress, amount: &Coin) -> Self {
        Self {
            address: address.clone(),
            amount: amount.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tx_output_decoding() {
        let tx_out = ByronTxout::from_bytes(
            hex::decode("8282d818582183581cc6eb29e2cbb7b616b28c83da505a08253c33ec371319261ad93e558ca0001a1102942c1b00000005f817ddfc").unwrap()
        ).unwrap();
        assert_eq!(tx_out.address().to_base58(), "Ae2tdPwUPEZGexC4LXgsr1BJ1PppXk71zpuRkboFopVpSDcykQvpyYJXCJf");
        assert!(tx_out.to_json().unwrap().contains("Ae2tdPwUPEZGexC4LXgsr1BJ1PppXk71zpuRkboFopVpSDcykQvpyYJXCJf"));
    }
}
