//! A Driver for the Waveshare 2.7" E-Ink Display via SPI
//!
//! # References
//!
//! - [Waveshare product page](https://www.waveshare.com/wiki/2.7inch_e-Paper_HAT)
//! - [Waveshare C driver](https://github.com/waveshare/e-Paper/blob/master/RaspberryPi%26JetsonNano/c/lib/e-Paper/EPD_2in7.c)
//! - [Waveshare Python driver](https://github.com/waveshare/e-Paper/blob/master/RaspberryPi%26JetsonNano/python/lib/waveshare_epd/epd2in7.py)
//!

use embedded_hal::{
    blocking::{delay::*, spi::Write},
    digital::v2::{InputPin, OutputPin},
};

use crate::color::Color;
use crate::interface::DisplayInterface;
use crate::traits::{InternalWiAdditions, RefreshLut, WaveshareDisplay};

pub(crate) mod command;
use self::command::Command;

#[cfg(feature = "graphics")]
mod graphics;
#[cfg(feature = "graphics")]
pub use self::graphics::Display2in7;

pub(crate) mod constants;
use self::constants::*;

/// Width of the display.
pub const WIDTH: u32 = 264;

/// Height of the display
pub const HEIGHT: u32 = 176;

/// Default Background Color
pub const DEFAULT_BACKGROUND_COLOR: Color = Color::White;
const IS_BUSY_LOW: bool = true;

/// Epd2in7 driver
///
pub struct Epd2in7<SPI, CS, BUSY, DC, RST, DELAY> {
    /// Connection Interface
    interface: DisplayInterface<SPI, CS, BUSY, DC, RST, DELAY>,

    /// Background Color
    color: Color,
}

impl<SPI, CS, BUSY, DC, RST, DELAY> InternalWiAdditions<SPI, CS, BUSY, DC, RST, DELAY>
    for Epd2in7<SPI, CS, BUSY, DC, RST, DELAY>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayMs<u8>,
{
    fn init(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.interface.reset(delay, 2);

        self.interface.cmd_with_data(
            spi,
            Command::PowerSetting,
            &[0x03, 0x00, 0x2B, 0x2B, 0x09],
        )?;

        self.interface
            .cmd_with_data(spi, Command::BoosterSoftStart, &[0x07, 0x07, 0x17])?;

        // Power optimization
        self.interface
            .cmd_with_data(spi, Command::PowerOptimization, &[0x60, 0xA5])?;
        self.interface
            .cmd_with_data(spi, Command::PowerOptimization, &[0x89, 0xA5])?;
        self.interface
            .cmd_with_data(spi, Command::PowerOptimization, &[0x90, 0x00])?;
        self.interface
            .cmd_with_data(spi, Command::PowerOptimization, &[0x93, 0x2A])?;
        self.interface
            .cmd_with_data(spi, Command::PowerOptimization, &[0xA0, 0xA5])?;
        self.interface
            .cmd_with_data(spi, Command::PowerOptimization, &[0xA1, 0x00])?;
        self.interface
            .cmd_with_data(spi, Command::PowerOptimization, &[0x73, 0x41])?;

        self.interface
            .cmd_with_data(spi, Command::PartialDisplayRefresh, &[0x00])?;

        self.interface.cmd(spi, Command::PowerOn)?;
        self.interface.wait_until_idle(IS_BUSY_LOW);

        self.interface
            .cmd_with_data(spi, Command::PanelSetting, &[0xAF])?;
        self.interface
            .cmd_with_data(spi, Command::PllControl, &[0x3A])?;
        self.interface
            .cmd_with_data(spi, Command::VcomDcSettingRegister, &[0x12])?;

        self.set_lut(spi, None)?;

        Ok(())
    }
}

