use crate::file::xref::XRef;
use crate::object;
use crate::object::r#ref::MaybeRef;
use crate::object::{Object, ObjectLike};
use crate::reader::{Readable, Reader, Skippable};
use log::warn;
use smallvec::SmallVec;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;

/// An array of PDF objects.
#[derive(Clone)]
pub struct Array<'a> {
    data: &'a [u8],
    xref: &'a XRef,
}

// Note that this is not structural equality, i.e. two arrays with the same
// items are still considered different if they have different whitespaces.
impl PartialEq for Array<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl<'a> Array<'a> {
    /// Returns an iterator over the objects of the array.
    fn raw_iter(&self) -> ArrayIter<'a> {
        ArrayIter::new(self.data, self.xref)
    }

    /// Returns an iterator over the resolved objects of the array.
    #[allow(
        private_bounds,
        reason = "users shouldn't be able to implement `ObjectLike` for custom objects."
    )]
    pub fn iter<T>(&self) -> ResolvedArrayIter<'a, T>
    where
        T: ObjectLike<'a>,
    {
        ResolvedArrayIter::new(self.data, self.xref)
    }

    pub fn flex_iter(&self) -> FlexArrayIter<'a> {
        FlexArrayIter::new(self.data, self.xref)
    }
}

impl Debug for Array<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut debug_list = f.debug_list();

        self.raw_iter().for_each(|i| {
            debug_list.entry(&i);
        });

        Ok(())
    }
}

object!(Array<'a>, Array);

impl Skippable for Array<'_> {
    fn skip<const PLAIN: bool>(r: &mut Reader<'_>) -> Option<()> {
        r.forward_tag(b"[")?;

        loop {
            r.skip_white_spaces_and_comments();

            if let Some(()) = r.forward_tag(b"]") {
                return Some(());
            } else {
                if PLAIN {
                    r.skip_non_plain::<Object>()?;
                } else {
                    r.skip_non_plain::<MaybeRef<Object>>()?;
                }
            }
        }
    }
}

impl Default for Array<'_> {
    fn default() -> Self {
        Self::from_bytes(b"[]").unwrap()
    }
}

impl<'a> Readable<'a> for Array<'a> {
    fn read<const PLAIN: bool>(r: &mut Reader<'a>, xref: &'a XRef) -> Option<Self> {
        let bytes = r.skip::<PLAIN, Array>()?;

        Some(Self {
            data: &bytes[1..bytes.len() - 1],
            xref: xref,
        })
    }
}

/// An iterator over the items of an array.
pub(crate) struct ArrayIter<'a> {
    reader: Reader<'a>,
    xref: &'a XRef,
}

impl<'a> ArrayIter<'a> {
    fn new(data: &'a [u8], xref: &'a XRef) -> Self {
        Self {
            reader: Reader::new(data),
            xref,
        }
    }
}

impl<'a> Iterator for ArrayIter<'a> {
    type Item = MaybeRef<Object<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.reader.skip_white_spaces_and_comments();

        if !self.reader.at_end() {
            // Objects are already guaranteed to be valid.
            let item = self
                .reader
                .read_with_xref::<MaybeRef<Object>>(&self.xref)
                .unwrap();
            return Some(item);
        }

        None
    }
}

pub struct ResolvedArrayIter<'a, T> {
    flex_iter: FlexArrayIter<'a>,
    phantom_data: PhantomData<T>,
}

impl<'a, T> ResolvedArrayIter<'a, T> {
    fn new(data: &'a [u8], xref: &'a XRef) -> Self {
        Self {
            flex_iter: FlexArrayIter::new(data, xref),
            phantom_data: PhantomData::default(),
        }
    }
}

impl<'a, T> Iterator for ResolvedArrayIter<'a, T>
where
    T: ObjectLike<'a>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.flex_iter.next::<T>()
    }
}

pub struct FlexArrayIter<'a> {
    reader: Reader<'a>,
    xref: &'a XRef,
}

impl<'a> FlexArrayIter<'a> {
    fn new(data: &'a [u8], xref: &'a XRef) -> Self {
        Self {
            reader: Reader::new(data),
            xref,
        }
    }

    #[allow(
        private_bounds,
        reason = "users shouldn't be able to implement `ObjectLike` for custom objects."
    )]
    pub fn next<T: ObjectLike<'a>>(&mut self) -> Option<T> {
        self.reader.skip_white_spaces_and_comments();

        if !self.reader.at_end() {
            return match self.reader.read_with_xref::<MaybeRef<T>>(&self.xref)? {
                MaybeRef::Ref(r) => self.xref.get::<T>(r.into()),
                MaybeRef::NotRef(i) => Some(i),
            };
        }

        None
    }
}

impl<'a, T: ObjectLike<'a> + Copy + Default, const C: usize> TryFrom<Array<'a>> for [T; C] {
    type Error = ();

    fn try_from(value: Array<'a>) -> Result<Self, Self::Error> {
        let mut iter = value.iter::<T>();

        let mut val = [T::default(); C];

        for i in 0..C {
            val[i] = iter.next().ok_or(())?;
        }

        if iter.next().is_some() {
            warn!("found excess elements in array");

            return Err(());
        }

        Ok(val)
    }
}

impl<'a, T: ObjectLike<'a> + Copy + Default, const C: usize> TryFrom<Object<'a>> for [T; C]
where
    [T; C]: TryFrom<Array<'a>, Error = ()>,
{
    type Error = ();

    fn try_from(value: Object<'a>) -> Result<Self, Self::Error> {
        match value {
            Object::Array(a) => a.try_into(),
            _ => Err(()),
        }
    }
}

