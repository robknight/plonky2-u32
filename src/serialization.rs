use alloc::vec::Vec;
use core::marker::PhantomData;
use plonky2::gadgets::arithmetic::EqualityGenerator;
use plonky2::gadgets::arithmetic_extension::QuotientGeneratorExtension;
use plonky2::gadgets::range_check::LowHighGenerator;
use plonky2::gadgets::split_base::BaseSumGenerator;
use plonky2::gadgets::split_join::{SplitGenerator, WireSplitGenerator};
use plonky2::gates::arithmetic_base::{ArithmeticBaseGenerator, ArithmeticGate};
use plonky2::gates::base_sum::{BaseSplitGenerator, BaseSumGate};
use plonky2::gates::constant::ConstantGate;
use plonky2::gates::coset_interpolation::{CosetInterpolationGate, InterpolationGenerator};
use plonky2::gates::exponentiation::{ExponentiationGate, ExponentiationGenerator};
use plonky2::gates::lookup::{LookupGate, LookupGenerator};
use plonky2::gates::lookup_table::{LookupTableGate, LookupTableGenerator};
use plonky2::gates::noop::NoopGate;
use plonky2::gates::poseidon::{PoseidonGate, PoseidonGenerator};
use plonky2::gates::poseidon_mds::{PoseidonMdsGate, PoseidonMdsGenerator};
use plonky2::gates::public_input::PublicInputGate;
use plonky2::gates::random_access::{RandomAccessGate, RandomAccessGenerator};
use plonky2::gates::reducing::{ReducingGate, ReducingGenerator};
use plonky2::gates::reducing_extension::{
    ReducingExtensionGate, ReducingGenerator as ReducingExtensionGenerator,
};
use plonky2::plonk::config::{AlgebraicHasher, GenericConfig};
use plonky2::read_gate_impl;
use plonky2::recursion::dummy_circuit::DummyProofGenerator;
use plonky2::util::serialization::{GateSerializer, WitnessGeneratorSerializer};
use plonky2::{impl_gate_serializer, impl_generator_serializer};

use crate::gates::add_many_u32::{U32AddManyGate, U32AddManyGenerator};
use crate::gates::arithmetic_u32::{U32ArithmeticGate, U32ArithmeticGenerator};
use crate::gates::comparison::{ComparisonGate, ComparisonGenerator};
use crate::gates::range_check_u32::{U32RangeCheckGate, U32RangeCheckGenerator};
use crate::gates::subtraction_u32::{U32SubtractionGate, U32SubtractionGenerator};
use plonky2::field::extension::Extendable;
use plonky2::gates::arithmetic_extension::{ArithmeticExtensionGate, ArithmeticExtensionGenerator};
use plonky2::gates::multiplication_extension::{MulExtensionGate, MulExtensionGenerator};
use plonky2::hash::hash_types::RichField;
use plonky2::iop::generator::{
    ConstantGenerator, CopyGenerator, NonzeroTestGenerator, RandomValueGenerator,
};
use plonky2::{get_gate_tag_impl, get_generator_tag_impl, read_generator_impl};

use plonky2::util::serialization::{Buffer, IoResult, Read, Write};

use crate::gadgets::arithmetic_u32::U32Target;

pub trait WriteU32 {
    fn write_target_u32(&mut self, x: U32Target) -> IoResult<()>;
}

impl WriteU32 for Vec<u8> {
    #[inline]
    fn write_target_u32(&mut self, x: U32Target) -> IoResult<()> {
        self.write_target(x.0)
    }
}

pub trait ReadU32 {
    fn read_target_u32(&mut self) -> IoResult<U32Target>;
}

impl ReadU32 for Buffer<'_> {
    #[inline]
    fn read_target_u32(&mut self) -> IoResult<U32Target> {
        Ok(U32Target(self.read_target()?))
    }
}

#[derive(Debug, Default)]
pub struct U32GeneratorSerializer<C: GenericConfig<D>, const D: usize> {
    pub _phantom: PhantomData<C>,
}

impl<F, C, const D: usize> WitnessGeneratorSerializer<F, D> for U32GeneratorSerializer<C, D>
where
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F> + 'static,
    C::Hasher: AlgebraicHasher<F>,
{
    impl_generator_serializer! {
        DefaultGeneratorSerializer,
        ArithmeticBaseGenerator<F, D>,
        ArithmeticExtensionGenerator<F, D>,
        BaseSplitGenerator<2>,
        BaseSumGenerator<2>,
        ConstantGenerator<F>,
        CopyGenerator,
        DummyProofGenerator<F, C, D>,
        EqualityGenerator,
        ExponentiationGenerator<F, D>,
        InterpolationGenerator<F, D>,
        LookupGenerator,
        LookupTableGenerator,
        LowHighGenerator,
        MulExtensionGenerator<F, D>,
        NonzeroTestGenerator,
        PoseidonGenerator<F, D>,
        PoseidonMdsGenerator<D>,
        QuotientGeneratorExtension<D>,
        RandomAccessGenerator<F, D>,
        RandomValueGenerator,
        ReducingGenerator<D>,
        ReducingExtensionGenerator<D>,
        SplitGenerator,
        WireSplitGenerator,
        U32ArithmeticGenerator<F,D>,
        U32AddManyGenerator<F,D>,
        U32SubtractionGenerator<F,D>,
        U32RangeCheckGenerator<F,D>,
        ComparisonGenerator<F,D>
    }
}

#[derive(Debug)]
pub struct U32GateSerializer;
impl<F: RichField + Extendable<D>, const D: usize> GateSerializer<F, D> for U32GateSerializer {
    impl_gate_serializer! {
        DefaultGateSerializer,
        ArithmeticGate,
        ArithmeticExtensionGate<D>,
        BaseSumGate<2>,
        ConstantGate,
        CosetInterpolationGate<F, D>,
        ExponentiationGate<F, D>,
        LookupGate,
        LookupTableGate,
        MulExtensionGate<D>,
        NoopGate,
        PoseidonMdsGate<F, D>,
        PoseidonGate<F, D>,
        PublicInputGate,
        RandomAccessGate<F, D>,
        ReducingExtensionGate<D>,
        ReducingGate<D>,
        U32ArithmeticGate<F,D>,
        U32AddManyGate<F,D>,
        U32SubtractionGate<F,D>,
        U32RangeCheckGate<F,D>,
        ComparisonGate<F,D>
    }
}
