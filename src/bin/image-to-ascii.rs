use std::{error::Error, fmt::Display};

use anyhow::Result;
use font_kit::{family_name::FamilyName, handle::Handle, properties::Properties};
use fontdue::{Font, FontSettings};
use image::GenericImageView;
use image_to_ascii::CharacterSet;

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

fn main() {
    let font = discover_monospace().unwrap();
    // println!("{}", average_brightness('@', &font));
    // let charset: String = " ▀▁▂▃▄▅▆▇█▉▊▋▌▍▎▏▐░▒▓▔▕▖▗▘▙▚▛▜▝▞▟🬀🬁🬂🬃🬄🬅🬆🬇🬈🬉🬊🬋🬌🬍🬎🬏🬐🬑🬒🬓🬔🬕🬖🬗🬘🬙🬚🬛🬜🬝🬞🬟🬠🬡🬢🬣🬤🬥🬦🬧🬨🬩🬪🬫🬬🬭🬮🬯🬰🬱🬲🬳🬴🬵🬶🬷🬸🬹🬺🬻🬼🬽🬾🬿🭀🭁🭂🭃🭄🭅🭆🭇🭈🭉🭊🭋🭌🭍🭎🭏🭐🭑🭒🭓🭔🭕🭖🭗🭘🭙🭚🭛🭜🭝🭞🭟🭠🭡🭢🭣🭤🭥🭦🭧🭨🭩🭪🭫🭬🭭🭮🭯🮐🮑🮒	🮔🮕🮖🮗🮘🮙🮚🮛🮜🮝🮞🮟🮀🮁🮂🮃🮄🮅🮆🮇🮈🮉🮊🮋🮌🮍🮎🮏🮰🮱🮲🮳🮴🮵🮶🮷🮸🮹🮺🮻🮼🮽🮾🮿🯀🯁🯂🯃🯄🯅🯆🯇🯈🯉🯊🯰🯱🯲🯳🯴🯵🯶🯷🯸🯹".to_string();
    
    let image_bytes = include_bytes!("./test.png");
    let image = image::load_from_memory(image_bytes).unwrap();
    
    let desired_width = 14;
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

    let image = image.to_luma16();

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
            let mut brightness = chunk
                .pixels()
                .map(|p| p.2 .0[0] as f32 / 65535.0)
                .sum::<f32>()
                / (chunk.width() * chunk.height()) as f32;

            let width = brightness_range.end - brightness_range.start;
            brightness *= width;
            brightness += brightness_range.start;

            chars.push(charset.nearest_brightness(brightness).unwrap_or(' '));
            left += horizontal_chunk_size;
        }
        top += vertical_chunk_size;
        left = 0;
    }

    for row in chars.chunks(image.width() as usize / horizontal_chunk_size as usize) {
        for c in row {
            print!("{c}");
        }
        println!();
    }
}
