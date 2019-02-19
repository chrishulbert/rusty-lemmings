use std::boxed::Box;

extern crate quicksilver;
use quicksilver::{
    Result,
    lifecycle::{Event, Window},
};

pub mod game_selection;
pub mod skill_selection;

// This is a list of actions for the scene to pass back to the controller, because rust's
// ownership model makes it almost impossible to use the delegate pattern instead.
pub enum EventAction {
    BeginFadeOut, // Start a scene transition. Since creating the new scene might be slow, we don't create the scene just yet, we fade out, then the controller asks us to create the next scene, then it fades it in. Old scene won't get 'update' called while transitioning.
    // TransitionToScene(Box<dyn Scene>),
}

// This is basically the same as quicksilver's State, but State can't be a
// `dyn` value because of it's `handle_error` func.
pub trait Scene {
    // Please call thread::yield_now() just before returning from this.
    fn update(&mut self, window: &mut Window) -> Result<()>;
    fn event(&mut self, event: &Event, window: &mut Window) -> Result<Vec<EventAction>>;
    fn draw(&mut self, window: &mut Window) -> Result<()>;
    fn next_scene(&mut self) -> Result<Option<Box<dyn Scene>>>;
}
