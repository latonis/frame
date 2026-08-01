use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::spi::FullDuplex;

pub use embedded_sdmmc::{
    Controller,
    Error as FatError,
    SdMmcError,
    SdMmcSpi,
    TimeSource,
    Timestamp,
    VolumeIdx,
};

/// Adapts the blocking SPI proxy used by the display bus to the
/// FullDuplex<u8> interface required by embedded-sdmmc.
pub struct SpiFullDuplex<SPI> {
    spi: SPI,
    pending_read: Option<u8>,
}

impl<SPI> SpiFullDuplex<SPI> {
    pub fn new(spi: SPI) -> Self {
        Self {
            spi,
            pending_read: None,
        }
    }
}

impl<SPI> FullDuplex<u8> for SpiFullDuplex<SPI>
where
    SPI: Transfer<u8> + Write<u8, Error = <SPI as Transfer<u8>>::Error>,
{
    type Error = <SPI as Transfer<u8>>::Error;

    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        if let Some(byte) = self.pending_read.take() {
            return Ok(byte);
        }

        let mut byte = [0xFF];
        match self.spi.transfer(&mut byte) {
            Ok(_) => Ok(byte[0]),
            Err(error) => Err(nb::Error::Other(error)),
        }
    }

    fn send(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        // A blocking SPI transfer clocks both the outgoing and incoming byte.
        // Preserve the incoming byte so the following FullDuplex::read call
        // does not generate an extra clock cycle.
        let mut byte = [word];
        match self.spi.transfer(&mut byte) {
            Ok(_) => {
                self.pending_read = Some(byte[0]);
                Ok(())
            }
            Err(error) => Err(nb::Error::Other(error)),
        }
    }
}

/// A fixed timestamp source for filesystem operations.
///
/// The Pico firmware does not have a real-time clock yet, so files created by
/// the application will use this fixed timestamp until one is added.
pub struct FixedTimeSource;

impl TimeSource for FixedTimeSource {
    fn get_timestamp(&self) -> Timestamp {
        Timestamp {
            year_since_1970: 56,
            zero_indexed_month: 0,
            zero_indexed_day: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
        }
    }
}
