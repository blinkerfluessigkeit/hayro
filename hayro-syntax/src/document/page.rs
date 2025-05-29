//! Reading the pages of a PDF document.

use crate::content::{TypedIter, UntypedIter};
use crate::object::array::Array;
use crate::object::dict::Dict;
use crate::object::dict::keys::*;
use crate::object::name::Name;
use crate::object::rect::Rect;
use crate::object::r#ref::{MaybeRef, ObjRef};
use crate::object::stream::Stream;
use crate::object::{Object, ObjectLike};
use crate::xref::XRef;
use log::warn;
use std::cell::OnceCell;

/// A structure holding the pages of a PDF document.
pub struct Pages<'a> {
    /// The pages of the document.
    pub pages: Vec<Page<'a>>,
}

/// Attributes that can be inherited.
#[derive(Debug, Clone)]
struct PagesContext {
    media_box: Option<Rect>,
    crop_box: Option<Rect>,
    rotate: Option<u32>,
}

impl PagesContext {
    fn new() -> Self {
        Self {
            media_box: None,
            crop_box: None,
            rotate: None,
        }
    }
}

impl<'a> Pages<'a> {
    /// Create a new `Pages` object.
    pub(crate) fn new(pages_dict: Dict<'a>, xref: &'a XRef) -> Option<Pages<'a>> {
        let mut pages = vec![];
        let ctx = PagesContext::new();
        resolve_pages(
            pages_dict,
            &mut pages,
            ctx,
            Resources::new(Dict::empty(), None, xref),
        )?;

        Some(Self { pages })
    }

    /// The number of available pages.
    pub fn len(&self) -> usize {
        self.pages.len()
    }
}

fn resolve_pages<'a>(
    pages_dict: Dict<'a>,
    entries: &mut Vec<Page<'a>>,
    mut ctx: PagesContext,
    resources: Resources<'a>,
) -> Option<()> {
    if let Some(media_box) = pages_dict.get::<Rect>(MEDIA_BOX) {
        ctx.media_box = Some(media_box);
    }

    if let Some(crop_box) = pages_dict.get::<Rect>(CROP_BOX) {
        ctx.crop_box = Some(crop_box);
    }

    if let Some(rotate) = pages_dict.get::<u32>(ROTATE) {
        ctx.rotate = Some(rotate);
    }

    let resources = Resources::from_parent(
        pages_dict.get::<Dict>(RESOURCES).unwrap_or_default(),
        resources.clone(),
    );

    let kids = pages_dict.get::<Array<'a>>(KIDS)?;

    for dict in kids.iter::<Dict>() {
        match dict.get::<Name>(TYPE)? {
            PAGES => resolve_pages(dict, entries, ctx.clone(), resources.clone())?,
            PAGE => entries.push(Page::new(dict, &ctx, resources.clone())),
            _ => return None,
        }
    }

    Some(())
}

/// The rotation of the page.
#[derive(Debug, Copy, Clone)]
pub enum Rotation {
    /// No rotation.
    None,
    /// A rotation of 90 degrees.
    Horizontal,
    /// A rotation of 180 degrees.
    Flipped,
    /// A rotation of 270 degrees.
    FlippedHorizontal,
}

pub struct Page<'a> {
    inner: Dict<'a>,
    media_box: kurbo::Rect,
    crop_box: kurbo::Rect,
    rotation: Rotation,
    page_streams: OnceCell<Option<Vec<u8>>>,
    resources: Resources<'a>,
    xref: &'a XRef,
}

impl<'a> Page<'a> {
    fn new(dict: Dict<'a>, ctx: &PagesContext, resources: Resources<'a>) -> Page<'a> {
        let media_box = dict
            .get::<Rect>(MEDIA_BOX)
            .or_else(|| ctx.media_box)
            .unwrap_or(A4);

        let crop_box = dict
            .get::<Rect>(CROP_BOX)
            .or_else(|| ctx.crop_box)
            .unwrap_or(media_box);

        let rotation = match dict.get::<u32>(ROTATE).or_else(|| ctx.rotate).unwrap_or(0) % 360 {
            0 => Rotation::None,
            90 => Rotation::Horizontal,
            180 => Rotation::Flipped,
            270 => Rotation::FlippedHorizontal,
            _ => Rotation::None,
        };

        let xref = resources.xref;
        let resources =
            Resources::from_parent(dict.get::<Dict>(RESOURCES).unwrap_or_default(), resources);

