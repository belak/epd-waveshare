//! SPI Commands for the Waveshare 2.7"

use crate::traits;

/// Epd2in7
///
/// For more infos about the addresses and what they are doing look into the pdfs
#[allow(dead_code)]
#[derive(Copy, Clone)]
pub(crate) enum Command {
    PanelSetting = 0x00,
    PowerSetting = 0x01,
    PowerOff = 0x02,
    PowerOffSequenceSetting = 0x03,
    PowerOn = 0x04,
    PowerOnMeasure = 0x05,
    BoosterSoftStart = 0x06,
    DeepSleep = 0x07,
    DataStartTransmission1 = 0x10,
    DataStop = 0x11,
    DisplayRefresh = 0x12,
    PartialDataStartTransmission1 = 0x14,
    PartialDataStartTransmission2 = 0x15,
    PartialDisplayRefresh = 0x16,
    LutForVcom = 0x20,
    LutWhiteToWhite = 0x21,
    LutBlackToWhite = 0x22,
    LutWhiteToBlack = 0x23,
    LutBlackToBlack = 0x24,
    PllControl = 0x30,
    TemperatureSensorCommand = 0x40,
    TemperatureSensorCalibration = 0x41,
    TemperatureSensorWrite = 0x42,
    TemperatureSensorRead = 0x43,
    VcomAndDataIntervalSetting = 0x50,
    LowPowerDetection = 0x51,
    TconSetting = 0x60,
    TconResolution = 0x61,
    SourceAndGateStartSetting = 0x62,
    GetStatus = 0x71,
    AutoMeasureVcom = 0x80,
    VcomValue = 0x81,
    VcomDcSettingRegister = 0x82,
    ProgramMode = 0xA0,
    ActiveProgram = 0xA1,
    ReadOtpData = 0xA2,

    /// Not shown in commands table, but used in init sequence
    PowerOptimization = 0xF8,

    /// Not shown in commands table, but used in clear and display
    DataStartTransmission2 = 0x13,
}

impl traits::Command for Command {
    /// Returns the address of the command
    fn address(self) -> u8 {
        self as u8
    }
}
