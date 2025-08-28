use std::collections::HashMap;
use std::time::Duration;
use web_sys::{InputEvent, KeyboardEvent, MouseEvent};
use yew::prelude::*;

use kiko::data::{PointSession, Session, SessionMessage};
use kiko::log;
use kiko::serde_json;

use crate::components::CopyUrlButton;

fn is_point_selected(selected_points: Option<u32>, points: u32) -> bool {
    selected_points
        == Some(points)
            .filter(|&p| p != 0)
            .or(if points == 0 { Some(0) } else { None })
}

#[derive(Properties, PartialEq)]
pub struct SessionViewProps {
    pub session: Session,
    pub on_refresh: Option<Callback<MouseEvent>>,
    pub on_send_message: Option<Callback<String>>,
    pub participant_name: Option<String>,
    pub is_joined: bool,
}

#[function_component(SessionView)]
pub fn session_view(props: &SessionViewProps) -> Html {
    let session = &props.session;
    let topic_input = use_state(String::new);
    let selected_points = use_state(|| None::<u32>);

    // Sync selected_points with session state when points are cleared
    use_effect_with(
        (
            session.current_points().clone(),
            props.participant_name.clone(),
        ),
        {
            let selected_points = selected_points.clone();
            let participants = session.participants().clone();
            move |(session_points, participant_name): &(
                HashMap<kiko::id::ParticipantId, Option<u32>>,
                Option<String>,
            )| {
                if let Some(name) = participant_name {
                    // Find participant ID by name
                    if let Some(participant) = participants.iter().find(|p| p.name() == name) {
                        let participant_id = participant.id();
                        // Check if this participant has points in the session
                        if !session_points.contains_key(participant_id) {
                            // No points for this participant, clear local state
                            selected_points.set(None);
                        }
                    }
                }
            }
        },
    );

    // WASM-compatible time functions using JavaScript Date API
    let get_current_timestamp = || -> u64 { (js_sys::Date::now() / 1000.0) as u64 };

    // State for live time updates
    let current_time = use_state(get_current_timestamp);

    // Update timer every second
    use_effect({
        let current_time = current_time.clone();
        move || {
            let interval = {
                let current_time = current_time.clone();
                gloo_timers::callback::Interval::new(1000, move || {
                    current_time.set(get_current_timestamp());
                })
            };

            // Cleanup function
            move || drop(interval)
        }
    });

    let is_active = {
        let elapsed = (*current_time).saturating_sub(session.started());
        elapsed < session.duration().as_secs()
    };

    let remaining = {
        let elapsed = (*current_time).saturating_sub(session.started());
        if elapsed < session.duration().as_secs() {
            Duration::from_secs(session.duration().as_secs() - elapsed)
        } else {
            Duration::from_secs(0)
        }
    };

    let format_duration = |duration: Duration| -> String {
        let total_seconds = duration.as_secs();
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;

        if hours > 0 {
            format!("{hours:02}:{minutes:02}:{seconds:02}")
        } else {
            format!("{minutes:02}:{seconds:02}")
        }
    };

    let format_timestamp = |timestamp: u64| -> String {
        let date = js_sys::Date::new(&((timestamp as f64) * 1000.0).into());
        date.to_locale_string("en-US", &js_sys::Object::new())
            .as_string()
            .unwrap_or_default()
    };

    html! {
        <div class="space-y-6">
            // Session Header Card
            <div class="bg-white border border-gray-200 rounded-lg p-6 shadow-sm">
                <div class="flex items-start justify-between mb-4">
                    <div>
                        <h2 class="text-2xl font-semibold text-gray-900 mb-2">{ session.name() }</h2>
                        <p class="text-sm text-gray-600">{ format!("Session ID: {}", session.id) }</p>
                    </div>
                    <div class="flex items-center space-x-2">
                        <span class={format!("inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {}",
                            if is_active { "bg-green-100 text-green-800" } else { "bg-red-100 text-red-800" })}>
                            { if is_active { "Active" } else { "Ended" } }
                        </span>
                        {
                            if let Some(on_refresh) = &props.on_refresh {
                                let callback = on_refresh.clone();
                                html! {
                                    <button
                                        class="p-1 text-gray-400 hover:text-gray-600 focus:outline-none focus:ring-2 focus:ring-blue-500 rounded"
                                        onclick={Callback::from(move |e| callback.emit(e))}
                                        title="Refresh session"
                                    >
                                        <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"></path>
                                        </svg>
                                    </button>
                                }
                            } else {
                                html! {}
                            }
                        }
                    </div>
                </div>

                // Session Timing Info
                <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                    <div class="bg-gray-50 rounded-lg p-4">
                        <h3 class="text-sm font-medium text-gray-700 mb-1">{ "Started" }</h3>
                        <p class="text-lg font-semibold text-gray-900">{ format_timestamp(session.started()) }</p>
                    </div>
                    <div class="bg-gray-50 rounded-lg p-4">
                        <h3 class="text-sm font-medium text-gray-700 mb-1">{ "Duration" }</h3>
                        <p class="text-lg font-semibold text-gray-900">{ format_duration(session.duration()) }</p>
                    </div>
                    <div class="bg-gray-50 rounded-lg p-4">
                        <h3 class="text-sm font-medium text-gray-700 mb-1">{ "Time Remaining" }</h3>
                        <p class={format!("text-lg font-semibold {}",
                            if is_active { "text-green-600" } else { "text-red-600" })}>
                            { if is_active { format_duration(remaining) } else { "Ended".to_string() } }
                        </p>
                    </div>
                </div>
            </div>

            // Participants Card
            <div class="bg-white border border-gray-200 rounded-lg p-6 shadow-sm">
                <div class="flex items-center justify-between mb-4">
                    <h3 class="text-lg font-semibold text-gray-900">
                        { format!("Participants ({})", session.participants().len()) }
                    </h3>
                </div>

                {
                    if session.participants().is_empty() {
                        html! {
                            <div class="text-center py-8">
                                <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"></path>
                                </svg>
                                <p class="mt-2 text-sm text-gray-500">{ "No participants yet" }</p>
                                <p class="text-xs text-gray-400">{ "Share this session link to invite participants" }</p>
                                <CopyUrlButton />
                            </div>
                        }
                    } else {
                        html! {
                            <div class="space-y-2">
                                {
                                    session.participants().iter().map(|participant| {
                                        html! {
                                            <div key={participant.id().to_string()} class="flex items-center p-3 bg-gray-50 rounded-lg">
                                                <div class="flex-shrink-0">
                                                    <div class="h-8 w-8 bg-blue-500 rounded-full flex items-center justify-center">
                                                        <span class="text-sm font-medium text-white">
                                                            { participant.name().chars().next().unwrap_or('?').to_uppercase().to_string() }
                                                        </span>
                                                    </div>
                                                </div>
                                                <div class="ml-3">
                                                    <p class="text-sm font-medium text-gray-900">{ participant.name() }</p>
                                                    <p class="text-xs text-gray-500">{ format!("ID: {}", participant.id()) }</p>
                                                </div>
                                            </div>
                                        }
                                    }).collect::<Html>()
                                }
                            </div>
                        }
                    }
                }
            </div>

            // Topic Setting Card (only show if joined)
            {
                if props.is_joined {
                    let on_topic_change = {
                        let on_send_message = props.on_send_message.clone();
                        let topic_input = topic_input.clone();
                        Callback::from(move |_: MouseEvent| {
                            if let Some(sender) = &on_send_message {
                                let topic_message = SessionMessage::SetTopic((*topic_input).clone());
                                if let Ok(message_text) = serde_json::to_string(&topic_message) {
                                    sender.emit(message_text);
                                    topic_input.set(String::new()); // Clear input after sending
                                }
                            }
                        })
                    };

                    let on_topic_keypress = {
                        let on_send_message = props.on_send_message.clone();
                        let topic_input = topic_input.clone();
                        Callback::from(move |e: KeyboardEvent| {
                            if e.key() == "Enter" {
                                if let Some(sender) = &on_send_message {
                                    let topic_message = SessionMessage::SetTopic((*topic_input).clone());
                                    if let Ok(message_text) = serde_json::to_string(&topic_message) {
                                        sender.emit(message_text);
                                        topic_input.set(String::new()); // Clear input after sending
                                    }
                                }
                            }
                        })
                    };

                    html! {
                        <div class="bg-white border border-gray-200 rounded-lg p-6 shadow-sm">
                            <h3 class="text-lg font-semibold text-gray-900 mb-4">{ "Current Topic" }</h3>
                            <div class="space-y-4">
                                <div class="flex items-center space-x-3">
                                    <input
                                        type="text"
                                        class="flex-1 px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                        placeholder="Enter discussion topic..."
                                        value={(*topic_input).clone()}
                                        oninput={{
                                            let topic_input = topic_input.clone();
                                            Callback::from(move |e: InputEvent| {
                                                if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                                                    topic_input.set(input.value());
                                                }
                                            })
                                        }}
                                        onkeypress={on_topic_keypress}
                                    />
                                    <button
                                        class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 text-sm font-medium"
                                        onclick={on_topic_change}
                                    >
                                        { "Set Topic" }
                                    </button>
                                </div>
                                {
                                    if !session.current_topic().is_empty() {
                                        html! {
                                            <div class="bg-blue-50 p-3 rounded-md">
                                                <p class="text-sm font-medium text-blue-900">{ "Current: " }{ session.current_topic() }</p>
                                            </div>
                                        }
                                    } else {
                                        html! {}
                                    }
                                }
                            </div>
                        </div>
                    }
                } else {
                    html! {}
                }
            }

            // Pointing/Voting Card (only show if joined and has participants)
            {
                if props.is_joined && !session.participants().is_empty() {
                    let points_options = [1, 2, 3, 5, 8, 13, 21, 0]; // 0 for "I don't know"

                    let on_point = {
                        let on_send_message = props.on_send_message.clone();
                        let participant_name = props.participant_name.clone();
                        let session_id = session.id.clone();
                        let participants = session.participants().clone();
                        let selected_points = selected_points.clone();
                        let participant_map: HashMap<String, _> = participants.iter()
                                .map(|p| (p.name().to_string(), p.id().to_string()))
                                .collect();
                        Callback::from(move |points: u32| {
                            // Update local state immediately
                            selected_points.set(Some(points));

                            if let (Some(sender), Some(name)) = (&on_send_message, &participant_name) {
                                if let Some(participant_id) = participant_map.get(name) {
                                    let point_value = if points == 0 { None } else { Some(points) };
                                    let point_message = SessionMessage::PointSession(PointSession {
                                        session_id: session_id.to_string(),
                                        participant_id: participant_id.to_string(),
                                        points: point_value,
                                    });
                                    if let Ok(message_text) = serde_json::to_string(&point_message) {
                                        sender.emit(message_text);
                                    }
                                } else {
                                    log::warn!("Participant name '{}' not found in session '{}'", name, session_id);
                                }
                            }
                        })
                    };

                    let on_clear_points = {
                        let on_send_message = props.on_send_message.clone();
                        let selected_points = selected_points.clone();
                        Callback::from(move |_: MouseEvent| {
                            // Clear local state immediately
                            selected_points.set(None);

                            if let Some(sender) = &on_send_message {
                                let clear_message = SessionMessage::ClearPoints;
                                if let Ok(message_text) = serde_json::to_string(&clear_message) {
                                    sender.emit(message_text);
                                }
                            }
                        })
                    };

                    let on_toggle_hide_points = {
                        let on_send_message = props.on_send_message.clone();
                        Callback::from(move |_: MouseEvent| {
                            if let Some(sender) = &on_send_message {
                                let toggle_message = SessionMessage::ToggleHidePoints;
                                if let Ok(message_text) = serde_json::to_string(&toggle_message) {
                                    sender.emit(message_text);
                                }
                            }
                        })
                    };

                    html! {
                        <div class="bg-white border border-gray-200 rounded-lg p-6 shadow-sm">
                            <div class="flex items-center justify-between mb-4">
                                <h3 class="text-lg font-semibold text-gray-900">{ "Story Points" }</h3>
                                <div class="flex items-center space-x-2">
                                    <button
                                        class={format!(
                                            "px-3 py-1 rounded-md focus:outline-none focus:ring-2 text-sm {}",
                                            if session.hide_points() {
                                                "bg-green-100 text-green-700 hover:bg-green-200 focus:ring-green-500"
                                            } else {
                                                "bg-blue-100 text-blue-700 hover:bg-blue-200 focus:ring-blue-500"
                                            }
                                        )}
                                        onclick={on_toggle_hide_points}
                                        title={if session.hide_points() { "Show Points" } else { "Hide Points" }}
                                    >
                                        { if session.hide_points() { "Show Points" } else { "Hide Points" } }
                                    </button>
                                    <button
                                        class="px-3 py-1 bg-red-100 text-red-700 rounded-md hover:bg-red-200 focus:outline-none focus:ring-2 focus:ring-red-500 text-sm"
                                        onclick={on_clear_points}
                                    >
                                        { "Clear All" }
                                    </button>
                                </div>
                            </div>

                            <div class="space-y-4">
                                <div class="grid grid-cols-4 md:grid-cols-8 gap-2">
                                    {
                                        points_options.iter().map(|&points| {
                                            let point_callback = {
                                                let on_point = on_point.clone();
                                                Callback::from(move |_: MouseEvent| {
                                                    on_point.emit(points);
                                                })
                                            };

                                            let is_selected = is_point_selected(*selected_points, points);

                                            html! {
                                                <button
                                                    key={points.to_string()}
                                                    class={format!(
                                                        "p-3 rounded-lg border-2 font-medium transition-colors {}",
                                                        if is_selected {
                                                            "bg-blue-600 text-white border-blue-600"
                                                        } else if points == 0 {
                                                            "bg-gray-100 text-gray-700 border-gray-300 hover:bg-gray-200"
                                                        } else {
                                                            "bg-white text-gray-900 border-gray-300 hover:border-blue-500 hover:bg-blue-50"
                                                        }
                                                    )}
                                                    onclick={point_callback}
                                                >
                                                    { if points == 0 { "?".to_string() } else { points.to_string() } }
                                                </button>
                                            }
                                        }).collect::<Html>()
                                    }
                                </div>

                                // Show current votes if any exist
                                {
                                    if !session.current_points().is_empty() {
                                        let participant_map: HashMap<String, _> = session.participants().iter()
                                            .map(|p| (p.id().to_string(), p.name()))
                                            .collect();
                                        html! {
                                            <div class="bg-gray-50 p-4 rounded-lg">
                                                <h4 class="text-sm font-medium text-gray-900 mb-3">{ "Current Votes:" }</h4>
                                                <div class="space-y-2">
                                                    {
                                                        session.current_points().iter().filter_map(|(participant_id, points)| {
                                                            participant_map.get(&participant_id.to_string()).map(|name| {
                                                                html! {
                                                                    <div key={participant_id.to_string()} class="flex items-center justify-between p-2 bg-white rounded border">
                                                                        <span class="text-sm font-medium">{ name }</span>
                                                                        <span class={format!(
                                                                            "px-2 py-1 rounded text-xs font-medium {}",
                                                                            if points.is_none() || session.hide_points() {
                                                                                "bg-gray-200 text-gray-700"
                                                                            } else {
                                                                                "bg-blue-100 text-blue-800"
                                                                            }
                                                                        )}>{
                                                                            if session.hide_points() {
                                                                                "•••".to_string()
                                                                            } else {
                                                                                match points {
                                                                                    Some(p) => p.to_string(),
                                                                                    None => "?".to_string(),
                                                                                }
                                                                            }
                                                                        }</span>
                                                                    </div>
                                                                }
                                                            })
                                                        }).collect::<Html>()
                                                    }
                                                </div>
                                            </div>
                                        }
                                    } else {
                                        html! {}
                                    }
                                }
                            </div>
                        </div>
                    }
                } else {
                    html! {}
                }
            }

        </div>
    }
}
