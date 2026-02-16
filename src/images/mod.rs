pub mod image1;
pub mod image2;

pub struct ImageEntry {
    pub name: &'static str,
    pub data: &'static [u8; 5000],
}

pub const IMAGES: &[ImageEntry] = &[
    ImageEntry {
        name: "Photo 1",
        data: &image1::IMAGE_DATA,
    },
    ImageEntry {
        name: "Photo 2",
        data: &image2::IMAGE_DATA,
    },
];

pub const IMAGE_COUNT: usize = IMAGES.len();
