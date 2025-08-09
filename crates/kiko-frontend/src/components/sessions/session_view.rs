use crate::components::CopyUrlButton;
use kiko::data::Session;
use std::time::Duration;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SessionViewProps {
    pub session: Session,
    pub on_refresh: Option<Callback<MouseEvent>>,
}

#[function_component(SessionView)]
pub fn session_view(props: &SessionViewProps) -> Html {
    let session = &props.session;

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

            // Session Actions (placeholder for future features)
            <div class="bg-white border border-gray-200 rounded-lg p-6 shadow-sm">
                <h3 class="text-lg font-semibold text-gray-900 mb-4">{ "Session Actions" }</h3>
                <div class="space-y-3">
                    <button
                        class="w-full sm:w-auto px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50"
                        disabled={!is_active}
                    >
                        { "Join Session" }
                    </button>
                    <div class="text-sm text-gray-500">
                        { if is_active { "Click to join this pointing poker session" } else { "This session has expired" } }
                    </div>
                </div>
            </div>
        </div>
    }
}