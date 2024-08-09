use fontdue::Font;
use itertools::Itertools;

pub struct CharacterSet {
    // Map of char to average brightness
    chars: Vec<(char, f32)>,
}
impl CharacterSet {
    pub fn new(chars: Vec<char>, font: Font) -> Self {
        let chars = chars
            .into_iter()
            .map(|c| (c, average_brightness(c, &font)))
            .sorted_by(|a, b| {
                PartialOrd::partial_cmp(&a.1, &b.1).unwrap_or(std::cmp::Ordering::Less)
            })
            .collect();
        Self { chars }
    }
    pub fn from_string(chars: &str, font: Font) -> Self {
        let chars = chars.chars().unique().collect();
        Self::new(chars, font)
    }
    pub fn chars(&self) -> impl Iterator<Item = &(char, f32)> {
        self.chars.iter()
    }

    pub fn highest_brightness(&self) -> Option<(char, f32)> {
        self.chars.last().copied()
    }
    pub fn lowest_brightness(&self) -> Option<(char, f32)> {
        self.chars.first().copied()
    }

    pub fn nearest_brightness(&self, brightness: f32) -> Option<char> {
        let index = self.chars.binary_search_by(|(_, c)| {
            c.partial_cmp(&brightness)
                .unwrap_or(std::cmp::Ordering::Less)
        });
        let index = match index {
            Ok(index) => index,
            Err(index) => {
                if index >= self.chars.len() {
                    self.chars.len() - 1
                } else {
                    index
                }
            }
        };
        let (c, _) = self.chars.get(index)?;

        Some(*c)
    }
}

pub fn average_brightness(glyph: char, font: &Font) -> f32 {
    let (metrics, bitmap) = font.rasterize(glyph, 100.0);

    if metrics.width == 0 || metrics.height == 0 {
        return 0.0;
    }

    let image = image::GrayImage::from_fn(100, 100, |x, y| {
        if x >= metrics.width as u32 || y >= metrics.height as u32 {
            return image::Luma([0]);
        }
        image::Luma([bitmap[(y * metrics.width as u32 + x) as usize]])
    });

    image
        .pixels()
        .map(|coverage| ((coverage.0[0] as f32) / 255.0))
        .sum::<f32>()
        / (image.width() * image.height()) as f32
}

pub fn rgb_to_ansi256(r: u8, g: u8, b: u8) -> u8 {
    // we use the extended greyscale palette here, with the exception of
    // black and white. normal palette only has 4 greyscale shades.
    if r == g && g == b {
        if r < 8 {
            return 16;
        }

        if r > 248 {
            return 231;
        }

        return (((r as f32 - 8.0) / 247.0) * 24.0).round() as u8 + 232;
    }

    16 + (36 * (r as f32 / 255.0 * 5.0).round() as u8)
        + (6 * (g as f32 / 255.0 * 5.0).round() as u8)
        + (b as f32 / 255.0 * 5.0).round() as u8
}
