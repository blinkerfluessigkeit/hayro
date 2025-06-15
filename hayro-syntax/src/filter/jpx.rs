//! A decoder for JPX-encoded images.

use crate::bit_reader::{BitSize, BitWriter};
use crate::filter::FilterResult;

/// Decode a JPX-encoded image stream.
#[cfg(feature = "jpeg2000")]
pub fn decode(data: &[u8]) -> Option<FilterResult> {
    use crate::filter::ImageColorSpace;

    let image = jpeg2k::Image::from_bytes(data).ok()?;
    let width = image.width();
    let height = image.height();
    let components = image.components();
    let bpc = components
        .iter()
        .fold(std::u32::MIN, |max, c| max.max(c.precision())) as u8;
    let cs = match components.iter().filter(|c| !c.is_alpha()).count() {
        1 => ImageColorSpace::Gray,
        3 => ImageColorSpace::Rgb,
        4 => ImageColorSpace::Cmyk,
        _ => return None,
    };
    let alpha = components.iter().flat_map(|c| if c.is_alpha() { Some(c) } else { None })
        .next().map(|c| c.data_u8().collect::<Vec<_>>());
    let mut components_iters = image
        .components()
        .iter()
        .flat_map(|c| if c.is_alpha() { None } else { Some(c.data_u8()) })
        .collect::<Vec<_>>();
    let mut buf = vec![];

    'outer: loop {
        for iter in &mut components_iters {
            if let Some(n) = iter.next() {
                buf.push(n);
            } else {
                break 'outer;
            }
        }
    }
    
    buf = if bpc == 8 { buf } else { scale(buf.as_slice(), bpc, cs.num_components(), width, height)? };

    Some(FilterResult {
        data: buf,
        alpha,
        color_space: Some(cs),
        bits_per_component: Some(bpc),
    })
}

/// A stub-method for decoding a JPX-encoded image stream. Always returns `None`.
#[cfg(not(feature = "jpeg2000"))]
pub fn decode(_: &[u8]) -> Option<FilterResult> {
    log::warn!("JPEG2000 images are not supported in the current build");

    None
}

fn scale(data: &[u8], bit_per_component: u8, num_components: u8, width: u32, height: u32) -> Option<Vec<u8>> {
    let mut input = vec![0; ((width + 1) * num_components as u32 * height) as usize];
    let bit_size = BitSize::from_u8(bit_per_component)?;
    let mut writer = BitWriter::new(&mut input, bit_size)?;
    
    let old_max = ((1 << 8) - 1) as f32;
    let new_max = ((1 << bit_per_component) - 1) as f32;
    
    for bytes in data.chunks_exact(num_components as usize * width as usize) {
        for byte in bytes {
            let scaled = ((*byte as f32 / old_max) * new_max) as u16;
            writer.write(scaled)?;
        }
        
        writer.align();
    }
    
    let final_pos = writer.cur_pos();
    input.truncate(final_pos);
    
    Some(input)
} 