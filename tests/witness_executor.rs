#![recursion_limit = "256"]

use anyhow::Result;
use plonky2::field::goldilocks_field::GoldilocksField as F;
use plonky2::field::types::Field;
use plonky2::iop::target::Target;
use plonky2::iop::witness::{PartialWitness, WitnessWrite};
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::{CircuitConfig, CircuitData};
use plonky2::plonk::config::PoseidonGoldilocksConfig as C;
use plonky2::plonk::witness_program::{check_against_worklist, WitnessConfig, WitnessFallback};
use plonky2_u32::gadgets::arithmetic_u32::CircuitBuilderU32;
use plonky2_u32::gadgets::range_check::range_check_u32_circuit;

struct Fixture {
    data: CircuitData<F, C, 2>,
    inputs: Vec<Target>,
}

fn fixture_with_addends(addends: usize) -> Fixture {
    let mut builder = CircuitBuilder::<F, 2>::new(CircuitConfig::standard_recursion_config());
    let inputs = builder.add_virtual_u32_targets(5 + addends);
    let product = builder.mul_add_u32(inputs[0], inputs[1], inputs[2]);
    let difference = builder.sub_u32(inputs[0], inputs[1], inputs[3]);
    let sum = builder.add_u32s_with_carry(&inputs[5..], inputs[4]);
    let mut words = inputs[..3].to_vec();
    words.extend_from_slice(&inputs[5..]);
    for chunk in words.chunks(7) {
        range_check_u32_circuit(&mut builder, chunk.to_vec());
    }
    builder.range_check(inputs[3].0, 1);
    builder.range_check(inputs[4].0, 4);
    for output in [
        product.0,
        product.1,
        sum.0,
        sum.1,
        difference.0,
        difference.1,
    ] {
        builder.register_public_input(output.0);
    }
    Fixture {
        data: builder.build::<C>(),
        inputs: inputs.into_iter().map(|t| t.0).collect(),
    }
}

fn witness(f: &Fixture, words: &[u64]) -> Result<PartialWitness<F>> {
    let mut pw = PartialWitness::new();
    for (&target, &value) in f.inputs.iter().zip(words) {
        pw.set_target(target, F::from_canonical_u64(value))?;
    }
    Ok(pw)
}

fn fixture() -> Fixture {
    fixture_with_addends(16)
}

#[test]
fn u32_executor_handles_variable_addend_shapes() -> Result<()> {
    for addends in [0, 2, 3, 7, 15, 16] {
        let f = fixture_with_addends(addends);
        let mut words = vec![u32::MAX as u64, 1, 1, 1, 15];
        words.extend(core::iter::repeat_n(u32::MAX as u64, addends));
        check_against_worklist(&witness(&f, &words)?, &f.data.prover_only, &f.data.common)?;
    }
    Ok(())
}

#[test]
fn u32_executor_matches_all_worklist_values_on_boundaries_and_fresh_inputs() -> Result<()> {
    let f = fixture();
    f.data.witness_program_support()?;
    let max = u32::MAX as u64;
    let mut cases = Vec::new();
    for (x, y, z, borrow, carry) in [
        (0, 0, 0, 0, 0),
        (0, 0, 0, 1, 15),
        (max, max, max, 1, 15),
        (max, 1, 1, 0, 0),
        (1, max, max, 1, 1),
        // Out-of-range values exercise field reduction, truncation, and the
        // exact subtraction threshold; constraints are not checked here.
        (1 << 32, 0, 0, 0, 0),
        (F::NEG_ONE.0, F::NEG_ONE.0, 1, 0, 15),
    ] {
        let mut words = vec![x, y, z, borrow, carry];
        words.extend([max; 16]);
        cases.push(words);
    }
    let mut state = 0x1234_5678_9abc_def0u64;
    for _ in 0..24 {
        let mut words = Vec::new();
        for _ in 0..21 {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            words.push((state >> 32) as u32 as u64);
        }
        words[3] &= 1;
        words[4] &= 15;
        cases.push(words);
    }
    for words in cases {
        check_against_worklist(&witness(&f, &words)?, &f.data.prover_only, &f.data.common)?;
    }
    Ok(())
}

#[test]
fn u32_executor_serialization_strict_proofs_and_conflicting_outputs() -> Result<()> {
    #[cfg(feature = "gpu")]
    assert!(
        plonky2::gpu::context().is_some(),
        "this GPU integration test requires an adapter"
    );
    let f = fixture();
    let seed = witness(&f, &[0; 21])?;
    let program = f.data.compile_witness_program_like(&seed)?;
    let encoded = program.to_bytes();
    let loaded = f.data.witness_program_from_bytes(&encoded)?;
    let mut config = WitnessConfig::from_env()?;
    config.fallback = WitnessFallback::Error;
    let program = loaded.with_execution_config(config.clone())?;
    f.data.install_witness_program(program)?;
    for (x, y, z, borrow, carry) in [
        (0u32, 0u32, 0u32, 1u64, 0u64),
        (u32::MAX, u32::MAX, u32::MAX, 1, 15),
        (17, 23, 42, 0, 3),
    ] {
        let mut words = vec![x as u64, y as u64, z as u64, borrow, carry];
        words.extend([u32::MAX as u64; 16]);
        let proof = f.data.prove(witness(&f, &words)?)?;
        let product = x as u64 * y as u64 + z as u64;
        let sum = 16 * u32::MAX as u64 + carry;
        let difference = x as i64 - y as i64 - borrow as i64;
        let expected = [
            product as u32 as u64,
            product >> 32,
            sum as u32 as u64,
            sum >> 32,
            difference.rem_euclid(1i64 << 32) as u64,
            u64::from(difference < 0),
        ]
        .map(F::from_canonical_u64);
        assert_eq!(proof.public_inputs, expected);
        f.data.verify(proof)?;
    }
    // This input violates a u32 range constraint. Generating matching limbs
    // must never turn it into an accepted proof.
    let mut invalid = [0u64; 21];
    invalid[0] = 1 << 32;
    if let Ok(proof) = f.data.prove(witness(&f, &invalid)?) {
        assert!(f.data.verify(proof).is_err());
    }
    // Supplying a wrong claimed output is a conflicting root; explicitly
    // compile for that shape and require the executor to reject it.
    let mut conflicting = seed;
    conflicting.set_target(f.data.prover_only.public_inputs[0], F::ONE)?;
    let program = f.data.compile_witness_program_like(&conflicting)?;
    f.data
        .install_witness_program(program.with_execution_config(config)?)?;
    assert!(f.data.prove(conflicting).is_err());
    Ok(())
}
