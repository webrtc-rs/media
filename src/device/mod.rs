pub mod capture;
pub mod fs;
pub mod render;

use crate::error::Result;

pub type ResetFn<R> = Box<dyn FnMut(usize) -> R>;

// Writer defines an interface to handle
// the creation of media files
pub trait Writer {
    // Add the content of an RTP packet to the media
    fn write_rtp(&mut self, pkt: &rtp::packet::Packet) -> Result<()>;
    // close the media
    // Note: close implementation must be idempotent
    fn close(&mut self) -> Result<()>;
}
