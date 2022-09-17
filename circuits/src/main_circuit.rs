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

pub struct TestCircuit<E: Engine> {
    pub a: Option<E::Fr>,
    pub b: Option<E::Fr>
}

impl<E: Engine> Circuit<E> for TestCircuit<E> {
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

        let a = Num::Variable(AllocatedNum::alloc_input(cs, || Ok(*self.a.get()?))?);
        let b = Num::Variable(AllocatedNum::alloc_input(cs, || Ok(*self.b.get()?))?);

        let bytes_a = get_bytes(&self.a);
        let bytes_b = get_bytes(&self.b);

        let mut allocated_bytes_a = vec![];
        let mut allocated_bytes_b = vec![];
        for byte in bytes_a.iter() {
            // allocated_bytes_a.push(Byte::from_u8_witness(cs, *byte)?);
            let inner = Num::alloc(
                cs,
                Some(E::Fr::from_str(&format!("{}", byte.unwrap())).unwrap())
            )?;

            let table = cs.get_table(&range_table_name)?;
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



            allocated_bytes_a.push(Byte{inner});
        }
        for byte in bytes_b.iter() {
            // allocated_bytes_b.push(Byte::from_u8_witness(cs, *byte)?)
            allocated_bytes_b.push(
                Byte{
                    inner: Num::alloc(
                        cs,
                        Some(E::Fr::from_str(&format!("{}", byte.unwrap())).unwrap())
                    )?
                }
            );
        }

        for (a, b) in allocated_bytes_a.iter().zip(
            allocated_bytes_b.iter().rev()
        ) {
            a.inner.enforce_equal(cs, &b.inner)?;
        }

        let mut a_bytes_sum = Num::zero();
        let coeff = Num::Constant(E::Fr::from_str("256").unwrap());
        for byte in allocated_bytes_a.iter().rev() {
            a_bytes_sum = a_bytes_sum.mul(cs, &coeff)?;
            a_bytes_sum = a_bytes_sum.add(cs, &byte.inner)?;
        }
        a_bytes_sum.enforce_equal(cs, &a)?;

        let mut b_bytes_sum = Num::zero();
        let coeff = Num::Constant(E::Fr::from_str("256").unwrap());
        for byte in allocated_bytes_b.iter().rev() {
            b_bytes_sum = b_bytes_sum.mul(cs, &coeff)?;
            b_bytes_sum = b_bytes_sum.add(cs, &byte.inner)?;
        }
        b_bytes_sum.enforce_equal(cs, &b)?;

        Ok(())
    }
}

fn get_bytes<F: PrimeField>(number: &Option<F>) -> [Option<u8>; 32]{
    let mut result = [None; 32];

    if let Some(number) = number {
        for (i, part) in number.into_repr().as_ref().iter().enumerate() {
            for (j, byte) in part.to_le_bytes().iter().enumerate() {
                result[i*8 + j] = Some(*byte);
            }
        }
    }

    result
}