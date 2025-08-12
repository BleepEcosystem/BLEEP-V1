//! BLEEP Zero-Knowledge Proofs (ZKP) module

use ark_ff::Field;
use ark_groth16::{Groth16, Proof, ProvingKey, VerifyingKey, create_random_proof, verify_proof};
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_bls12_381::{Bls12_381, Fr};
use rand::thread_rng;

/// Example circuit for demonstration
#[derive(Clone)]
pub struct ExampleCircuit<F: Field> {
    pub a: Option<F>,
    pub b: Option<F>,
}

impl<F: Field> ConstraintSynthesizer<F> for ExampleCircuit<F> {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        let a = cs.new_witness_variable(|| self.a.ok_or(SynthesisError::AssignmentMissing))?;
        let b = cs.new_witness_variable(|| self.b.ok_or(SynthesisError::AssignmentMissing))?;
        let c = cs.new_input_variable(|| {
            let a = self.a.ok_or(SynthesisError::AssignmentMissing)?;
            let b = self.b.ok_or(SynthesisError::AssignmentMissing)?;
            Ok(a * b)
        })?;
        cs.enforce_constraint(a.into(), b.into(), c.into())?;
        Ok(())
    }
}

/// Generate a Groth16 proof for the example circuit
default pub fn generate_example_proof(a: Fr, b: Fr) -> (Proof<Bls12_381>, VerifyingKey<Bls12_381>, Fr) {
    let circuit = ExampleCircuit { a: Some(a), b: Some(b) };
    let mut rng = thread_rng();
    let (pk, vk) = Groth16::<Bls12_381>::setup(circuit.clone(), &mut rng).unwrap();
    let public_input = a * b;
    let proof = create_random_proof(circuit, &pk, &mut rng).unwrap();
    (proof, vk, public_input)
}

/// Verify a Groth16 proof for the example circuit
default pub fn verify_example_proof(proof: &Proof<Bls12_381>, vk: &VerifyingKey<Bls12_381>, public_input: Fr) -> bool {
    verify_proof(vk, proof, &[public_input]).unwrap_or(false)
}
