use crate::function::Values;
use crate::object::array::Array;
use crate::object::dict::Dict;
use crate::object::dict::keys::{C0, C1, N};
use crate::object::number::Number;
use itertools::izip;
use smallvec::smallvec;

/// Type 2 exponential interpolation function.
#[derive(Debug)]
pub(crate) struct Type2 {
    pub(crate) c0: Values,
    pub(crate) c1: Values,
    pub(crate) n: f32,
}

impl Type2 {
    pub(crate) fn new(dict: &Dict) -> Option<Self> {
        let c0 = dict.get::<Array>(C0)?.into();
        let c1 = dict.get::<Array>(C1)?.into();
        let n = dict.get::<Number>(N)?.as_f32();

        Some(Self { c0, c1, n })
    }

    pub(crate) fn eval(&self, mut input: f32) -> Values {
        let mut out = smallvec![0.0; self.c0.len()];

        for (c0, c1, out) in izip!(&self.c0, &self.c1, &mut out) {
            *out = *c0 + input.powf(self.n) * (*c1 - *c0);
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use crate::function::Function;
    use crate::function::type2::Type2;
    use crate::object::Object;
    use crate::object::dict::Dict;
    use crate::reader::Readable;
    use smallvec::smallvec;

    #[test]
    fn simple() {
        let func = Function::new(&Object::Dict(
            Dict::from_bytes(
                b"<<
              /FunctionType 2
              /Domain [ 0  1 ]
              /C0 [ 0 20  ]
              /C1 [ 30 -50 ]
              /N 1
            >>",
            )
            .unwrap(),
        ))
        .unwrap();

        assert_eq!(func.eval(smallvec![0.0]).unwrap().as_ref(), &[0.0, 20.0]);
        assert_eq!(func.eval(smallvec![0.5]).unwrap().as_ref(), &[15.0, -15.0]);
        assert_eq!(func.eval(smallvec![1.0]).unwrap().as_ref(), &[30.0, -50.0]);
    }

    #[test]
    fn with_exponent() {
        let func = Function::new(&Object::Dict(
            Dict::from_bytes(
                b"<<
              /FunctionType 2
              /Domain [ 0  1 ]
              /C0 [ 0  ]
              /C1 [ 30 ]
              /N 2
            >>",
            )
            .unwrap(),
        ))
        .unwrap();

        assert_eq!(func.eval(smallvec![0.5]), Some(smallvec![7.5]));
    }

    #[test]
    fn clamp_domain() {
        let func = Function::new(&Object::Dict(
            Dict::from_bytes(
                b"<<
              /FunctionType 2
              /Domain [ 0.2  0.8 ]
              /C0 [ 0  ]
              /C1 [ 30 ]
              /N 2
            >>",
            )
            .unwrap(),
        ))
        .unwrap();

        assert_eq!(
            func.eval(smallvec![0.0]).as_ref(),
            func.eval(smallvec![0.2]).as_ref(),
        );
        assert_eq!(
            func.eval(smallvec![-10.]).as_ref(),
            func.eval(smallvec![0.2]).as_ref(),
        );
        assert_eq!(
            func.eval(smallvec![0.8]).as_ref(),
            func.eval(smallvec![0.8]).as_ref(),
        );
        assert_eq!(
            func.eval(smallvec![1.2]).as_ref(),
            func.eval(smallvec![1.0]).as_ref(),
        );
    }

    #[test]
    fn clamp_range() {
        let func = Function::new(&Object::Dict(
            Dict::from_bytes(
                b"<<
              /FunctionType 2
              /Domain [ 0.0  1.0 ]
              /Range [10.0 20.0]
              /C0 [ 0  ]
              /C1 [ 30 ]
              /N 1
            >>",
            )
            .unwrap(),
        ))
        .unwrap();

        assert_eq!(func.eval(smallvec![0.0]), Some(smallvec![10.0]));
        assert_eq!(func.eval(smallvec![0.5]), Some(smallvec![15.0]));
        assert_eq!(func.eval(smallvec![1.0]), Some(smallvec![20.0]));
    }
}
