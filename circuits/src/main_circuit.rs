pub use franklin_crypto::{
    bellman::{
        kate_commitment::{Crs, CrsForMonomialForm},
        plonk::better_better_cs::{
            cs::{
                Assembly, Circuit, ConstraintSystem, Gate, GateInternal, LookupTableApplication,
                PlonkCsWidth4WithNextStepAndCustomGatesParams, PolyIdentifier, Setup, Width4MainGateWithDNext,
                TrivialAssembly,
                ArithmeticTerm,
                MainGateTerm,
            },
            proof::Proof,
            setup::VerificationKey,
            verifier,
            gates::selector_optimized_with_d_next::SelectorOptimizedWidth4MainGateWithDNext,
        },
        Engine, Field, PrimeField, ScalarEngine, SynthesisError,
        worker::Worker,
        plonk::commitments::transcript::{keccak_transcript::RollingKeccakTranscript, Transcript},
    },
    plonk::circuit::{
        allocated_num::{AllocatedNum, Num},
        boolean::{AllocatedBit, Boolean},
        byte::Byte,
        custom_rescue_gate::Rescue5CustomGate,
    },
};
use franklin_crypto::plonk::circuit::Assignment;

pub const MAX_COMPRESSED_DATA_SIZE: usize = 100;
pub const MAX_UNCOMPRESSED_DATA_SIZE: usize = 100;
pub const MAX_WORDS: usize = 10;

pub struct CompressionCircuit<E: Engine> {
    pub data: Vec<Option<u8>>,
    pub compressed_data: Vec<Option<u8>>,
    pub data_hash: Vec<Option<u8>>,
    pub compressed_data_hash: Vec<Option<u8>>,
    pub compressed_data_len: Option<E::Fr>,
}

impl<E: Engine> Circuit<E> for CompressionCircuit<E> {
    type MainGate = Width4MainGateWithDNext;

    fn synthesize<CS: ConstraintSystem<E>>(&self, cs: &mut CS) -> Result<(), SynthesisError> {
        let columns = vec![
            PolyIdentifier::VariablesPolynomial(0),
            PolyIdentifier::VariablesPolynomial(1),
            PolyIdentifier::VariablesPolynomial(2),
        ];
        let range_table = LookupTableApplication::new_range_table_of_width_3(8, columns.clone())?;
        let range_table_name = range_table.functional_name();
        cs.add_table(range_table)?;

        let compressed_data_hash_bytes = allocate_and_prove_bytes(&self.compressed_data_hash, cs, range_table_name.as_str(), true);
        let data_hash = allocate_and_prove_bytes(&self.data_hash, cs, range_table_name.as_str(), true);

        let compressed_data_bytes = allocate_and_prove_bytes(&self.compressed_data, cs, range_table_name.as_str(), false);
        let data_bytes = allocate_and_prove_bytes(&self.data, cs, range_table_name.as_str(), false);

        // TODO: prove hashes correctness

        let compressed_data_len = Num::alloc(
            cs,
            self.compressed_data_len
        )?;

        let ptr = cs.get_explicit_zero();

        for word in 0..MAX_WORDS {

        }
        Ok(())
    }
}

fn allocate_and_prove_bytes<E: Engine, CS: ConstraintSystem<E>>(bytes: &Vec<Option<u8>>, cs: &mut CS, range_table_name: &str, alloc_as_inputs: bool) -> Result<Vec<Byte<E>>, SynthesisError> {
    let mut result = Vec::with_capacity(bytes.len());

    for byte in bytes {
        let inner = if alloc_as_inputs {
            Num::Variable(AllocatedNum::alloc_input(cs, || Ok(E::Fr::from_str(&format!("{}", byte.unwrap())).unwrap()))?)
        } else {
            Num::alloc(
                cs,
                Some(E::Fr::from_str(&format!("{}", byte.unwrap())).unwrap())
            )?
        };

        let table = cs.get_table(range_table_name)?;
        let num_keys_and_values = table.width();

        let var_zero = cs.get_explicit_zero()?;
        let dummy = CS::get_dummy_variable();

        let inner_var = inner.get_variable().get_variable();
        let vars = [inner_var, var_zero.clone(), var_zero.clone(), dummy];

        cs.begin_gates_batch_for_step()?;

        cs.allocate_variables_without_gate(
            &vars,
            &[]
        )?;

        cs.apply_single_lookup_gate(&vars[..num_keys_and_values], table)?;
        cs.end_gates_batch_for_step()?;


        result.push(Byte{inner});
    }

    Ok(result)
}
