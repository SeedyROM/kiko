use std::collections::HashMap;
use std::time::Duration;
use web_sys::{InputEvent, KeyboardEvent, MouseEvent};
use yew::prelude::*;

use kiko::data::{Participant, PointSession, Session, SessionMessage};
use kiko::serde_json;

use crate::components::{ConnectionIndicator, CopyUrlButton};
use crate::hooks::ConnectionState;

// CSS class constants
const CARD_CLASSES: &str = "bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl p-6 shadow-sm";
const TEXT_PRIMARY: &str = "text-gray-900 dark:text-gray-100";
const TEXT_SECONDARY: &str = "text-gray-600 dark:text-gray-400";
// const GRID_LAYOUT: &str = "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3";

fn is_point_selected(selected_points: Option<u32>, points: u32) -> bool {
    selected_points
        == Some(points)
            .filter(|&p| p != 0)
            .or(if points == 0 { Some(0) } else { None })
}

// Utility function for sending WebSocket messages
fn send_session_message(sender: &Option<Callback<String>>, message: SessionMessage) -> bool {
    if let Some(sender) = sender {
        if let Ok(message_text) = serde_json::to_string(&message) {
            sender.emit(message_text);
            return true;
        }
    }
    false
}

// Participant Avatar Component
#[derive(Properties, PartialEq)]
struct ParticipantAvatarProps {
    participant: Participant,
}

#[function_component(ParticipantAvatar)]
fn participant_avatar(props: &ParticipantAvatarProps) -> Html {
    let initial = props
        .participant
        .name()
        .chars()
        .next()
        .unwrap_or('?')
        .to_uppercase()
        .to_string();

    html! {
        <div class="h-10 w-10 bg-blue-500 rounded-full flex items-center justify-center">
            <span class="text-sm font-medium text-white">
                { initial }
            </span>
        </div>
    }
}

// Edit Button Component
#[derive(Properties, PartialEq)]
struct EditButtonProps {
    onclick: Callback<MouseEvent>,
    #[prop_or("blue".to_string())]
    color: String,
}

#[function_component(EditButton)]
fn edit_button(props: &EditButtonProps) -> Html {
    let button_classes = if props.color == "blue" {
        "p-1.5 text-white bg-blue-600 hover:bg-blue-700 rounded-md transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 opacity-0 group-hover:opacity-100"
    } else {
        "p-1.5 text-white bg-gray-600 hover:bg-gray-700 dark:bg-gray-500 dark:hover:bg-gray-600 rounded-md transition-colors focus:outline-none focus:ring-2 focus:ring-gray-500 opacity-0 group-hover:opacity-100"
    };

    html! {
        <button
            class={button_classes}
            onclick={props.onclick.clone()}
            title="Edit topic"
        >
            <svg class="h-3 w-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
            </svg>
        </button>
    }
}

#[derive(Properties, PartialEq)]
pub struct SessionViewProps {
    pub session: Session,
    pub ws_state: ConnectionState,
    pub on_refresh: Option<Callback<MouseEvent>>,
    pub on_send_message: Option<Callback<String>>,
    pub participant_name: Option<String>,
    pub participant_id: Option<String>,
    pub is_joined: bool,
}

