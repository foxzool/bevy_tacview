use bevy::prelude::*;
use bevy_octopus::connections::NetworkPeer;
use bevy_octopus::prelude::{ChannelId, NetworkNode};
use bevy_octopus::shared::{NetworkEvent, NetworkNodeEvent};
use chrono::{DateTime, SecondsFormat, Utc};

use crate::record::PropertyList;
use crate::{record::{Coords, Event, EventKind, GlobalProperty, Property, Record, Update}, TACVIEW_CHANNEL, Writer};

static REAL_TIME_PROTOCOL: &str = "XtraLib.Stream.0
Tacview.RealTimeTelemetry.0
Host hoevo
\0";

/// Tacview Resource
#[derive(Resource, Default)]
pub struct TacviewResource {
    /// Mission/flight title or designation.
    pub title: String,
    /// Category of the flight/mission.
    pub category: String,
    /// Author or operator who has created this recording.
    pub author: String,
    /// Base time (UTC) for the current mission. This time is combined with each frame offset (in seconds) to get the final absolute UTC time for each data sample.
    /// ReferenceTime=2011-06-02T05:00:00Z
    pub reference_time: Option<DateTime<Utc>>,
    /// Recording (file) creation (UTC) time.
    /// RecordingTime=2016-02-18T16:44:12Z
    pub recording_time: Option<DateTime<Utc>>,
    /// Free text containing the briefing of the flight/mission.
    /// Briefing=Destroy all SCUD launchers
    pub briefing: String,
    /// Free text containing the debriefing.
    /// Debriefing=Managed to stay ahead of the airplane.
    pub debriefing: String,
    /// Free comments about the flight. Do not forget to escape any end-of-line character you want to inject into the comments.
    /// Comments=Part of the recording is missing because of technical difficulties
    pub comments: String,
    /// Source simulator, control station or file format.
    /// DataSource=DCS 2.0.0.48763
    /// DataSource=GPX File
    pub data_source: String,
    /// Software or hardware used to record the data.
    /// DataRecorder=Tacview 1.5
    /// DataRecorder=Falcon 4.0
    pub data_recorder: String,
}

/// send header after connected
pub(crate) fn send_header_after_connected(
    mut network_events: EventReader<NetworkNodeEvent>,
    q_node: Query<(Entity, &NetworkNode), With<NetworkPeer>>,
    mut commands: Commands,
    tacview_res: Res<TacviewResource>,
) {
    for event in network_events.read() {
        if event.channel_id != TACVIEW_CHANNEL {
            continue;
        }
        match &event.event {
            NetworkEvent::Connected => {
                info!("Tacview Client Connected {:?}", event.node);
                if let Ok((e, net_node)) = q_node.get(event.node) {
                    net_node.send(REAL_TIME_PROTOCOL.as_bytes());
                    commands.entity(e).insert(NeedFullSync);
                    // meta
                    let meta = build_meta_data(&tacview_res);
                    net_node.send(&meta);
                } else {
                    warn!("Failed to get network node for {:?}", event.node)
                }
            }
            NetworkEvent::Disconnected => {
                info!("Tacview Client Disconnected");
            }
            NetworkEvent::Error(err) => {
                error!("net err: {:?}", err);
            }
            _ => {}
        }
    }
}

