use kurbo::{Affine, Point, Vec2};
use peniko::color::{AlphaColor, Srgb};
use peniko::color::palette::css::TRANSPARENT;
use rustc_hash::FxHashMap;
use smallvec::{smallvec, ToSmallVec};
use hayro_interpret::color::{ColorComponents, ColorSpace};
use hayro_interpret::pattern::ShadingPattern;
use hayro_interpret::shading::{ShadingFunction, ShadingType, Triangle};
use crate::encode::{x_y_advances, EncodeExt, EncodedPaint};
use crate::paint::{IndexedPaint, Paint};

#[derive(Debug)]
pub(crate) struct EncodedShading {
    pub(crate) x_advance: Vec2,
    pub(crate) y_advance: Vec2,
    pub(crate) color_space: ColorSpace,
    pub(crate) initial_transform: Affine,
    pub(crate) background_color: AlphaColor<Srgb>,
    pub(crate) shading_type: EncodedShadingType,
}

impl EncodeExt for ShadingPattern {
    fn encode_into(&self, paints: &mut Vec<EncodedPaint>, transform: Affine) -> Paint {
        let mut base_transform;
        let idx = paints.len();

        let shading_type = match self.shading.shading_type.as_ref() {
            ShadingType::FunctionBased {
                domain,
                matrix,
                function,
            } => {
                base_transform = (transform * self.matrix * *matrix).inverse();
                encode_function_shading(domain, function)
            },
            ShadingType::RadialAxial {
                coords,
                domain,
                function,
                extend,
                axial,
            } => {
                let (encoded, initial_transform) = encode_axial_shading(
                    *coords, *domain, function, *extend, *axial,
                );

                base_transform = initial_transform * (transform * self.matrix).inverse();

                encoded
            },
            ShadingType::TriangleMesh {
                triangles,
                function,
            } => {
                let full_transform = transform * self.matrix;
                let samples = sample_triangles(triangles, full_transform);

                base_transform = Affine::IDENTITY;

                EncodedShadingType::Sampled {
                    samples,
                    function: function.clone(),
                }
            }
            ShadingType::CoonsPatchMesh { patches, function } => {
                let triangles = patches.iter().flat_map(|p| p.to_triangles()).collect::<Vec<_>>();

                let full_transform = transform * self.matrix;
                let samples = sample_triangles(&triangles, full_transform);

                base_transform = Affine::IDENTITY;

                EncodedShadingType::Sampled {
                    samples,
                    function: function.clone(),
                }
            }
            ShadingType::TensorProductPatchMesh { patches, function } => {
                let triangles = patches.iter().flat_map(|p| p.to_triangles()).collect::<Vec<_>>();

                let full_transform = transform * self.matrix;
                let samples = sample_triangles(&triangles, full_transform);

                base_transform = Affine::IDENTITY;

                EncodedShadingType::Sampled {
                    samples,
                    function: function.clone(),
                }
            }
        };
        
        base_transform = base_transform * Affine::translate((0.5, 0.5));
        let color_space = self.shading.color_space.clone();
        let (x_advance, y_advance) = x_y_advances(&base_transform);

        let background_color = self
            .shading
            .background
            .as_ref()
            .map(|b| color_space.to_rgba(&b, 1.0))
            .unwrap_or(TRANSPARENT);

        let encoded = EncodedShading {
            x_advance,
            y_advance,
            color_space,
            initial_transform: base_transform,
            background_color,
            shading_type,
        };

        paints.push(EncodedPaint::Shading(encoded));

        Paint::Indexed(IndexedPaint::new(idx))
    }
}

fn encode_axial_shading(
    coords: [f32; 6],
    domain: [f32; 2],
    function: &ShadingFunction,
    extend: [bool; 2],
    is_axial: bool,
) -> (EncodedShadingType, Affine) {
    let initial_transform;

    let params = if is_axial {
        let [x_0, y_0, x_1, y_1, _, _] = coords;

        initial_transform = ts_from_line_to_line(
            Point::new(x_0 as f64, y_0 as f64),
            Point::new(x_1 as f64, y_1 as f64), Point::ZERO, Point::new(1.0, 0.0));

        RadialAxialParams::Axial
    } else {
        let [x_0, y_0, r0, x_1, y_1, r_1] = coords;

        initial_transform = Affine::translate((-x_0 as f64, -y_0 as f64));
        let new_x1 = x_1 - x_0;
        let new_y1 = y_1 - y_0;

        let p1 = Point::new(new_x1 as f64, new_y1 as f64);
        let r = Point::new(r0 as f64, r_1 as f64);

        RadialAxialParams::Radial {
            p1, r
        }
    };

    (EncodedShadingType::RadialAxial {
        function: function.clone(),
        params,
        domain,
        extend,
    }, initial_transform)
}

fn sample_triangles(triangles: &[Triangle], transform: Affine) -> FxHashMap<(u16, u16), ColorComponents> {
    let mut map = FxHashMap::default();

    for t in triangles {
        let t = {
            let p0 = transform * t.p0.point;
            let p1 = transform * t.p1.point;
            let p2 = transform * t.p2.point;

            let mut v0 = t.p0.clone();
            v0.point = p0;
            let mut v1 = t.p1.clone();
            v1.point = p1;
            let mut v2 = t.p2.clone();
            v2.point = p2;

            Triangle::new(v0, v1, v2)
        };

        let bbox = t.bounding_box();

        for y in (bbox.y0.floor() as u16)..(bbox.y1.ceil() as u16) {
            for x in (bbox.x0.floor() as u16)..(bbox.x1.ceil() as u16) {
                let point = Point::new(x as f64, y as f64);
                if t.contains_point(point) {
                    map.insert((x, y), t.interpolate(point));
                }
            }
        }
    }

    map
}

