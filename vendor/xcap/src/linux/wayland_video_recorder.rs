use std::sync::mpsc::{self, Receiver};

use crate::{XCapError, XCapResult, video_recorder::Frame};

use super::impl_monitor::ImplMonitor;

#[derive(Clone, Debug)]
pub struct WaylandVideoRecorder {
    #[allow(dead_code)]
    monitor: ImplMonitor,
}

impl WaylandVideoRecorder {
    pub fn new(monitor: ImplMonitor) -> XCapResult<(Self, Receiver<Frame>)> {
        let (_sender, receiver) = mpsc::channel();
        Ok((Self { monitor }, receiver))
    }

    pub fn start(&self) -> XCapResult<()> {
        Err(XCapError::NotSupported)
    }

    pub fn stop(&self) -> XCapResult<()> {
        Ok(())
    }
}
