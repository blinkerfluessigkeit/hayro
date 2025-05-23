use crate::bit::{BitReader, BitSize};
use crate::function::{DomainRange, Values, interpolate, read_domain_range};
use crate::object::array::Array;
use crate::object::dict::keys::{BITS_PER_SAMPLE, DECODE, ENCODE, SIZE};
use crate::object::stream::Stream;
use crate::util::OptionLog;
use itertools::izip;
use log::warn;
use smallvec::{SmallVec, ToSmallVec, smallvec};
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct Type0 {
    sizes: Vec<u32>,
    table: HashMap<Key, IntVec>,
    domain: DomainRange,
    range: DomainRange,
    bits_per_sample: u8,
    encode: DomainRange,
    decode: DomainRange,
}

impl Type0 {
    pub fn new(stream: &Stream, domain: &DomainRange, range: &DomainRange) -> Option<Self> {
        let dict = stream.dict();
        let bits_per_sample = dict.get::<u8>(BITS_PER_SAMPLE)?;

        if !matches!(bits_per_sample, 1 | 2 | 4 | 8 | 16 | 24 | 32) {
            warn!("unsupported bits per sample: {}", bits_per_sample);
            return None;
        }
        let sizes = dict.get::<Array>(SIZE)?.iter::<u32>().collect::<Vec<_>>();

        let encode = dict
            .get::<Array>(ENCODE)
            .and_then(|a| read_domain_range(&a))
            .unwrap_or(sizes.iter().map(|s| (0.0, (*s - 1) as f32)).collect());

        let decode = dict
            .get::<Array>(DECODE)
            .and_then(|a| read_domain_range(&a))
            .unwrap_or(range.clone());

        let data = {
            let decoded = stream.decoded()?;
            let mut buf = vec![];
            let mut reader = BitReader::new(&decoded);

            while let Some(data) = reader.read(BitSize::from_u8(bits_per_sample)?) {
                buf.push(data);
            }

            buf
        };

        let table = build_table(&data, &sizes, range.len())?;

        Some(Self {
            sizes,
            domain: domain.clone(),
            range: range.clone(),
            bits_per_sample,
            table,
            encode,
            decode,
        })
    }

    fn output_dimension(&self) -> usize {
        self.range.len()
    }

    pub(crate) fn eval(&self, input: Values) -> Option<Values> {
        if input.len() != self.sizes.len() {
            warn!("wrong number of arguments for sampled function");

            return None;
        }

        let mut key = input;
        for (x, domain, encode, size) in izip!(&mut key, &self.domain, &self.encode, &self.sizes) {
            *x = interpolate(*x, domain.0, domain.1, encode.0, encode.1);
            *x = x.max(0.0).min(*size as f32 - 1.0);
        }

        let in_prev = key.iter().map(|v| v.floor() as u32).collect::<IntVec>();
        let in_next = key.iter().map(|v| v.floor() as u32).collect::<IntVec>();

        let interpolator = Interpolator::new(
            key.clone().to_smallvec(),
            in_prev,
            in_next,
            self.sizes.to_smallvec(),
            self.range.len(),
        );

        let interpolated = interpolator.interpolate(&self.table);
        let mut out = smallvec![0.0; self.output_dimension()];

        for (x, decode, out) in izip!(&interpolated, &self.decode, &mut out) {
            *out = interpolate(
                *x as f32,
                0.0,
                (2u32.pow(self.bits_per_sample as u32) - 1) as f32,
                decode.0,
                decode.1,
            );
        }

        Some(out)
    }
}

type FloatVec = SmallVec<[f32; 4]>;
type IntVec = SmallVec<[u32; 4]>;

// Taken from PDFBox
struct Interpolator {
    input: FloatVec,
    sizes: IntVec,
    in_prev: IntVec,
    in_next: IntVec,
    out_len: usize,
}

impl Interpolator {
    pub fn new(
        input: FloatVec,
        in_prev: IntVec,
        in_next: IntVec,
        sizes: IntVec,
        out_len: usize,
    ) -> Self {
        Self {
            input,
            in_prev,
            in_next,
            sizes,
            out_len,
        }
    }

    pub fn interpolate(&self, table: &HashMap<Key, IntVec>) -> FloatVec {
        self.interpolate_inner(smallvec![0; self.input.len()], 0, table)
    }

    fn interpolate_inner(
        &self,
        mut coord: IntVec,
        step: usize,
        table: &HashMap<Key, IntVec>,
    ) -> FloatVec {
        if step == self.input.len() - 1 {
            if self.in_prev[step] == self.in_next[step] {
                coord[step] = self.in_prev[step];
                table
                    .get(&Key::from_raw(&self.sizes, &coord))
                    .unwrap()
                    .clone()
                    .iter()
                    .map(|n| *n as f32)
                    .collect()
            } else {
                coord[step] = self.in_prev[step];
                let val1 = table.get(&Key::from_raw(&self.sizes, &coord)).unwrap();
                coord[step] = self.in_next[step];
                let val2 = table.get(&Key::from_raw(&self.sizes, &coord)).unwrap();

                let mut out = smallvec![0.0; self.out_len];

                for i in 0..self.out_len {
                    out[i] = interpolate(
                        self.input[step],
                        self.in_prev[step] as f32,
                        self.in_next[step] as f32,
                        val1[i] as f32,
                        val2[i] as f32,
                    )
                }

                out
            }
        } else {
            if self.in_prev[step] == self.in_next[step] {
                coord[step] = self.in_prev[step];
                self.interpolate_inner(coord, step + 1, table)
            } else {
                coord[step] = self.in_prev[step];
                let val1 = self.interpolate_inner(coord.clone(), step + 1, table);
                coord[step] = self.in_next[step];
                let val2 = self.interpolate_inner(coord, step + 1, table);

                let mut out = smallvec![0.0; self.out_len];

                for i in 0..self.out_len {
                    out[i] = interpolate(
                        self.input[step],
                        self.in_prev[step] as f32,
                        self.in_next[step] as f32,
                        val1[i],
                        val2[i],
                    )
                }

                out
            }
        }
    }
}

fn build_table(data: &[u32], sizes: &[u32], n: usize) -> Option<HashMap<Key, IntVec>> {
    let mut key = Key::new(sizes);
    let mut table = HashMap::new();

    let mut first = true;
    for b in data.chunks(n) {
        if !first {
            key.increment();
        }

        table.insert(key.clone(), b.to_smallvec());

        first = false;
    }

    Some(table)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Key {
    sizes: SmallVec<[u32; 4]>,
    parts: SmallVec<[u32; 4]>,
}

impl Key {
    fn new(sizes: &[u32]) -> Self {
        let parts = smallvec![0; sizes.len()];

        Self {
            sizes: sizes.to_smallvec(),
            parts,
        }
    }

    fn from_raw(sizes: &[u32], parts: &[u32]) -> Self {
        Self {
            sizes: sizes.to_smallvec(),
            parts: parts.to_smallvec(),
        }
    }

    fn increment(&mut self) -> Option<()> {
        self.increment_index(0)
    }

    fn increment_index(&mut self, index: usize) -> Option<()> {
        let size = self
            .sizes
            .get(index)
            .error_none("overflowed key in sampled function")?;
        let val = self.parts.get_mut(index)?;

        if *val >= (*size - 1) {
            *val = 0;
            self.increment_index(index + 1)?;
        } else {
            *val += 1;
        }

        Some(())
    }
}
