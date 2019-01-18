extern crate quicksilver;
use quicksilver::{
    Result,
    lifecycle::{Event, Window},
};

pub mod game_selection;

// This is basically the same as quicksilver's State, but State can't be a
// `dyn` value because of it's `handle_error` func.
pub trait Scene {
    // Please call thread::yield_now() just before returning from this.
    fn update(&mut self, window: &mut Window) -> Result<()>;
    fn event(&mut self, event: &Event, window: &mut Window) -> Result<()>;
    fn draw(&mut self, window: &mut Window) -> Result<()>;
}
