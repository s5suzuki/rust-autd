pub mod primitives;

/// Modulation contains the amplitude modulation data.
pub struct Modulation {
    buffer: Vec<u8>,
    sent: usize,
}

impl Modulation {
    pub fn new(buffer: Vec<u8>) -> Self {
        Modulation { buffer, sent: 0 }
    }

    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    pub fn sent(&self) -> usize {
        self.sent
    }

    pub fn send(&mut self, sent: usize) {
        self.sent += sent;
    }
}
