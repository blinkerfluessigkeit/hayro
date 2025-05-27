use crate::file::xref::XRef;
use crate::filter::{Filter, FilterResult, apply_filter};
use crate::object::array::Array;
use crate::object::dict::Dict;
use crate::object::dict::keys::{DECODE_PARMS, DP, F, FILTER, LENGTH};
use crate::object::name::Name;
use crate::object::{Object, ObjectLike};
use crate::reader::{Readable, Reader, Skippable};
use crate::util::OptionLog;
use log::warn;
use std::fmt::{Debug, Formatter};

/// A stream of arbitrary data.
#[derive(Clone, PartialEq)]
pub struct Stream<'a> {
    pub(crate) dict: Dict<'a>,
    data: &'a [u8],
}

impl<'a> Stream<'a> {
    /// Return the raw (potentially with some applied filters) data of the stream.
    pub fn raw_data(&self) -> &'a [u8] {
        self.data
    }

    /// Return the raw (potentially with some applied filters) data of the stream.
    pub fn dict(&self) -> &Dict<'a> {
        &self.dict
    }

    /// Return the decoded data of the stream.
    ///
    /// Note that the result of this method will not be cached, so calling it multiple
    /// times is expensive.
    pub fn decoded(&self) -> Option<Vec<u8>> {
        self.decoded_image().map(|r| r.data)
    }

    pub fn decoded_image(&self) -> Option<FilterResult> {
        if let Some(filter) = self
            .dict
            .get::<Name>(F)
            .or_else(|| self.dict.get::<Name>(FILTER))
            .and_then(|n| Filter::from_name(&n))
        {
            let params = self
                .dict
                .get::<Dict>(DP)
                .or_else(|| self.dict.get::<Dict>(DECODE_PARMS));

            Some(apply_filter(self.data, filter, params.as_ref())?)
        } else if let Some(filters) = self
            .dict
            .get::<Array>(F)
            .or_else(|| self.dict.get::<Array>(FILTER))
        {
            // TODO: Avoid allocation?

            let filters = filters
                .iter::<Name>()
                .map(|n| Filter::from_name(&n))
                .collect::<Option<Vec<_>>>()?;
            let params = self
                .dict
                .get::<Array>(DP)
                .or_else(|| self.dict.get::<Array>(DECODE_PARMS))
                .map(|a| a.iter::<Object>().collect())
                .unwrap_or(vec![]);

            let mut current: Option<FilterResult> = None;

            for i in 0..filters.len() {
                let params = params.get(i).and_then(|p| p.clone().cast::<Dict>());

                let new = apply_filter(
                    current
                        .as_ref()
                        .map(|c| c.data.as_ref())
                        .unwrap_or(self.data),
                    filters[i],
                    params.as_ref(),
                )?;
                current = Some(new);
            }

            Some(current.unwrap_or(FilterResult {
                data: self.data.to_vec(),
                color_space: None,
                bits_per_component: None,
            }))
        } else {
            Some(FilterResult {
                data: self.data.to_vec(),
                color_space: None,
                bits_per_component: None,
            })
        }
    }

    pub fn from_raw(data: &'a [u8], dict: Dict<'a>) -> Self {
        Self { dict, data }
    }
}

impl Debug for Stream<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Stream (len: {:?})", self.data.len())
    }
}

impl Skippable for Stream<'_> {
    fn skip<const PLAIN: bool>(_: &mut Reader<'_>) -> Option<()> {
        // A stream can never appear in a dict/array, so it should never be skipped.
        warn!("attempted to skip a stream object");

        None
    }
}

impl<'a> Readable<'a> for Stream<'a> {
    fn read<const PLAIN: bool>(r: &mut Reader<'a>, xref: &XRef<'a>) -> Option<Self> {
        let dict = r.read_with_xref::<Dict>(xref)?;

        if dict.contains_key(F) {
            warn!("encountered stream referencing external file, which is unsupported");

            return None;
        }

        let offset = r.offset();
        parse_proper(r, &dict)
            .or_else(|| {
                warn!("failed to parse stream, trying to parse it manually");

                r.jump(offset);
                parse_fallback(r, &dict)
            })
            .error_none("was unable to manually parse the stream")
    }
}

fn parse_proper<'a>(r: &mut Reader<'a>, dict: &Dict<'a>) -> Option<Stream<'a>> {
    let length = dict.get::<u32>(LENGTH)?;

    r.skip_white_spaces_and_comments();
    r.forward_tag(b"stream")?;
    r.forward_tag(b"\n").or_else(|| r.forward_tag(b"\r\n"))?;
    let data = r.read_bytes(length as usize)?;
    r.skip_white_spaces();
    r.forward_tag(b"endstream")?;

    Some(Stream {
        data,
        dict: dict.clone(),
    })
}

fn parse_fallback<'a>(r: &mut Reader<'a>, dict: &Dict<'a>) -> Option<Stream<'a>> {
    while r.forward_tag(b"stream").is_none() {
        r.read_byte()?;
    }

    r.forward_tag(b"\n").or_else(|| r.forward_tag(b"\r\n"))?;

    let data_start = r.tail()?;
    let start = r.offset();

    loop {
        if r.peek_byte()?.is_ascii_whitespace() {
            let length = r.offset() - start;
            let data = data_start.get(..length)?;

            r.skip_white_spaces();

            // This was just a whitespace in the data stream but not actually marking the end
            // of the stream, so continue searching.
            if r.forward_tag(b"endstream").is_none() {
                continue;
            }

            let stream = Stream {
                data,
                dict: dict.clone(),
            };

            // Try decoding the stream to see if it is valid.
            if stream.decoded().is_some() {
                warn!("managed to reconstruct the stream");

                // Seems like we found the end!
                return Some(stream);
            }
        } else {
            r.read_byte()?;
        }
    }
}

impl<'a> TryFrom<Object<'a>> for Stream<'a> {
    type Error = ();

    fn try_from(value: Object<'a>) -> Result<Self, Self::Error> {
        match value {
            Object::Stream(s) => Ok(s),
            _ => Err(()),
        }
    }
}

impl<'a> ObjectLike<'a> for Stream<'a> {}

#[cfg(test)]
mod tests {
    use crate::file::xref::XRef;
    use crate::object::stream::Stream;
    use crate::reader::Reader;

    #[test]
    fn stream() {
        let data = b"<< /Length 10 >> stream\nabcdefghij\nendstream";
        let mut r = Reader::new(data);
        let stream = r.read_with_xref::<Stream>(&XRef::dummy()).unwrap();

        assert_eq!(stream.data, b"abcdefghij");
    }
}