/// build tacview meta data
fn build_meta_data(host_res: &TacviewResource) -> Vec<u8> {
    let mut writer = Writer::new(vec![]).unwrap();
    writer
        .write(Record::GlobalProperty(GlobalProperty::Title(
            host_res.title.clone(),
        )))
        .unwrap();
    writer
        .write(Record::GlobalProperty(GlobalProperty::Category(
            host_res.category.clone(),
        )))
        .unwrap();
    writer
        .write(Record::GlobalProperty(GlobalProperty::Author(
            host_res.author.clone(),
        )))
        .unwrap();
    if let Some(time) = host_res.reference_time {
        writer
            .write(Record::GlobalProperty(GlobalProperty::ReferenceTime(
                time.to_rfc3339_opts(SecondsFormat::Secs, true),
            )))
            .unwrap();
    }
    if let Some(time) = host_res.recording_time {
        writer
            .write(Record::GlobalProperty(GlobalProperty::RecordingTime(
                time.to_rfc3339_opts(SecondsFormat::Secs, true),
            )))
            .unwrap();
    }
    writer
        .write(Record::GlobalProperty(GlobalProperty::Briefing(
            host_res.briefing.clone(),
        )))
        .unwrap();
    writer
        .write(Record::GlobalProperty(GlobalProperty::Debriefing(
            host_res.debriefing.clone(),
        )))
        .unwrap();
    writer
        .write(Record::GlobalProperty(GlobalProperty::Comments(
            host_res.comments.clone(),
        )))
        .unwrap();
    writer
        .write(Record::GlobalProperty(GlobalProperty::DataSource(
            host_res.data_source.clone(),
        )))
        .unwrap();
    writer
        .write(Record::GlobalProperty(GlobalProperty::DataRecorder(
            host_res.data_recorder.clone(),
        )))
        .unwrap();

    writer.into_inner()
}


#[derive(Component)]
pub struct NeedFullSync;

#[derive(Component, Debug, Reflect)]
pub enum ObjectNeedSync {
    Spawn,
    Update,
    LeftArea,
    Timeout,
    Destroy,
}


pub(crate) fn update_objects(
    time: Res<Time>,
    tacview_res: Res<TacviewResource>,
    q_objects: Query<(Entity, &ObjectNeedSync, &Coords, &PropertyList)>,
    q_node: Query<(Entity, &ChannelId, &NetworkNode, Option<&NeedFullSync>), With<NetworkPeer>>,
    mut commands: Commands,
) {
    for (e, channel_id, net_node, opt_full_sync) in q_node.iter() {
        if *channel_id != TACVIEW_CHANNEL {
            continue;
        }
        let mut w = Writer::new_empty(vec![]).unwrap();

        let need_full_sync = opt_full_sync.is_some();

        if need_full_sync {
            // meta
            let meta = build_meta_data(&tacview_res);
            net_node.send(&meta);
        }

        let frame_time = if let Some(recording_time) = tacview_res.recording_time {
            (Utc::now() - recording_time).num_milliseconds() as f64 / 1000.0
        } else {
            time.elapsed_seconds_f64()
        };
        w.write(Record::Frame(frame_time)).unwrap();


        for (entity, need_sync, coords, props_list) in q_objects.iter() {
            let mut props = vec![Property::T(coords.clone())];

            let sync_kind = if need_full_sync {
                &ObjectNeedSync::Spawn
            } else {
                need_sync
            };

            match sync_kind {
                ObjectNeedSync::Spawn => {
                    props.extend(props_list.0.clone());
                    w.write(Record::Update(Update {
                        id: entity.to_bits(),
                        props,
                    }))
                        .unwrap();
                }
                ObjectNeedSync::Update => {
                    w.write(Record::Update(Update {
                        id: entity.to_bits(),
                        props,
                    }))
                        .unwrap();
                }
                ObjectNeedSync::Destroy => {
                    w.write(Record::Remove(entity.to_bits())).unwrap();
                    w.write(Event {
                        kind: EventKind::Destroyed,
                        params: vec![entity.to_bits().to_string()],
                        text: None,
                    })
                        .unwrap();
                }
                ObjectNeedSync::LeftArea => {
                    w.write(Record::Remove(entity.to_bits())).unwrap();
                    w.write(Event {
                        kind: EventKind::LeftArea,
                        params: vec![entity.to_bits().to_string()],
                        text: None,
                    })
                        .unwrap();
                }
                ObjectNeedSync::Timeout => {
                    w.write(Record::Remove(entity.to_bits())).unwrap();
                    w.write(Event {
                        kind: EventKind::Timeout,
                        params: vec![entity.to_bits().to_string()],
                        text: None,
                    })
                        .unwrap();
                }
            }

            commands.entity(entity).remove::<ObjectNeedSync>();
        }

        net_node.send(&w.into_inner());

        if need_full_sync {
            commands.entity(e).remove::<NeedFullSync>();
        }
    }
}