#[function_component(SessionView)]
pub fn session_view(props: &SessionViewProps) -> Html {
    let session = &props.session;
    let ws_state = &props.ws_state;
    let topic_input = use_state(String::new);
    let selected_points = use_state(|| None::<u32>);
    let show_topic_input = use_state(|| false);

    // Sync selected_points with session state when points are cleared
    use_effect_with(
        (
            session.current_points().clone(),
            props.participant_id.clone(),
        ),
        {
            let selected_points = selected_points.clone();
            move |(session_points, participant_id_str): &(
                HashMap<kiko::id::ParticipantId, Option<u32>>,
                Option<String>,
            )| {
                if let Some(id_str) = participant_id_str {
                    let participant_id: kiko::id::ParticipantId = id_str.clone().into();
                    // Check if this participant has points in the session
                    if !session_points.contains_key(&participant_id) {
                        // No points for this participant, clear local state
                        selected_points.set(None);
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
        <div class="min-h-screen bg-gray-50 dark:bg-gray-900 pb-8">
            // Compact Header Bar - Fixed at top
            <div class="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 sticky top-0 z-10 px-4 py-3 sm:px-6">
                <div class="max-w-7xl mx-auto">
                    <div class="flex items-center justify-between">
                        <div class="flex items-center space-x-4">
                            <h1 class="text-xl font-semibold text-gray-900 dark:text-gray-100">{ session.name() }</h1>
                            <ConnectionIndicator state={ws_state.clone()} />
                            <span class={format!("inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {}",
                                if is_active { "bg-green-100 dark:bg-green-900/40 text-green-800 dark:text-green-300" } else { "bg-red-100 dark:bg-red-900/40 text-red-800 dark:text-red-300" })}>
                                { if is_active { "Active" } else { "Ended" } }
                            </span>
                        </div>
                        <div class="flex items-center space-x-3">
                            <div class="text-right">
                                <div class="text-xs md:text-sm text-gray-600 dark:text-gray-400">{ "Time Remaining" }</div>
                                <div class={format!("text-md md:text-lg font-semibold {}",
                                    if is_active { "text-green-600 dark:text-green-400" } else { "text-red-600 dark:text-red-400" })}>
                                    { if is_active { format_duration(remaining) } else { "Ended".to_string() } }
                                </div>
                            </div>
                            {
                                if let Some(on_refresh) = &props.on_refresh {
                                    let callback = on_refresh.clone();
                                    html! {
                                        <button
                                            class="p-2 text-gray-400 dark:text-gray-500 hover:text-gray-600 dark:hover:text-gray-300 focus:outline-none focus:ring-2 focus:ring-blue-500 rounded-md"
                                            onclick={Callback::from(move |e| callback.emit(e))}
                                            title="Refresh session"
                                        >
                                            <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
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
                </div>
            </div>

            <div class="pt-6 px-4 md:px-6">
                // Main Content Layout - 2 Column on Desktop
                <div class="mx-auto max-w-7xl flex flex-col-reverse lg:grid lg:grid-cols-4 gap-6">
                    // Left Column - Session Controls & Topic
                    <div class="lg:col-span-1 space-y-6">


                        // Session Details - Compact
                        <div class={CARD_CLASSES}>
                            <h3 class={classes!("text-lg", "font-semibold", "mb-4", TEXT_PRIMARY)}>{ "Session Details" }</h3>
                            <div class="space-y-3 text-sm flex lg:block gap-x-4 justify-between">
                                <div>
                                    <div class={classes!("block", "mb-1", TEXT_SECONDARY)}>{ "Session ID" }</div>
                                    <div class={classes!("font-mono", "text-xs", "bg-gray-100", "dark:bg-gray-700", "px-2", "py-1", "rounded", TEXT_PRIMARY)}>{ format!("{}", session.id) }</div>
                                </div>
                                <div>
                                    <div class={classes!("block", "mb-1", TEXT_SECONDARY)}>{ "Started" }</div>
                                    <div class={classes!(TEXT_PRIMARY)}>{ format_timestamp(session.started()) }</div>
                                </div>
                                <div>
                                    <div class={classes!("block", "mb-1", TEXT_SECONDARY)}>{ "Duration" }</div>
                                    <div class={TEXT_PRIMARY}>{ format_duration(session.duration()) }</div>
                                </div>
                                <div>
                                    <div class={classes!("block", "mb-1", TEXT_SECONDARY)}>{ "Participants" }</div>
                                    <div class={classes!("font-semibold", "text-right", "lg:text-left", TEXT_PRIMARY)}>{ session.participants().len() }</div>
                                </div>
                            </div>
                            <div class="mt-4 pt-3 border-t border-gray-200 dark:border-gray-600">
                                <CopyUrlButton />
                            </div>
                        </div>
                    </div>

                    // Right Column - Participants & Voting
                    <div class="lg:col-span-3 space-y-6">
                        // Topic Section - Always show if there are participants
                        <div class="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-xl p-6 mb-6 shadow-sm">
                            <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">{ "Story Topic" }</h3>
                            {
                                if !session.current_topic().is_empty() {
                                    html! {
                                        <div class={format!("bg-blue-50 dark:bg-blue-900/20 {} border border-blue-200 dark:border-blue-800 p-4 rounded-lg mb-4 relative group",
                                            if props.is_joined { "flex justify-between" } else { "" })}>
                                            <p class="text-blue-900 dark:text-blue-300 font-medium pr-8">{ session.current_topic() }</p>
                                            {
                                                if props.is_joined {
                                                    let toggle_topic_input = {
                                                        let show_topic_input = show_topic_input.clone();
                                                        Callback::from(move |_: MouseEvent| {
                                                            show_topic_input.set(!*show_topic_input);
                                                        })
                                                    };
                                                    html! {
                                                        <button
                                                            class="p-1.5 text-white bg-blue-600 hover:bg-blue-700 rounded-md transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 opacity-0 group-hover:opacity-100"
                                                            onclick={toggle_topic_input}
                                                            title="Edit topic"
                                                        >
                                                            <svg class="h-3 w-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                                                            </svg>
                                                        </button>
                                                    }
                                                } else {
                                                    html! {}
                                                }
                                            }
                                        </div>
                                    }
                                } else {
                                    html! {
                                        <div class={format!("bg-gray-50 dark:bg-gray-700 {} border border-gray-200 dark:border-gray-600 p-4 rounded-lg mb-4 relative group",
                                            if props.is_joined { "flex justify-between" } else { "" })}>
                                            <p class="text-gray-500 dark:text-gray-400 italic pr-8">{ "No topic set yet" }</p>
                                            {
                                                if props.is_joined {
                                                    let toggle_topic_input = {
                                                        let show_topic_input = show_topic_input.clone();
                                                        Callback::from(move |_: MouseEvent| {
                                                            show_topic_input.set(!*show_topic_input);
                                                        })
                                                    };
                                                    html! {
                                                        <button
                                                            class="p-1.5 text-white bg-gray-600 hover:bg-gray-700 dark:bg-gray-500 dark:hover:bg-gray-600 rounded-md transition-colors focus:outline-none focus:ring-2 focus:ring-gray-500 opacity-0 group-hover:opacity-100"
                                                            onclick={toggle_topic_input}
                                                            title="Edit topic"
                                                        >
                                                            <svg class="h-3 w-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                                                            </svg>
                                                        </button>
                                                    }
                                                } else {
                                                    html! {}
                                                }
                                            }
                                        </div>
                                    }
                                }
                            }
                            {
                                if props.is_joined && *show_topic_input {
                                    let on_topic_change = {
                                        let on_send_message = props.on_send_message.clone();
                                        let topic_input = topic_input.clone();
                                        Callback::from(move |_: MouseEvent| {
                                            let topic_message = SessionMessage::SetTopic((*topic_input).clone());
                                            if send_session_message(&on_send_message, topic_message) {
                                                topic_input.set(String::new());
                                            }
                                        })
                                    };

                                    let on_topic_keypress = {
                                        let on_send_message = props.on_send_message.clone();
                                        let topic_input = topic_input.clone();
                                        Callback::from(move |e: KeyboardEvent| {
                                            if e.key() == "Enter" {
                                                let topic_message = SessionMessage::SetTopic((*topic_input).clone());
                                                if send_session_message(&on_send_message, topic_message) {
                                                    topic_input.set(String::new());
                                                }
                                            }
                                        })
                                    };
                                    html! {
                                        <div class="flex space-x-2">
                                            <input
                                                type="text"
                                                class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                                placeholder="Enter story to estimate..."
                                                value={(*topic_input).clone()}
                                                oninput={{
                                                    let topic_input = topic_input.clone();
                                                    Callback::from(move |e: InputEvent| {
                                                        if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                                                            topic_input.set(input.value());
                                                        }
                                                    })
                                                }}
                                                onkeypress={{
                                                    let on_topic_keypress = on_topic_keypress.clone();
                                                    let show_topic_input = show_topic_input.clone();
                                                    Callback::from(move |e: KeyboardEvent| {
                                                        if e.key() == "Enter" {
                                                            on_topic_keypress.emit(e);
                                                            show_topic_input.set(false); // Hide input after setting topic
                                                        }
                                                    })
                                                }}
                                            />
                                            <button
                                                class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 text-sm font-medium"
                                                onclick={{
                                                    let on_topic_change = on_topic_change.clone();
                                                    let show_topic_input = show_topic_input.clone();
                                                    Callback::from(move |e: MouseEvent| {
                                                        on_topic_change.emit(e);
                                                        show_topic_input.set(false); // Hide input after setting topic
                                                    })
                                                }}
                                            >
                                                { "Set" }
                                            </button>
                                        </div>
                                    }
                                } else {
                                    html! {}
                                }
                            }
                        </div>


                        // Voting Section - Show votes to everyone, but only allow interaction if joined
                        {
                            if !session.participants().is_empty() {
                                let points_options = [1, 2, 3, 5, 8, 13, 21, 0]; // 0 for "I don't know"

                                let on_point = {
                                    let on_send_message = props.on_send_message.clone();
                                    let participant_id = props.participant_id.clone();
                                    let session_id = session.id.clone();
                                    let selected_points = selected_points.clone();
                                    Callback::from(move |points: u32| {
                                        // Update local state immediately
                                        selected_points.set(Some(points));

                                        if let (Some(sender), Some(id_str)) = (&on_send_message, &participant_id) {
                                            let point_value = if points == 0 { None } else { Some(points) };
                                            let point_message = SessionMessage::PointSession(PointSession {
                                                session_id: session_id.to_string(),
                                                participant_id: id_str.clone(),
                                                points: point_value,
                                            });
                                            if let Ok(message_text) = serde_json::to_string(&point_message) {
                                                sender.emit(message_text);
                                            }
                                        }
                                    })
                                };

                                let on_clear_points = {
                                    let on_send_message = props.on_send_message.clone();
                                    let selected_points = selected_points.clone();
                                    Callback::from(move |_: MouseEvent| {
                                        selected_points.set(None);
                                        send_session_message(&on_send_message, SessionMessage::ClearPoints);
                                    })
                                };

                                let on_toggle_hide_points = {
                                    let on_send_message = props.on_send_message.clone();
                                    Callback::from(move |_: MouseEvent| {
                                        send_session_message(&on_send_message, SessionMessage::ToggleHidePoints);
                                    })
                                };

                                html! {
                                    <div class={CARD_CLASSES}>
                                        <div class="flex items-center justify-between mb-6">
                                            <h3 class={classes!("text-xl", "font-semibold", TEXT_PRIMARY)}>
                                                { if props.is_joined { "Choose Your Estimate" } else { "Voting" } }
                                            </h3>
                                            {
                                                if props.is_joined {
                                                    html! {
                                                        <div class="flex items-center space-x-2">
                                                            <button
                                                                class={format!(
                                                                    "px-3 py-1 rounded-lg focus:outline-none focus:ring-2 text-sm transition-colors {}",
                                                                    if session.hide_points() {
                                                                        "bg-green-50 dark:bg-green-800/60 text-green-600 dark:text-green-200 hover:bg-green-100 dark:hover:bg-green-700/70 focus:ring-green-500"
                                                                    } else {
                                                                        "bg-blue-50 dark:bg-blue-800/60 text-blue-600 dark:text-blue-200 hover:bg-blue-100 dark:hover:bg-blue-700/70 focus:ring-blue-500"
                                                                    }
                                                                )}
                                                                onclick={on_toggle_hide_points}
                                                                title={if session.hide_points() { "Show Points" } else { "Hide Points" }}
                                                            >
                                                                { if session.hide_points() { "👁️ Show" } else { "🙈 Hide" } }
                                                            </button>
                                                            <button
                                                                class="px-3 py-1 bg-red-50 dark:bg-red-800/60 text-red-600 dark:text-red-200 rounded-lg hover:bg-red-100 dark:hover:bg-red-700/70 focus:outline-none focus:ring-2 focus:ring-red-500 text-sm transition-colors"
                                                                onclick={on_clear_points}
                                                            >
                                                                { "🗑️ Clear All" }
                                                            </button>
                                                        </div>
                                                    }
                                                } else {
                                                    html! {}
                                                }
                                            }
                                        </div>

                                        // Show voting buttons only for joined participants
                                        {
                                            if props.is_joined {
                                                html! {
                                                    <div class="grid grid-cols-4 md:grid-cols-8 gap-3 mb-6">
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
                                                                            "h-12 rounded-lg border-2 font-bold text-lg transition-all duration-200 transform hover:scale-105 focus:scale-105 {}",
                                                                            if is_selected {
                                                                                "bg-blue-600 text-white border-blue-600 shadow-lg ring-4 ring-blue-200 dark:ring-blue-800"
                                                                            } else if points == 0 {
                                                                                "bg-orange-50 dark:bg-orange-900/20 text-orange-700 dark:text-orange-300 border-orange-300 dark:border-orange-700 hover:bg-orange-100 dark:hover:bg-orange-800/40 hover:border-orange-400 dark:hover:border-orange-600"
                                                                            } else {
                                                                                "bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 border-gray-300 dark:border-gray-600 hover:border-blue-400 hover:bg-blue-50 dark:hover:bg-gray-600 shadow-sm hover:shadow"
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
                                                }
                                            } else {
                                                html! {
                                                }
                                            }
                                        }


                                        // Results Section - Show votes to everyone, including non-voters
                                        {
                                            if !session.participants().is_empty() {
                                                html! {
                                                    <div class={if props.is_joined { "border-t pt-6" } else { "" }}>
                                                        <h4 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
                                                            {
                                                                if props.is_joined {
                                                                    "Current Estimates"
                                                                } else {
                                                                    "Participant's Estimates"
                                                                }
                                                            }
                                                        </h4>
                                                        <div class="space-y-3">
                                                            {
                                                                session.participants().iter().map(|participant| {
                                                                    // Look up points for this participant
                                                                    let has_voted = session.current_points().contains_key(participant.id());
                                                                    let participant_points = if has_voted {
                                                                        session.current_points().get(participant.id()).copied().unwrap()
                                                                    } else {
                                                                        None
                                                                    };
                                                                    html! {
                                                                        <div key={participant.id().to_string()} class="flex items-center justify-between p-4 bg-gray-50 dark:bg-gray-700 rounded-lg">
                                                                            <div class="flex items-center space-x-3">
                                                                                <div class="h-10 w-10 bg-blue-500 rounded-full flex items-center justify-center">
                                                                                    <span class="text-sm font-medium text-white">
                                                                                        { participant.name().chars().next().unwrap_or('?').to_uppercase().to_string() }
                                                                                    </span>
                                                                                </div>
                                                                                <span class="text-sm font-medium text-gray-900 dark:text-gray-100">{ participant.name() }</span>
                                                                            </div>
                                                                            <span class={format!(
                                                                                "px-3 py-1 rounded-full text-sm font-bold {}",
                                                                                if !has_voted || participant_points.is_none() {
                                                                                    "bg-gray-200 dark:bg-gray-600 text-gray-700 dark:text-gray-300"
                                                                                } else {
                                                                                    match participant_points {
                                                                                        Some(_) => "bg-blue-100 dark:bg-blue-900/40 text-blue-800 dark:text-blue-300",
                                                                                        None => "bg-orange-100 dark:bg-orange-900/40 text-orange-800 dark:text-orange-300",
                                                                                    }
                                                                                }
                                                                            )}>{
                                                                                if !has_voted {
                                                                                    "—".to_string()
                                                                                } else if session.hide_points() {
                                                                                    "•••".to_string()
                                                                                } else {
                                                                                    match participant_points {
                                                                                        Some(0) => "🤷‍♀️".to_string(),
                                                                                        Some(p) => p.to_string(),
                                                                                        None => "?".to_string(),
                                                                                    }
                                                                                }
                                                                            }</span>
                                                                        </div>
                                                                    }
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
                                }
                            } else {
                                html! {}
                            }
                        }
                    </div>
                </div>
            </div>
        </div>
    }
}
