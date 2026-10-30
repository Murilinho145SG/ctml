use crate::style::WritingMode;

pub(super) fn layout_dimensions(mode: WritingMode, width: u32, height: u32) -> (u32, u32) {
    match mode {
        WritingMode::HorizontalTb => (width, height),
        WritingMode::VerticalRl | WritingMode::VerticalLr => (height, width),
    }
}

pub(super) fn layout_point(
    mode: WritingMode,
    width: u32,
    height: u32,
    x: f32,
    y: f32,
) -> (f32, f32) {
    match mode {
        WritingMode::HorizontalTb => (x, y),
        WritingMode::VerticalRl => (y, width as f32 - x),
        WritingMode::VerticalLr => (height as f32 - y, x),
    }
}

pub(super) fn orient_pixels(
    mode: WritingMode,
    pixels: Vec<u8>,
    width: u32,
    height: u32,
) -> Vec<u8> {
    if mode == WritingMode::HorizontalTb {
        return pixels;
    }

    let (source_width, source_height) = layout_dimensions(mode, width, height);
    let mut output = vec![0; width as usize * height as usize * 4];
    for source_y in 0..source_height {
        for source_x in 0..source_width {
            let (output_x, output_y) = match mode {
                WritingMode::HorizontalTb => unreachable!(),
                WritingMode::VerticalRl => (width - 1 - source_y, source_x),
                WritingMode::VerticalLr => (source_y, height - 1 - source_x),
            };
            let source = ((source_y * source_width + source_x) * 4) as usize;
            let destination = ((output_y * width + output_x) * 4) as usize;
            output[destination..destination + 4].copy_from_slice(&pixels[source..source + 4]);
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::{layout_dimensions, layout_point, orient_pixels};
    use crate::style::WritingMode;

    #[test]
    fn vertical_modes_rotate_the_complete_layout_in_opposite_directions() {
        let pixels = [1, 0, 0, 255, 2, 0, 0, 255, 3, 0, 0, 255, 4, 0, 0, 255];
        let right = orient_pixels(WritingMode::VerticalRl, pixels.to_vec(), 2, 2);
        let left = orient_pixels(WritingMode::VerticalLr, pixels.to_vec(), 2, 2);
        assert_eq!(
            right.chunks_exact(4).map(|p| p[0]).collect::<Vec<_>>(),
            [3, 1, 4, 2]
        );
        assert_eq!(
            left.chunks_exact(4).map(|p| p[0]).collect::<Vec<_>>(),
            [2, 4, 1, 3]
        );
    }

    #[test]
    fn vertical_pointer_coordinates_map_back_to_layout_coordinates() {
        assert_eq!(layout_dimensions(WritingMode::VerticalRl, 80, 30), (30, 80));
        assert_eq!(
            layout_point(WritingMode::VerticalRl, 80, 30, 10.0, 12.0),
            (12.0, 70.0)
        );
        assert_eq!(
            layout_point(WritingMode::VerticalLr, 80, 30, 10.0, 12.0),
            (18.0, 10.0)
        );
    }
}
