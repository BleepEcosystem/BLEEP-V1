// Minimal stub for BLEEPLogger
pub struct BLEEPLogger;

impl BLEEPLogger {
    pub fn new() -> Self { BLEEPLogger }
    pub fn info(&self, _msg: &str) {}
    pub fn warning(&self, _msg: &str) {}
}
