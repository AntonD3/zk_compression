use franklin_crypto::bellman::{
    PrimeField, Field,
    compact_bn256::{Bn256, Fr, FrRepr},
    kate_commitment::{Crs, CrsForMonomialForm},
    plonk::{
        // better_better_cs::cs::{Circuit, SelectorOptimizedWidth4MainGateWithDNext},
        better_better_cs::cs::{Circuit, Width4MainGateWithDNext},
        better_better_cs::gates::selector_optimized_with_d_next::*,

        better_better_cs::{
            cs::{PlonkCsWidth4WithNextStepParams, ProvingAssembly, SetupAssembly, TrivialAssembly},
            setup::VerificationKey,
            setup::SetupPrecomputations,
            verifier::verify,
        },
        commitments::transcript::{keccak_transcript::RollingKeccakTranscript, Prng},
    },
    worker::Worker,
};
use compression::StorageTransition;
use compression::sha3;
use compression::sha3::Digest;

pub(crate) mod utils;
mod main_circuit;

use crate::main_circuit::CompressionCircuit;

fn main() {
    let transitions = vec![StorageTransition {
        address: [221, 31, 123, 46, 34, 67, 213, 90, 55, 0, 12, 54, 222, 56, 77, 0, 132, 12, 1, 5],
        key: [31, 8, 32, 23, 2, 65, 222, 1, 34, 0, 6, 0, 234, 0, 0, 243, 0, 0, 22, 0, 0, 0, 234, 0, 122, 65, 33, 0, 4, 0, 4, 11],
        value: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 222, 131, 5],
        meta: None
    }];

    let data = StorageTransition::into_bytes(transitions.clone());
    let compressed_data = StorageTransition::compress(transitions);
    let mut circuit = CompressionCircuit::<Bn256>{
        data: data.clone().into_iter().map(|byte| Some(byte)).collect(),
        compressed_data: compressed_data.clone().into_iter().map(|byte| Some(byte)).collect(),
        data_hash: sha3::Keccak256::digest(
            data.as_slice()
        ).as_slice().to_vec().into_iter().map(|byte| Some(byte)).collect::<Vec<Option<u8>>>(),
        compressed_data_hash: sha3::Keccak256::digest(
            compressed_data.as_slice()
        ).as_slice().to_vec().into_iter().map(|byte| Some(byte)).collect::<Vec<Option<u8>>>(),
        compressed_data_len: Some(Fr::from_repr(FrRepr([compressed_data.len() as u64, 0, 0, 0])).unwrap()),
    };

    let old_worker = Worker::new();

    let mut assembly = TrivialAssembly::<
        Bn256,
        PlonkCsWidth4WithNextStepParams,
        Width4MainGateWithDNext,
    >::new();

    circuit.synthesize(&mut assembly).expect("must work");
    assert!(assembly.is_satisfied());

    assembly.finalize();

    let domain_size = assembly.n().next_power_of_two();

    let crs_mons = Crs::<Bn256, CrsForMonomialForm>::crs_42(domain_size, &old_worker);

    let setup = assembly
        .create_setup::<CompressionCircuit<Bn256>>(&old_worker)
        .unwrap();


    let proof = assembly.clone()
        .create_proof::<CompressionCircuit<Bn256>, RollingKeccakTranscript<Fr>>(
            &old_worker,
            &setup,
            &crs_mons,
            None,
        )
        .unwrap();

    let vk = VerificationKey::from_setup(&setup, &old_worker, &crs_mons).unwrap();

    let valid =
        verify::<Bn256, CompressionCircuit<Bn256>, RollingKeccakTranscript<Fr>>(&vk, &proof, None)
            .unwrap();

    if valid {
        println!("Proof is valid!😎");
    }
}