fn encode_function_shading(
    domain: &[f32; 4],
    function: &ShadingFunction,
) -> EncodedShadingType {
    let domain = kurbo::Rect::new(
        domain[0] as f64,
        domain[2] as f64,
        domain[1] as f64,
        domain[3] as f64,
    );

    EncodedShadingType::FunctionBased {
        domain,
        function: function.clone(),
    }
}

#[derive(Debug)]
pub(crate) enum RadialAxialParams {
    Axial,
    Radial {
        p1: Point,
        r: Point,
    },
}

#[derive(Debug)]
pub(crate) enum EncodedShadingType {
    FunctionBased {
        domain: kurbo::Rect,
        function: ShadingFunction,
    },
    RadialAxial {
        function: ShadingFunction,
        params: RadialAxialParams,
        domain: [f32; 2],
        extend: [bool; 2],
    },
    Sampled {
        samples: FxHashMap<(u16, u16), ColorComponents>,
        function: Option<ShadingFunction>,
    }
}

impl EncodedShadingType {
    pub(crate) fn eval(&self, pos: Point, bg_color: AlphaColor<Srgb>, color_space: &ColorSpace) -> AlphaColor<Srgb> {
        match self {
            EncodedShadingType::FunctionBased { domain, function } => {
                if !domain.contains(pos) {
                    bg_color
                } else {
                    let out = function
                        .eval(&smallvec![pos.x as f32, pos.y as f32])
                        .unwrap();
                    // TODO: Clamp out-of-range values.
                    color_space.to_rgba(&out, 1.0)
                }
            },
            EncodedShadingType::RadialAxial { function, params, domain, extend } => {
                let (t0, t1) = (domain[0], domain[1]);

                let mut t = match params {
                    RadialAxialParams::Axial => pos.x as f32,
                    RadialAxialParams::Radial { p1, r } => {
                        radial_pos(
                            &pos,
                            &p1,
                            *r,
                            extend[0],
                            extend[1],
                        )
                            .unwrap_or(f32::MIN)
                    }
                };

                if t == f32::MIN {
                    return bg_color;
                }

                if t < 0.0 {
                    if extend[0] {
                        t = 0.0;
                    } else {
                        return bg_color;
                    }
                } else if t > 1.0 {
                    if extend[1] {
                        t = 1.0;
                    } else {
                        return bg_color
                    }
                }

                let t = t0 + (t1 - t0) * t;

                let val = function.eval(&smallvec![t]).unwrap();

                let color = color_space.to_rgba(&val, 1.0);
                color
            }
            EncodedShadingType::Sampled { samples, function } => {
                let sample_point = (pos.x as u16, pos.y as u16);

                if let Some(color) = samples.get(&sample_point) {
                    if let Some(function) = function {
                        let val = function.eval(&color.to_smallvec()).unwrap();
                        color_space.to_rgba(&val, 1.0)
                    } else {
                        color_space.to_rgba(&color, 1.0)
                    }
                } else {
                    bg_color
                }
            }
        }
    }
}

fn ts_from_line_to_line(src1: Point, src2: Point, dst1: Point, dst2: Point) -> Affine {
    let unit_to_line1 = unit_to_line(src1, src2);
    let line1_to_unit = unit_to_line1.inverse();
    let unit_to_line2 = unit_to_line(dst1, dst2);

    unit_to_line2 * line1_to_unit
}

fn unit_to_line(p0: Point, p1: Point) -> Affine {
    Affine::new([
        p1.y - p0.y,
        p0.x - p1.x,
        p1.x - p0.x,
        p1.y - p0.y,
        p0.x,
        p0.y,
    ])
}


fn radial_pos(
    pos: &Point,
    p1: &Point,
    r: Point,
    min_extend: bool,
    max_extend: bool,
) -> Option<f32> {
    let r0 = r.x as f32;
    let dx = p1.x as f32;
    let dy = p1.y as f32;
    let dr = r.y as f32 - r0;

    let px = pos.x as f32;
    let py = pos.y as f32;

    let a = dx * dx + dy * dy - dr * dr;
    let b = -2.0 * (px * dx + py * dy + r0 * dr);
    let c = px * px + py * py - r0 * r0;

    let discriminant = b * b - 4.0 * a * c;

    // No solution available.
    if discriminant < 0.0 {
        return None;
    }

    if a.abs() < 1e-6 {
        if b.abs() < 1e-6 {
            return None;
        }

        let t = -c / b;

        if (!min_extend && t < 0.0) || (!max_extend && t > 1.0) {
            return None;
        }

        let r_t = r0 + dr * t;
        if r_t < 0.0 {
            return None;
        }

        return Some(t);
    }

    let sqrt_d = discriminant.sqrt();
    let t1 = (-b - sqrt_d) / (2.0 * a);
    let t2 = (-b + sqrt_d) / (2.0 * a);

    let max = t1.max(t2);
    let mut take_max = Some(max);
    let min = t1.min(t2);
    let mut take_min = Some(min);

    if (!min_extend && min < 0.0) || r0 + dr * min < 0.0 {
        take_min = None;
    }

    if (!max_extend && max > 1.0) || r0 + dr * max < 0.0 {
        take_max = None;
    }

    match (take_min, take_max) {
        (Some(_), Some(max)) => Some(max),
        (Some(min), None) => Some(min),
        (None, Some(max)) => Some(max),
        (None, None) => None,
    }
}