use crate::core::configuration::Configuration;
pub mod primitives;

/// Modulation contains the amplitude modulation data.
pub trait Modulation: Send {
    fn build(&mut self, config: Configuration);
    fn buffer(&self) -> &[u8];
    fn sent(&mut self) -> &mut usize;
}
