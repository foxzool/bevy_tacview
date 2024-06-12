#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

use bevy::prelude::*;
use bevy_octopus::prelude::*;

pub use parser::ParseError;
pub use systems::TacviewResource;
pub use writer::Writer;

use crate::systems::{send_header_after_connected, update_objects, ObjectNeedSync};

mod parser;
pub mod record;
pub mod systems;
mod writer;

pub const TACVIEW_CHANNEL: ChannelId = ChannelId("Tacview client");

pub struct TacviewPlugin;

impl Plugin for TacviewPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<OctopusPlugin>() {
            app.add_plugins(OctopusPlugin);
        }

        app.init_resource::<TacviewResource>()
            .register_type::<ObjectNeedSync>()
            .add_systems(Update, send_header_after_connected)
            .add_systems(Update, update_objects);
    }
}
