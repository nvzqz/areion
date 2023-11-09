use core::{fmt, slice};

use crate::intrinsics::*;

use digest::block_buffer::Eager;
use digest::core_api::{
    Block, BlockSizeUser, Buffer, BufferKindUser, CoreWrapper, FixedOutputCore, UpdateCore,
};
use digest::crypto_common::AlgorithmName;
use digest::generic_array::GenericArray;
use digest::typenum::{Unsigned, U32};
use digest::{HashMarker, Output, OutputSizeUser, Reset};

#[derive(Debug, Clone)]
struct State(AesBlock, AesBlock);

impl Default for State {
    fn default() -> Self {
        Self(
            load_64x2(0x85ae67bb67e6096a, 0x3af54fa572f36e3c),
            load_64x2(0x8c68059b7f520e51, 0x19cde05babd9831f),
        )
    }
}

impl State {
    fn compress(&mut self, blocks: &[GenericArray<u8, U32>]) {
        let Self(mut h0, mut h1) = self;
        for block in blocks {
            let (m0, m1) = (load(&block[..16]), load(&block[16..]));
            (h0, h1) = crate::areion512_dm(m0, m1, h0, h1);
        }
        *self = Self(h0, h1);
    }
}

#[derive(Debug, Default, Clone)]
pub struct Core {
    state: State,
    block_len: u64,
}

impl HashMarker for Core {}

impl BlockSizeUser for Core {
    type BlockSize = U32;
}

impl BufferKindUser for Core {
    type BufferKind = Eager;
}

impl OutputSizeUser for Core {
    type OutputSize = U32;
}

impl UpdateCore for Core {
    #[inline]
    fn update_blocks(&mut self, blocks: &[Block<Self>]) {
        self.block_len += blocks.len() as u64;
        self.state.compress(blocks);
    }
}

impl FixedOutputCore for Core {
    fn finalize_fixed_core(&mut self, buffer: &mut Buffer<Self>, out: &mut Output<Self>) {
        let bs = Self::BlockSize::U64;
        let bit_len = 8 * (buffer.get_pos() as u64 + bs * self.block_len);
        buffer.len64_padding_be(bit_len, |b| self.state.compress(slice::from_ref(b)));

        store(&mut out[..16], self.state.0);
        store(&mut out[16..], self.state.1);
    }
}

impl Reset for Core {
    #[inline]
    fn reset(&mut self) {
        *self = Default::default();
    }
}

impl AlgorithmName for Core {
    #[inline]
    fn write_alg_name(f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Areion512-MD")
    }
}

pub type Areion512Md = CoreWrapper<Core>;
