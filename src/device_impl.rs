use crate::{
    interface::{I2cInterface, ReadData, SpiInterface, WriteData},
    AccelerometerPowerMode, BitFlags, Bmi160, Error, GyroscopePowerMode, MagnetometerPowerMode,
    Register, SensorPowerMode, SlaveAddr, Status,
};

impl<I2C> Bmi160<I2cInterface<I2C>> {
    /// Create new instance of the BMI160 device communicating through I2C.
    pub fn new_with_i2c(i2c: I2C, address: SlaveAddr) -> Self {
        Bmi160 {
            iface: I2cInterface {
                i2c,
                address: address.addr(),
            },
        }
    }

    /// Destroy driver instance, return I2C bus.
    pub fn destroy(self) -> I2C {
        self.iface.i2c
    }
}

impl<SPI, CS> Bmi160<SpiInterface<SPI, CS>> {
    /// Create new instance of the BMI160 device communicating through SPI.
    pub fn new_with_spi(spi: SPI, chip_select: CS) -> Self {
        Bmi160 {
            iface: SpiInterface {
                spi,
                cs: chip_select,
            },
        }
    }

    /// Destroy driver instance, return SPI bus instance and chip select pin.
    pub fn destroy(self) -> (SPI, CS) {
        (self.iface.spi, self.iface.cs)
    }
}

impl<DI, CommE, PinE> Bmi160<DI>
where
    DI: ReadData<Error = Error<CommE, PinE>> + WriteData<Error = Error<CommE, PinE>>,
{
    /// Get chip ID
    pub async fn chip_id(&mut self) -> Result<u8, Error<CommE, PinE>> {
        self.iface.read_register(Register::CHIPID).await
    }

    /// Get sensor power mode
    pub async fn power_mode(&mut self) -> Result<SensorPowerMode, Error<CommE, PinE>> {
        let status = self.iface.read_register(Register::PMU_STATUS).await?;
        let accel = match status & (0b11 << 4) {
            0 => AccelerometerPowerMode::Suspend,
            0b10_0000 => AccelerometerPowerMode::LowPower,
            _ => AccelerometerPowerMode::Normal,
        };
        let magnet = match status & 0b11 {
            0 => MagnetometerPowerMode::Suspend,
            2 => MagnetometerPowerMode::LowPower,
            _ => MagnetometerPowerMode::Normal,
        };
        let gyro = match status & (0b11 << 2) {
            0 => GyroscopePowerMode::Suspend,
            0b1100 => GyroscopePowerMode::FastStartUp,
            _ => GyroscopePowerMode::Normal,
        };
        Ok(SensorPowerMode {
            accel,
            gyro,
            magnet,
        })
    }

    /// Get sensor status
    pub async fn status(&mut self) -> Result<Status, Error<CommE, PinE>> {
        let status = self.iface.read_register(Register::STATUS).await?;
        Ok(Status {
            accel_data_ready: (status & BitFlags::DRDY_ACC) != 0,
            gyro_data_ready: (status & BitFlags::DRDY_GYR) != 0,
            magnet_data_ready: (status & BitFlags::DRDY_MAG) != 0,
            nvm_ready: (status & BitFlags::NVM_RDY) != 0,
            foc_ready: (status & BitFlags::FOC_RDY) != 0,
            magnet_manual_op: (status & BitFlags::MAG_MAN_OP) != 0,
            gyro_self_test_ok: (status & BitFlags::GYR_SELF_TEST_OK) != 0,
        })
    }

    /// Configure accelerometer power mode
    pub async fn set_accel_power_mode(
        &mut self,
        mode: AccelerometerPowerMode,
    ) -> Result<(), Error<CommE, PinE>> {
        let cmd = match mode {
            AccelerometerPowerMode::Suspend => 0b0001_0000,
            AccelerometerPowerMode::Normal => 0b0001_0001,
            AccelerometerPowerMode::LowPower => 0b0001_0010,
        };
        self.iface.write_register(Register::CMD, cmd).await
    }

    /// Configure gyroscope power mode
    pub async fn set_gyro_power_mode(
        &mut self,
        mode: GyroscopePowerMode,
    ) -> Result<(), Error<CommE, PinE>> {
        let cmd = match mode {
            GyroscopePowerMode::Suspend => 0b0001_0100,
            GyroscopePowerMode::Normal => 0b0001_0101,
            GyroscopePowerMode::FastStartUp => 0b0001_0111,
        };
        self.iface.write_register(Register::CMD, cmd).await
    }

    /// Configure magnetometer power mode
    pub async fn set_magnet_power_mode(
        &mut self,
        mode: MagnetometerPowerMode,
    ) -> Result<(), Error<CommE, PinE>> {
        let cmd = match mode {
            MagnetometerPowerMode::Suspend => 0b0001_1000,
            MagnetometerPowerMode::Normal => 0b0001_1001,
            MagnetometerPowerMode::LowPower => 0b0001_1010,
        };
        self.iface.write_register(Register::CMD, cmd).await
    }
}
