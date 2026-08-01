use core::fmt::Debug;

use embedded_sdmmc::{BlockDevice, Controller, Error, Mode, TimeSource, Volume};

#[derive(Debug)]
pub enum BmpError<E: Debug> {
    Filesystem(Error<E>),
    InvalidFormat,
    UnsupportedFormat,
    UnexpectedEndOfFile,
}

fn read_exact<D, T>(
    controller: &mut Controller<D, T>,
    volume: &Volume,
    file: &mut embedded_sdmmc::File,
    buffer: &mut [u8],
) -> Result<(), BmpError<D::Error>>
where
    D: BlockDevice,
    D::Error: Debug,
    T: TimeSource,
{
    let mut offset = 0;
    while offset < buffer.len() {
        let count = controller
            .read(volume, file, &mut buffer[offset..])
            .map_err(BmpError::Filesystem)?;
        if count == 0 {
            return Err(BmpError::UnexpectedEndOfFile);
        }
        offset += count;
    }
    Ok(())
}

/// Loads an uncompressed 24-bit, 200x200 BMP into a 1-bit display buffer.
///
/// BMP pixels are stored as BGR and rows are normally stored bottom-up. Light
/// source pixels become set bits to match this display's pixel polarity.
pub fn load_bmp<D, T>(
    controller: &mut Controller<D, T>,
    volume: &mut Volume,
    filename: &str,
    output: &mut [u8; 5000],
) -> Result<(), BmpError<D::Error>>
where
    D: BlockDevice,
    D::Error: Debug,
    T: TimeSource,
{
    let root = controller
        .open_root_dir(volume)
        .map_err(BmpError::Filesystem)?;
    let mut file = controller
        .open_file_in_dir(volume, &root, filename, Mode::ReadOnly)
        .map_err(BmpError::Filesystem)?;

    let result = (|| {
        let mut header = [0u8; 54];
        read_exact(controller, volume, &mut file, &mut header)?;

        if header[0] != b'B' || header[1] != b'M' {
            return Err(BmpError::InvalidFormat);
        }

        let data_offset = u32::from_le_bytes([
            header[10], header[11], header[12], header[13],
        ]) as usize;
        let width = i32::from_le_bytes([header[18], header[19], header[20], header[21]]);
        let height = i32::from_le_bytes([header[22], header[23], header[24], header[25]]);
        let planes = u16::from_le_bytes([header[26], header[27]]);
        let bits_per_pixel = u16::from_le_bytes([header[28], header[29]]);
        let compression = u32::from_le_bytes([
            header[30], header[31], header[32], header[33],
        ]);

        if width != 200 || height.abs() != 200 || planes != 1 {
            return Err(BmpError::UnsupportedFormat);
        }
        if bits_per_pixel != 24 || compression != 0 {
            return Err(BmpError::UnsupportedFormat);
        }
        if data_offset < 54 {
            return Err(BmpError::InvalidFormat);
        }

        // Skip any extended header or palette data before pixel data.
        let mut remaining = data_offset - 54;
        let mut skip = [0u8; 32];
        while remaining > 0 {
            let count = remaining.min(skip.len());
            read_exact(controller, volume, &mut file, &mut skip[..count])?;
            remaining -= count;
        }

        let row_size = 200 * 3 + 3 & !3;
        let mut row = [0u8; 604];
        output.fill(0);

        for source_row in 0..200 {
            read_exact(controller, volume, &mut file, &mut row[..row_size])?;
            let output_row = if height > 0 { 199 - source_row } else { source_row };

            for x in 0..200 {
                let pixel = x * 3;
                let blue = row[pixel] as u32;
                let green = row[pixel + 1] as u32;
                let red = row[pixel + 2] as u32;
                let luminance = (red * 299 + green * 587 + blue * 114) / 1000;

                if luminance >= 128 {
                    let index = output_row * 200 + x;
                    output[index / 8] |= 1 << (7 - (index % 8));
                }
            }
        }

        Ok(())
    })();

    let _ = controller.close_file(volume, file);
    let _ = controller.close_dir(volume, root);
    result
}
