use std::time::Duration;

use bevy::{
    app::ScheduleRunnerPlugin, log::LogPlugin, prelude::*, time::common_conditions::on_timer,
};
use bevy_octopus::connections::NetworkPeer;
use bevy_octopus::network::ListenTo;
use bevy_octopus::prelude::{ConnectTo, NetworkNode};
use bevy_tacview::{TacviewPlugin, TacviewResource, TACVIEW_CHANNEL};
use bytes::Bytes;
use chrono::Utc;

fn main() {
    let mut app = App::new();
    app.add_plugins(LogPlugin::default())
        .add_plugins(
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
                1.0 / 60.0,
            ))),
        );

    app.add_plugins(TacviewPlugin).add_systems(Startup, setup);

    app.run();
}

fn setup(mut host_res: ResMut<TacviewResource>, mut commands: Commands) {
    *host_res = TacviewResource {
        title: "bevy tacview sample".to_string(),
        category: "test".to_string(),
        author: "zool".to_string(),
        reference_time: Some(Utc::now()),
        recording_time: Some(Utc::now()),
        briefing: "hit".to_string(),
        debriefing: "live".to_string(),
        comments: "no comment".to_string(),
        data_source: "Tacview".to_string(),
        data_recorder: "TacviewHost Example".to_string(),
    };
    commands.spawn((TACVIEW_CHANNEL, ListenTo::new("tcp://0.0.0.0:42674")));
}
