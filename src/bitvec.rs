#[derive(Clone, Debug)]
pub struct PackedVec32 {
    bits_per_elem: u8, // 1..=32 (caller contract)
    len: usize,
    words: Vec<u32>,
}

impl PackedVec32 {
    pub fn new(len: usize, bits_per_elem: u8) -> Self {
        // No runtime checks by default; caller must pass valid bits_per_elem.
        let total_bits = len * bits_per_elem as usize;
        let words = vec![0u32; total_bits.div_ceil(32)];
        Self {
            bits_per_elem,
            len,
            words,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }
    pub fn bits_per_elem(&self) -> u8 {
        self.bits_per_elem
    }

    #[inline]
    fn mask(bits: u8) -> u32 {
        if bits == 32 {
            u32::MAX
        } else {
            (1u32 << bits) - 1
        }
    }

    /// No bounds/fit checks. Caller guarantees `index < len`.
    #[inline]
    pub fn get(&self, index: usize) -> u32 {
        let bpe = self.bits_per_elem as usize;
        let bit_pos = index * bpe;
        let w = bit_pos / 32;
        let o = bit_pos % 32;
        let mask = Self::mask(self.bits_per_elem) as u64;

        if o + bpe <= 32 {
            (((self.words[w] as u64) >> o) & mask) as u32
        } else {
            let low = (self.words[w] as u64) >> o;
            let high_bits = o + bpe - 32;
            let high_mask = (1u64 << high_bits) - 1;
            let high = (self.words[w + 1] as u64) & high_mask;
            (((high << (32 - o)) | low) & mask) as u32
        }
    }

    /// No bounds/fit checks. Caller guarantees `index < len`.
    /// Value is masked to `bits_per_elem` bits (truncates high bits).
    #[inline]
    pub fn set(&mut self, index: usize, value: u32) {
        let bpe = self.bits_per_elem as usize;
        let mask = Self::mask(self.bits_per_elem);
        let value = value & mask; // truncate instead of checking

        let bit_pos = index * bpe;
        let w = bit_pos / 32;
        let o = bit_pos % 32;

        if o + bpe <= 32 {
            let shifted_mask = (mask as u64) << o;
            let old = self.words[w] as u64;
            let newv = (old & !shifted_mask) | ((value as u64) << o);
            self.words[w] = newv as u32;
        } else {
            let low_bits = 32 - o;
            let high_bits = bpe - low_bits;

            let low_part_mask = (1u64 << low_bits) - 1;
            let low_mask = low_part_mask << o;
            let low_part = ((value as u64) & low_part_mask) << o;

            let w0 = self.words[w] as u64;
            self.words[w] = ((w0 & !low_mask) | low_part) as u32;

            let high_mask = (1u64 << high_bits) - 1;
            let high_part = ((value as u64) >> low_bits) & high_mask;

            let w1 = self.words[w + 1] as u64;
            self.words[w + 1] = ((w1 & !high_mask) | high_part) as u32;
        }
    }

    /// Repack and overwrite buffer. No fit checks; values are truncated to new width.
    pub fn repack_in_place(&mut self, new_bits_per_elem: u8) {
        let mut out = PackedVec32::new(self.len, new_bits_per_elem);
        for i in 0..self.len {
            out.set(i, self.get(i)); // truncation handled by set()
        }
        *self = out;
    }

    /// Replace everything. No fit checks; values are truncated to width.
    pub fn overwrite_from_values(&mut self, bits_per_elem: u8, values: &[u32]) {
        let mut out = PackedVec32::new(values.len(), bits_per_elem);
        for (i, &v) in values.iter().enumerate() {
            out.set(i, v);
        }
        *self = out;
    }
}