impl<'a, T: ObjectLike<'a> + Copy + Default, const C: usize> Readable<'a> for [T; C] {
    fn read<const PLAIN: bool>(r: &mut Reader<'a>, xref: &'a XRef) -> Option<Self> {
        let array = Array::read::<PLAIN>(r, xref)?;
        array.try_into().ok()
    }
}

impl<'a, T: ObjectLike<'a> + Copy + Default, const C: usize> ObjectLike<'a> for [T; C] {}

impl<'a, T: ObjectLike<'a>> TryFrom<Array<'a>> for Vec<T> {
    type Error = ();

    fn try_from(value: Array<'a>) -> Result<Self, Self::Error> {
        Ok(value.iter::<T>().collect())
    }
}

impl<'a, T: ObjectLike<'a>> TryFrom<Object<'a>> for Vec<T> {
    type Error = ();

    fn try_from(value: Object<'a>) -> Result<Self, Self::Error> {
        match value {
            Object::Array(a) => a.try_into(),
            _ => Err(()),
        }
    }
}

impl<'a, T: ObjectLike<'a>> Readable<'a> for Vec<T> {
    fn read<const PLAIN: bool>(r: &mut Reader<'a>, xref: &'a XRef) -> Option<Self> {
        let array = Array::read::<PLAIN>(r, xref)?;
        array.try_into().ok()
    }
}

impl<'a, T: ObjectLike<'a>> ObjectLike<'a> for Vec<T> {}

impl<'a, U: ObjectLike<'a>, T: ObjectLike<'a> + smallvec::Array<Item = U>> TryFrom<Array<'a>>
    for SmallVec<T>
{
    type Error = ();

    fn try_from(value: Array<'a>) -> Result<Self, Self::Error> {
        Ok(value.iter::<U>().collect())
    }
}

impl<'a, U: ObjectLike<'a>, T: ObjectLike<'a> + smallvec::Array<Item = U>> TryFrom<Object<'a>>
    for SmallVec<T>
{
    type Error = ();

    fn try_from(value: Object<'a>) -> Result<Self, Self::Error> {
        match value {
            Object::Array(a) => a.try_into(),
            _ => Err(()),
        }
    }
}

impl<'a, U: ObjectLike<'a>, T: ObjectLike<'a> + smallvec::Array<Item = U>> Readable<'a>
    for SmallVec<T>
{
    fn read<const PLAIN: bool>(r: &mut Reader<'a>, xref: &'a XRef) -> Option<Self> {
        let array = Array::read::<PLAIN>(r, xref)?;
        array.try_into().ok()
    }
}

impl<'a, U: ObjectLike<'a>, T: ObjectLike<'a> + smallvec::Array<Item = U>> ObjectLike<'a>
    for SmallVec<T>
where
    U: Clone,
    U: Debug,
{
}

#[cfg(test)]
mod tests {
    use crate::file::xref::XRef;
    use crate::object::Object;
    use crate::object::array::Array;
    use crate::object::r#ref::{MaybeRef, ObjRef};
    use crate::reader::Reader;

    fn array_impl(data: &[u8]) -> Option<Vec<Object>> {
        Reader::new(data)
            .read_with_xref::<Array>(&XRef::dummy())
            .map(|a| a.iter::<Object>().collect::<Vec<_>>())
    }

    fn array_ref_impl(data: &[u8]) -> Option<Vec<MaybeRef<Object>>> {
        Reader::new(data)
            .read_with_xref::<Array>(&XRef::dummy())
            .map(|a| a.raw_iter().collect::<Vec<_>>())
    }

    #[test]
    fn empty_array_1() {
        let res = array_impl(b"[]").unwrap();
        assert!(res.is_empty());
    }

    #[test]
    fn empty_array_2() {
        let res = array_impl(b"[   \n]").unwrap();
        assert!(res.is_empty());
    }

    #[test]
    fn array_1() {
        let res = array_impl(b"[34]").unwrap();
        assert!(matches!(res[0], Object::Number(_)));
    }

    #[test]
    fn array_2() {
        let res = array_impl(b"[true  ]").unwrap();
        assert!(matches!(res[0], Object::Boolean(_)));
    }

    #[test]
    fn array_3() {
        let res = array_impl(b"[true \n false 34.564]").unwrap();
        assert!(matches!(res[0], Object::Boolean(_)));
        assert!(matches!(res[1], Object::Boolean(_)));
        assert!(matches!(res[2], Object::Number(_)));
    }

    #[test]
    fn array_4() {
        let res = array_impl(b"[(A string.) << /Hi 34.35 >>]").unwrap();
        assert!(matches!(res[0], Object::String(_)));
        assert!(matches!(res[1], Object::Dict(_)));
    }

    #[test]
    fn array_5() {
        let res = array_impl(b"[[32]  345.6]").unwrap();
        assert!(matches!(res[0], Object::Array(_)));
        assert!(matches!(res[1], Object::Number(_)));
    }

    #[test]
    fn array_with_ref() {
        let res = array_ref_impl(b"[345 34 5 R 34.0]").unwrap();
        assert!(matches!(res[0], MaybeRef::NotRef(Object::Number(_))));
        assert!(matches!(
            res[1],
            MaybeRef::Ref(ObjRef {
                obj_number: 34,
                gen_number: 5
            })
        ));
        assert!(matches!(res[2], MaybeRef::NotRef(Object::Number(_))));
    }

    #[test]
    fn array_with_comment() {
        let res = array_impl(b"[true % A comment \n false]").unwrap();
        assert!(matches!(res[0], Object::Boolean(_)));
        assert!(matches!(res[1], Object::Boolean(_)));
    }

    #[test]
    fn array_with_trailing() {
        let res = array_impl(b"[(Hi) /Test]trialing data").unwrap();
        assert!(matches!(res[0], Object::String(_)));
        assert!(matches!(res[1], Object::Name(_)));
    }
}
