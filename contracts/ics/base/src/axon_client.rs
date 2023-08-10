extern crate alloc;
use alloc::vec::Vec;
use axon_tools_riscv::types::{AxonBlock, Proof as AxonProof, Validator};
use axon_types::metadata::Metadata;
use ckb_ics_axon::handler::Client;
use ckb_ics_axon::object::{Object, VerifyError};
use ckb_ics_axon::proof::ObjectProof;
use ckb_ics_axon::verify_message;
use molecule::prelude::Entity;

pub struct AxonClient {
    pub id: [u8; 32],
    pub validators: Vec<Validator>,
}

impl Client for AxonClient {
    fn verify_object<O: Object>(&mut self, obj: O, proof: ObjectProof) -> Result<(), VerifyError> {
        #[cfg(feature = "debugging")]
        if self.validators.is_empty() {
            return Ok(());
        }
        let block = rlp::decode::<AxonBlock>(&proof.block).unwrap();

        verify_message(
            block.header.receipts_root,
            proof.receipt,
            obj,
            proof.receipt_proof,
        )?;

        let axon_proof = rlp::decode::<AxonProof>(&proof.axon_proof).unwrap();

        axon_tools_riscv::verify_proof(block, proof.state_root, &mut self.validators, axon_proof)
            .map_err(|_| VerifyError::InvalidReceiptProof)
    }

    fn client_id(&self) -> &[u8] {
        self.id.as_slice()
    }
}

impl AxonClient {
    pub fn new(id: [u8; 32], slice: &[u8]) -> Result<Self, Error> {
        let metadata = Metadata::from_slice(slice).map_err(|_| Error::MetadataSerde)?;
        let validators = metadata.validators();
        let mut client_validators: Vec<Validator> = Vec::new();
        for i in 0..validators.len() {
            let v = validators.get(i).unwrap();
            let bls_pub_key = v.bls_pub_key().raw_data().to_vec();
            let address_data = v.address().raw_data();
            let address: [u8; 20] = address_data
                .as_ref()
                .try_into()
                .map_err(|_| Error::MetadataSerde)?;
            let height: [u8; 4] = v.propose_weight().as_slice().try_into().unwrap();
            let weight: [u8; 4] = v.vote_weight().as_slice().try_into().unwrap();
            let validator = Validator {
                bls_pub_key: bls_pub_key.into(),
                address: address.into(),
                propose_weight: u32::from_le_bytes(height),
                vote_weight: u32::from_le_bytes(weight),
            };
            client_validators.push(validator);
        }
        Ok(AxonClient {
            id,
            validators: client_validators,
        })
    }
}

pub enum Error {
    MetadataSerde,
}