impl<SPI, CS, BUSY, DC, RST, DELAY> WaveshareDisplay<SPI, CS, BUSY, DC, RST, DELAY>
    for Epd2in7<SPI, CS, BUSY, DC, RST, DELAY>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayMs<u8>,
{
    type DisplayColor = Color;

    fn new(
        spi: &mut SPI,
        cs: CS,
        busy: BUSY,
        dc: DC,
        rst: RST,
        delay: &mut DELAY,
    ) -> Result<Self, SPI::Error> {
        let interface = DisplayInterface::new(cs, busy, dc, rst);
        let color = DEFAULT_BACKGROUND_COLOR;

        let mut epd = Epd2in7 { interface, color };

        epd.init(spi, delay)?;

        Ok(epd)
    }

    fn wake_up(&mut self, spi: &mut SPI, delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.init(spi, delay)
    }

    fn sleep(&mut self, spi: &mut SPI, _delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.cmd_with_data(spi, Command::VcomAndDataIntervalSetting, &[0xF7])?;
        self.cmd(spi, Command::PowerOff)?;
        self.cmd_with_data(spi, Command::DeepSleep, &[0xA5])?;
        Ok(())
    }

    fn update_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        _delay: &mut DELAY,
    ) -> Result<(), SPI::Error> {
        /*
        self.interface.cmd(spi, Command::DataStartTransmission1)?;
        self.send_buffer_helper(spi, buffer)?;
        */

        // Clear chromatic layer since we won't be using it here
        self.interface.cmd(spi, Command::DataStartTransmission2)?;
        self.send_buffer_helper(spi, buffer)?;

        self.interface.cmd(spi, Command::DataStop)?;
        Ok(())
    }

    fn update_partial_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<(), SPI::Error> {
        // NOTE: this is not documented, but it's copied from the epd2in7b and
        // seems to work.
        self.cmd(spi, Command::PartialDataStartTransmission1)?;

        self.send_data(spi, &[(x >> 8) as u8])?;
        self.send_data(spi, &[(x & 0xf8) as u8])?;
        self.send_data(spi, &[(y >> 8) as u8])?;
        self.send_data(spi, &[(y & 0xff) as u8])?;
        self.send_data(spi, &[(width >> 8) as u8])?;
        self.send_data(spi, &[(width & 0xf8) as u8])?;
        self.send_data(spi, &[(height >> 8) as u8])?;
        self.send_data(spi, &[(height & 0xff) as u8])?;
        self.wait_until_idle();

        self.send_buffer_helper(spi, buffer)?;

        self.cmd(spi, Command::DataStop)
    }

    fn display_frame(&mut self, spi: &mut SPI, _delay: &mut DELAY) -> Result<(), SPI::Error> {
        self.cmd(spi, Command::DisplayRefresh)?;
        self.wait_until_idle();
        Ok(())
    }

    fn update_and_display_frame(
        &mut self,
        spi: &mut SPI,
        buffer: &[u8],
        delay: &mut DELAY,
    ) -> Result<(), SPI::Error> {
        self.update_frame(spi, buffer, delay)?;
        self.display_frame(spi, delay)?;
        Ok(())
    }

    fn clear_frame(&mut self, spi: &mut SPI, _delay: &mut DELAY) -> Result<(), SPI::Error> {
        todo!();
    }

    fn set_background_color(&mut self, color: Color) {
        self.color = color;
    }

    fn background_color(&self) -> &Color {
        &self.color
    }

    fn width(&self) -> u32 {
        WIDTH
    }

    fn height(&self) -> u32 {
        HEIGHT
    }

    fn set_lut(
        &mut self,
        spi: &mut SPI,
        _refresh_rate: Option<RefreshLut>,
    ) -> Result<(), SPI::Error> {
        self.wait_until_idle();
        self.cmd_with_data(spi, Command::LutForVcom, &LUT_VCOM_DC)?;
        self.cmd_with_data(spi, Command::LutWhiteToWhite, &LUT_WW)?;
        self.cmd_with_data(spi, Command::LutBlackToWhite, &LUT_BW)?;
        self.cmd_with_data(spi, Command::LutWhiteToBlack, &LUT_WB)?;
        self.cmd_with_data(spi, Command::LutBlackToBlack, &LUT_BB)?;
        Ok(())
    }

    fn is_busy(&self) -> bool {
        self.interface.is_busy(IS_BUSY_LOW)
    }
}

impl<SPI, CS, BUSY, DC, RST, DELAY> Epd2in7<SPI, CS, BUSY, DC, RST, DELAY>
where
    SPI: Write<u8>,
    CS: OutputPin,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayMs<u8>,
{
    fn cmd(&mut self, spi: &mut SPI, command: Command) -> Result<(), SPI::Error> {
        self.interface.cmd(spi, command)
    }

    fn send_data(&mut self, spi: &mut SPI, data: &[u8]) -> Result<(), SPI::Error> {
        self.interface.data(spi, data)
    }

    fn send_buffer_helper(&mut self, spi: &mut SPI, buffer: &[u8]) -> Result<(), SPI::Error> {
        // Based on the waveshare implementation, all data for color values is flipped. This helper
        // method makes that transmission easier
        for b in buffer.iter() {
            self.send_data(spi, &[!b])?;
        }
        Ok(())
    }

    fn cmd_with_data(
        &mut self,
        spi: &mut SPI,
        command: Command,
        data: &[u8],
    ) -> Result<(), SPI::Error> {
        self.interface.cmd_with_data(spi, command, data)
    }

    fn wait_until_idle(&mut self) {
        self.interface.wait_until_idle(IS_BUSY_LOW);
    }
}
