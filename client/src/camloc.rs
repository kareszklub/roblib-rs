use anyhow::Result;
use roblib::cmd;

use crate::{transports::Transport, Robot};

impl<T: Transport> roblib::camloc::Camloc for Robot<T> {
    fn get_position(&self) -> Result<Option<roblib::camloc::Position>> {
        self.transport.cmd(cmd::GetPosition)
    }
}
