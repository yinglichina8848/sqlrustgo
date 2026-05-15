mod device;
mod record;
mod status;

pub use device::{CalibrationDevice, CalibrationInterval};
pub use record::{CalibrationRecord, CalibrationResult};
pub use status::CalibrationStatus;

pub const TABLE_CALIBRATION_DEVICES: &str = "gmp_calibration_devices";
pub const TABLE_CALIBRATION_RECORDS: &str = "gmp_calibration_records";