        let crop_box = crop_box.get().intersect(media_box.get());

        Self {
            inner: dict,
            media_box: media_box.get(),
            crop_box,
            rotation,
            page_streams: OnceCell::new(),
            resources,
            xref,
        }
    }

    fn operations_impl(&self) -> Option<UntypedIter> {
        let convert_single = |s: Stream| {
            let data = s.decoded()?;
            Some(data.to_vec())
        };

        let stream = self
            .page_streams
            .get_or_init(|| {
                if let Some(stream) = self.inner.get::<Stream>(CONTENTS) {
                    convert_single(stream)
                } else if let Some(array) = self.inner.get::<Array>(CONTENTS) {
                    let streams = array.iter::<Stream>().flat_map(convert_single);

                    let mut collected = vec![];

                    for stream in streams {
                        collected.extend(stream);
                        // Streams must have at least one whitespace in-between.
                        collected.push(b' ')
                    }

                    Some(collected)
                } else {
                    warn!("contents entry of page was neither stream nor array of streams");

                    return None;
                }
            })
            .as_ref()?;

        let iter = UntypedIter::new(&stream);

        Some(iter)
    }

    /// Get the resources of the page.
    pub fn resources(&self) -> &Resources<'a> {
        &self.resources
    }

    /// Get the media box of the page.
    pub fn media_box(&self) -> kurbo::Rect {
        self.media_box
    }

    /// Get the rotation of the page.
    pub fn rotation(&self) -> Rotation {
        self.rotation
    }

    /// Get the crop box of the page.
    pub fn crop_box(&self) -> kurbo::Rect {
        self.crop_box
    }

    /// Get the operations of the content stream of the page.
    pub fn operations(&self) -> UntypedIter {
        self.operations_impl().unwrap_or(UntypedIter::empty())
    }

    // TODO: Remove?
    /// Get the xref table (of the document the page belongs to).
    pub fn xref(&self) -> &'a XRef {
        self.xref
    }

    /// Return an iterator over the operators in the page's content stream.
    pub fn typed_operations(&self) -> TypedIter {
        TypedIter::new(self.operations().into_iter())
    }
}

/// A structure keeping track of the resources of a page.
#[derive(Clone, Debug)]
pub struct Resources<'a> {
    parent: Option<Box<Resources<'a>>>,
    xref: &'a XRef,
    ext_g_states: Dict<'a>,
    fonts: Dict<'a>,
    color_spaces: Dict<'a>,
    x_objects: Dict<'a>,
    patterns: Dict<'a>,
    shadings: Dict<'a>,
}

impl<'a> Resources<'a> {
    /// Create a new `Resources` object from a dictionary with a parent.
    pub fn from_parent(resources: Dict<'a>, parent: Resources<'a>) -> Resources<'a> {
        let xref = parent.xref;

