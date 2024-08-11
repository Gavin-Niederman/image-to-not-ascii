use std::{
    error::Error, fmt::Display, iter::Sum, ops::{Add, Div}
};

use anyhow::Result;
use font_kit::{family_name::FamilyName, handle::Handle, properties::Properties};
use fontdue::{Font, FontSettings};
use image::GenericImageView;
use image_to_ascii::{rgb_to_ansi256, CharacterSet};

#[derive(Debug)]
struct FontLoadError;
impl Error for FontLoadError {}
impl Display for FontLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to load font")
    }
}

fn discover_monospace() -> Result<Font> {
    let source = font_kit::source::SystemSource::new();
    let font = source.select_best_match(&[FamilyName::Monospace], &Properties::new())?;
    let font = match font {
        Handle::Path {
            path,
            font_index: _,
        } => {
            let bytes = std::fs::read(path)?;
            Font::from_bytes(bytes, FontSettings::default()).map_err(|_| FontLoadError)?
        }
        Handle::Memory {
            bytes,
            font_index: _,
        } => Font::from_bytes(bytes.as_slice(), FontSettings::default())
            .map_err(|_| FontLoadError)?,
    };
    Ok(font)
}

struct Rgba(f32, f32, f32, f32);
impl Rgba {
    pub fn from_rbga8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self(r as f32, g as f32, b as f32, a as f32)
    }
    pub fn brightness(&self) -> f32 {
        (self.0 + self.1 + self.2) / 3.0 / 255.0
    }
    pub fn alpha(&self) -> f32 {
        self.3 / 255.0
    }
    pub fn to_rgb8(&self) -> (u8, u8, u8) {
        (
            self.0.clamp(0.0, 255.0) as u8,
            self.1.clamp(0.0, 255.0) as u8,
            self.2.clamp(0.0, 255.0) as u8,
        )
    }
}
impl Sum for Rgba {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self(0.0, 0.0, 0.0, 0.0), |acc, x| acc + x)
    }
}
impl Add for Rgba {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0, self.1 + other.1, self.2 + other.2, self.3 + other.3)
    }
}
impl Div<f32> for Rgba {
    type Output = Self;

    fn div(self, rhs: f32) -> Self {
        Self(self.0 / rhs, self.1 / rhs, self.2 / rhs, self.3 / rhs)
    }
}

fn main() {
    let font = discover_monospace().unwrap();
    // println!("{}", average_brightness('@', &font));
    // let charset: String = " ▀▁▂▃▄▅▆▇█▉▊▋▌▍▎▏▐░▒▓▔▕▖▗▘▙▚▛▜▝▞▟🬀🬁🬂🬃🬄🬅🬆🬇🬈🬉🬊🬋🬌🬍🬎🬏🬐🬑🬒🬓🬔🬕🬖🬗🬘🬙🬚🬛🬜🬝🬞🬟🬠🬡🬢🬣🬤🬥🬦🬧🬨🬩🬪🬫🬬🬭🬮🬯🬰🬱🬲🬳🬴🬵🬶🬷🬸🬹🬺🬻🬼🬽🬾🬿🭀🭁🭂🭃🭄🭅🭆🭇🭈🭉🭊🭋🭌🭍🭎🭏🭐🭑🭒🭓🭔🭕🭖🭗🭘🭙🭚🭛🭜🭝🭞🭟🭠🭡🭢🭣🭤🭥🭦🭧🭨🭩🭪🭫🭬🭭🭮🭯🮐🮑🮒	🮔🮕🮖🮗🮘🮙🮚🮛🮜🮝🮞🮟🮀🮁🮂🮃🮄🮅🮆🮇🮈🮉🮊🮋🮌🮍🮎🮏🮰🮱🮲🮳🮴🮵🮶🮷🮸🮹🮺🮻🮼🮽🮾🮿🯀🯁🯂🯃🯄🯅🯆🯇🯈🯉🯊🯰🯱🯲🯳🯴🯵🯶🯷🯸🯹".to_string();

    let image_bytes = include_bytes!("./test.png");
    let image = image::load_from_memory(image_bytes).unwrap();

    let colored = true;

    let desired_width = 40;
    let charset = " ░▒▓█";
    // let charset = "!@#$%^&*()1234567890 ";
    let charset = CharacterSet::from_string(charset, font);

    // calculate chunk size based on desired width
    let vertical_chunk_size = image.width() / desired_width;
    let horizontal_chunk_size = vertical_chunk_size / 2;
    let brightness_range =
        (charset.lowest_brightness().unwrap().1)..(charset.highest_brightness().unwrap().1);

    let w_correction = horizontal_chunk_size - image.width() % horizontal_chunk_size;
    let h_correction = vertical_chunk_size - image.height() % vertical_chunk_size;
    let image = image.resize_exact(
        image.width() + w_correction,
        image.height() + h_correction,
        image::imageops::FilterType::CatmullRom,
    );

    let image = image.to_rgba8();

    let mut left = 0;
    let mut top = 0;
    let mut chars = Vec::new();
    while top < image.height() {
        while left < image.width() {
            let chunk = image.view(
                left,
                top,
                horizontal_chunk_size.min(image.width() - left),
                vertical_chunk_size.min(image.height() - top),
            );
            let avg_color = chunk
                .pixels()
                .map(|p| {
                    Rgba::from_rbga8(p.2.0[0], p.2.0[1], p.2.0[2], p.2.0[3])
                })
                .sum::<Rgba>()
                / (chunk.width() * chunk.height()) as f32;
            let mut alpha =  if colored { avg_color.alpha() } else { avg_color.brightness()};
            let width = brightness_range.end - brightness_range.start;
            alpha *= width;
            alpha += brightness_range.start;

            chars.push((charset.nearest_brightness(alpha).unwrap_or(' '), avg_color.to_rgb8()));
            left += horizontal_chunk_size;
        }
        top += vertical_chunk_size;
        left = 0;
    }

    for row in chars.chunks(image.width() as usize / horizontal_chunk_size as usize) {
        for (i, (c, color)) in row.iter().enumerate() {
            let escape_rgb = rgb_to_ansi256(color.0, color.1, color.2);
            let escape_code = format!("\x1b[38;5;{escape_rgb}m\x1b[38;2;{};{};{}m", color.0, color.1, color.2);
            if let Some((_, last_color)) = row.get(i.saturating_sub(1)) {
                if last_color != color || i == 0 && *c != ' ' {
                    print!("{escape_code}{c}")
                } else {
                    print!("{c}");
                }
            } else {
                print!("{escape_code}{c}")
            }
        }
        println!("\x1b[0m");
    }
}
