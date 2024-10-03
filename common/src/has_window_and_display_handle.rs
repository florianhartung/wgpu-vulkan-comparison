//! It's pretty self-explanatory, right?

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

pub trait HasWindowAndDisplayHandle: HasWindowHandle + HasDisplayHandle {}

impl<T: HasWindowHandle + HasDisplayHandle> HasWindowAndDisplayHandle for T {}