        Self::new(resources, Some(parent), xref)
    }

    /// Create a new `Resources` object.
    pub fn new(
        resources: Dict<'a>,
        parent: Option<Resources<'a>>,
        xref: &'a XRef,
    ) -> Resources<'a> {
        let ext_g_states = resources.get::<Dict>(EXT_G_STATE).unwrap_or_default();
        let fonts = resources.get::<Dict>(FONT).unwrap_or_default();
        let color_spaces = resources.get::<Dict>(COLORSPACE).unwrap_or_default();
        let x_objects = resources.get::<Dict>(XOBJECT).unwrap_or_default();
        let patterns = resources.get::<Dict>(PATTERN).unwrap_or_default();
        let shadings = resources.get::<Dict>(SHADING).unwrap_or_default();

        let parent = parent.map(|r| Box::new(r));

        Self {
            parent,
            ext_g_states,
            fonts,
            color_spaces,
            x_objects,
            patterns,
            shadings,
            xref,
        }
    }

    /// Resolve an object reference to an object.
    #[allow(private_bounds)]
    pub fn resolve_ref<T: ObjectLike<'a>>(&self, ref_: ObjRef) -> Option<T> {
        self.xref.get(ref_.into())
    }

    fn get_resource<T: ObjectLike<'a>, U>(
        &self,
        name: &Name,
        dict: &Dict<'a>,
        mut cache: impl FnMut(ObjRef) -> Option<U>,
        mut resolve: impl FnMut(T) -> Option<U>,
    ) -> Option<U> {
        // TODO: Cache non-ref resources as well

        match dict.get_raw::<T>(name)? {
            MaybeRef::Ref(ref_) => {
                cache(ref_).or_else(|| self.xref.get::<T>(ref_.into()).and_then(|t| resolve(t)))
            }
            MaybeRef::NotRef(i) => resolve(i),
        }
    }

    // TODO: Refactor caching mechanism

    /// Get an external graphics state by name.
    pub fn get_ext_g_state<U>(
        &self,
        name: &Name,
        mut cache: Box<dyn FnMut(ObjRef) -> Option<U> + '_>,
        mut resolve: Box<dyn FnMut(Dict<'a>) -> Option<U> + '_>,
    ) -> Option<U> {
        self.get_resource::<Dict, U>(name, &self.ext_g_states, &mut cache, &mut resolve)
            .or_else(|| {
                self.parent
                    .as_ref()
                    .and_then(|p| p.get_ext_g_state::<U>(name, cache, resolve))
            })
    }

    /// Get a color space by name.
    pub fn get_color_space<U>(
        &self,
        name: &Name,
        mut cache: Box<dyn FnMut(ObjRef) -> Option<U> + '_>,
        mut resolve: Box<dyn FnMut(Object<'a>) -> Option<U> + '_>,
    ) -> Option<U> {
        self.get_resource::<Object, U>(name, &self.color_spaces, &mut cache, &mut resolve)
            .or_else(|| {
                self.parent
                    .as_ref()
                    .and_then(|p| p.get_color_space::<U>(name, cache, resolve))
            })
    }

    /// Get a font by name.
    pub fn get_font<U>(
        &self,
        name: &Name,
        mut cache: Box<dyn FnMut(ObjRef) -> Option<U> + '_>,
        mut resolve: Box<dyn FnMut(Dict<'a>) -> Option<U> + '_>,
    ) -> Option<U> {
        self.get_resource::<Dict, U>(name, &self.fonts, &mut cache, &mut resolve)
            .or_else(|| {
                self.parent
                    .as_ref()
                    .and_then(|p| p.get_font::<U>(name, cache, resolve))
            })
    }

    /// Get a pattern by name.
    pub fn get_pattern<U>(
        &self,
        name: &Name,
        mut cache: Box<dyn FnMut(ObjRef) -> Option<U> + '_>,
        mut resolve: Box<dyn FnMut(Dict<'a>) -> Option<U> + '_>,
    ) -> Option<U> {
        self.get_resource::<Dict, U>(name, &self.patterns, &mut cache, &mut resolve)
            .or_else(|| {
                self.parent
                    .as_ref()
                    .and_then(|p| p.get_pattern::<U>(name, cache, resolve))
            })
    }

    /// Get an x object by name.
    pub fn get_x_object<U>(
        &self,
        name: &Name,
        mut cache: Box<dyn FnMut(ObjRef) -> Option<U> + '_>,
        mut resolve: Box<dyn FnMut(Stream<'a>) -> Option<U> + '_>,
    ) -> Option<U> {
        self.get_resource::<Stream, U>(name, &self.x_objects, &mut cache, &mut resolve)
            .or_else(|| {
                self.parent
                    .as_ref()
                    .and_then(|p| p.get_x_object::<U>(name, cache, resolve))
            })
    }

    /// Get a shading by name.
    pub fn get_shading<U>(
        &self,
        name: &Name,
        mut cache: Box<dyn FnMut(ObjRef) -> Option<U> + '_>,
        mut resolve: Box<dyn FnMut(Object<'a>) -> Option<U> + '_>,
    ) -> Option<U> {
        self.get_resource::<Object, U>(name, &self.shadings, &mut cache, &mut resolve)
            .or_else(|| {
                self.parent
                    .as_ref()
                    .and_then(|p| p.get_shading::<U>(name, cache, resolve))
            })
    }
}

// <https://github.com/apache/pdfbox/blob/a53a70db16ea3133994120bcf1e216b9e760c05b/pdfbox/src/main/java/org/apache/pdfbox/pdmodel/common/PDRectangle.java#L38>
const POINTS_PER_INCH: f64 = 72.0;
const POINTS_PER_MM: f64 = 1.0 / (10.0 * 2.54) * POINTS_PER_INCH;

const A4: Rect = Rect(kurbo::Rect {
    x0: 0.0,
    y0: 0.0,
    x1: 210.0 * POINTS_PER_MM,
    y1: 297.0 * POINTS_PER_MM,
});
