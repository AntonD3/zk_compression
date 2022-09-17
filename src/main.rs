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

mod main_circuit;
use crate::main_circuit::TestCircuit;

fn main() {
    // let mut circuit = TestCircuit::<Bn256>{
    //     a: Some(Fr::from_repr(FrRepr([0, 0, 1 << 7, 0])).unwrap()),
    //     b: Some(Fr::from_repr(FrRepr([0, 1 << 63, 0, 0])).unwrap()),
    // };

    let mut circuit = TestCircuit::<Bn256>{
        a: Some(Fr::from_repr(FrRepr([0, (((1<<63)-1)<<1)+1, 0, 0])).unwrap()),
        b: Some(Fr::from_repr(FrRepr([0, 0, (((1<<63)-1)<<1)+1, 0])).unwrap()),
    };

    dbg!(circuit.a.as_ref().unwrap());
    dbg!(circuit.b.as_ref().unwrap());

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
        .create_setup::<TestCircuit<Bn256>>(&old_worker)
        .unwrap();


    let proof = assembly.clone()
        .create_proof::<TestCircuit<Bn256>, RollingKeccakTranscript<Fr>>(
            &old_worker,
            &setup,
            &crs_mons,
            None,
        )
        .unwrap();

    let vk = VerificationKey::from_setup(&setup, &old_worker, &crs_mons).unwrap();

    let valid =
        verify::<Bn256, TestCircuit<Bn256>, RollingKeccakTranscript<Fr>>(&vk, &proof, None)
            .unwrap();
    assert!(valid);
}
