use crate::{
    bindings::{hb_codepoint_t, hb_glyph_to_svg_path},
    font::Font,
    shape, Feature, GlyphPosition, HarfbuzzObject, UnicodeBuffer,
};
#[derive(Debug)]
pub struct GlyphInfoWithSvgPath {
    svg_path: String,
    position: GlyphPosition,
}

#[derive(Debug)]
pub struct BoundingBox {
    height: f64,
    width: f64,
}

impl<'a> Font<'a> {
    pub fn glyph_to_svg_path(&mut self, glyph: hb_codepoint_t) -> String {
        const PATH_BUFFER_SIZE: u32 = 65536; // should be enough for most glyphs
        let mut path_buffer: Vec<std::os::raw::c_char> = vec![0; PATH_BUFFER_SIZE as usize];
        unsafe {
            hb_glyph_to_svg_path(
                self.as_raw(),
                glyph,
                path_buffer.as_mut_ptr(),
                PATH_BUFFER_SIZE,
            );
        }
        let end_pos = path_buffer.iter().position(|&x| x == 0).unwrap();
        let u8_array: Vec<u8> = path_buffer[..end_pos].iter().map(|&x| x as u8).collect();
        String::from_utf8(u8_array).unwrap()
    }
    fn sharp_text_to_glyphs(
        &mut self,
        text: &str,
        features: &[Feature],
    ) -> Vec<GlyphInfoWithSvgPath> {
        self.set_scale(100, -100);

        let buffer = UnicodeBuffer::new()
            .add_str(text)
            .guess_segment_properties()
            .set_direction(crate::Direction::Ltr);
        let result_buffer = shape(self, buffer, features);
        let infos = result_buffer.get_glyph_infos();
        let positions = result_buffer.get_glyph_positions();
        let result: Vec<GlyphInfoWithSvgPath> = positions
            .iter()
            .zip(infos)
            .map(|(position, info)| GlyphInfoWithSvgPath {
                svg_path: self.glyph_to_svg_path(info.codepoint),
                position: *position,
            })
            .collect();
        result
    }
    pub fn render_svg_text(&mut self, text: &str, features: &[Feature]) -> String {
        let base_line = 24;
        let line_height = 1.0;
        let line_height_px = 100.0 * line_height;
        let rows: Vec<Vec<GlyphInfoWithSvgPath>> = text
            .split('\n')
            .map(|t| self.sharp_text_to_glyphs(t, features))
            .collect();

        let mut max_bounding = BoundingBox {
            height: 0.0,
            width: 0.0,
        };

        let mut bounding = BoundingBox {
            height: line_height_px,
            width: 0.0,
        };

        let paths: Vec<String> = rows
            .iter()
            .flat_map(|row| {
                let rendered_row: Vec<String> = row
                    .iter()
                    .map(|glyph| {
                        let path: String = format!(
                            r#"<path transform="translate({} {})" d="{}"></path>"#,
                            bounding.width,
                            bounding.height + glyph.position.y_advance as f64,
                            glyph.svg_path
                        );
                        bounding.width += glyph.position.x_advance as f64;
                        path
                    })
                    .collect();
                // set the maximum bounding box
                max_bounding.height = max_bounding.height.max(bounding.height as f64);
                max_bounding.width = max_bounding.width.max(bounding.width as f64);

                // reset row width
                bounding.height += line_height_px;
                bounding.width = 0.0;

                rendered_row
            })
            .collect();

        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="{}" height="{}" viewBox="0 0 {} {}">{}</svg>"#,
            max_bounding.width,
            max_bounding.height,
            max_bounding.width + base_line as f64,
            max_bounding.height + base_line as f64,
            paths.join("")
        )
    }
}
#[cfg(test)]
mod test {

    use crate::Face;

    use super::*;

    #[test]
    fn test_glyph_to_svg() {
        let path = "testfiles/SourceSansVariable-Roman.ttf";
        let face = Face::from_file(path, 0).unwrap();
        let mut font = Font::new(face);
        let graph_str = font.glyph_to_svg_path(b'h' as u32);
        assert_eq!(
            graph_str,
            "M100,0L100,660L454,660L454,632L132,632L132,366L402,366L402,338L132,338L132,28L464,28L464,0L100,0ZM185,710L171,726L267,808L295,808L391,726L377,710L283,778L279,778L185,710ZM143,844Q144,860 149.5,877.5Q155,895 169.5,907.5Q184,920 211,920Q237,920 255.5,912Q274,904 289,893Q304,882 318.5,874Q333,866 353,866Q374,866 383,880.5Q392,895 395,918L419,916Q418,900 412.5,882.5Q407,865 392.5,852.5Q378,840 351,840Q325,840 306.5,848Q288,856 273,867Q258,878 243.5,886Q229,894 209,894Q188,894 179,879.5Q170,865 167,842L143,844Z")
    }
    #[test]
    fn test_svg_text() {
        let path = "testfiles/SourceSansVariable-Roman.ttf";
        let face = Face::from_file(path, 0).unwrap();
        let mut font = Font::new(face);
        let features = [Feature::new(b"liga", 1, 0..10000)];
        let graph_str = font.render_svg_text("Hello World\nIt's me!\nliga feature fft!", &features);
        assert_eq!(graph_str.starts_with("<svg"), true);
        assert_eq!(graph_str.ends_with("</path></svg>"), true);
    }
}
