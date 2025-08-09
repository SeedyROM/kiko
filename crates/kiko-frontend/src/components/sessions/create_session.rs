use std::time::Duration;

use web_sys::HtmlInputElement;
use yew::prelude::*;

use kiko::{
    async_callback,
    data::{self, Session},
};

use crate::providers::api;

#[derive(Properties, PartialEq)]
pub struct CreateSessionProps {
    /// Callback fired when a session is successfully created
    #[prop_or_default]
    pub on_session_created: Option<Callback<Session>>,
    /// Callback fired when the form is cancelled
    #[prop_or_default]
    pub on_cancel: Option<Callback<()>>,
    /// Whether to show the form in a modal style
    #[prop_or(false)]
    pub modal: bool,
}

#[function_component(CreateSession)]
pub fn create_session(props: &CreateSessionProps) -> Html {
    let api = use_memo((), |_| api::create());

    // Form state
    let session_name = use_state(String::new);
    let duration_hours = use_state(|| 0u32);
    let duration_minutes = use_state(|| 30u32); // Default to 30 minutes

    // UI state
    let loading = use_state(|| false);
    let error_msg = use_state(|| None::<String>);
    let success = use_state(|| false);

    // Form validation - fix the logic to work with defaults
    let is_valid = !session_name.is_empty()
        && (*duration_hours > 0 || *duration_minutes > 0)
        && (*duration_hours <= 24);

    // Input handlers
    let on_name_change = {
        let session_name = session_name.clone();
        let error_msg = error_msg.clone();
        Callback::from(move |e: InputEvent| {
            // <- Change to InputEvent
            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                session_name.set(input.value());
                error_msg.set(None);
            }
        })
    };

    let on_hours_change = {
        let duration_hours = duration_hours.clone();
        let error_msg = error_msg.clone();
        Callback::from(move |e: Event| {
            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                if let Ok(hours) = input.value().parse::<u32>() {
                    duration_hours.set(hours);
                    error_msg.set(None);
                }
            }
        })
    };

    let on_minutes_change = {
        let duration_minutes = duration_minutes.clone();
        let error_msg = error_msg.clone();
        Callback::from(move |e: Event| {
            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                if let Ok(minutes) = input.value().parse::<u32>() {
                    if minutes < 60 {
                        duration_minutes.set(minutes);
                        error_msg.set(None);
                    }
                }
            }
        })
    };

    // Submit handler - using manual approach that works
    let on_session_created = props.on_session_created.clone();
    let on_submit = async_callback!([
        api,
        session_name,
        duration_hours,
        duration_minutes,
        loading,
        error_msg,
        success,
        on_session_created,
    ] |e: SubmitEvent| {
        e.prevent_default();

        // Always prevent submission if form is invalid
        if session_name.is_empty() {
            error_msg.set(Some("Session name is required".to_string()));
            return;
        }

        let total_seconds = (*duration_hours * 3600) + (*duration_minutes * 60);
        if total_seconds == 0 {
            error_msg.set(Some("Duration must be greater than 0".to_string()));
            return;
        }

        if *duration_hours > 24 {
            error_msg.set(Some("Duration cannot exceed 24 hours".to_string()));
            return;
        }

        // Don't submit if already loading or successfully created
        if *loading || *success {
            return;
        }

        loading.set(true);
        error_msg.set(None);

        let create_request = data::CreateSession {
            name: (*session_name).clone(),
            duration: Duration::from_secs(total_seconds as u64),
        };

        match api.create_session(&create_request).await {
            Ok(session) => {
                loading.set(false);
                success.set(true);

                // Notify parent component
                if let Some(callback) = &on_session_created {
                    callback.emit(session.clone());
                }

                // Reset form after showing success message and open window
                let session_name = session_name.clone();
                let duration_hours = duration_hours.clone();
                let duration_minutes = duration_minutes.clone();
                let success = success.clone();
                let session_id = session.id.clone();

                wasm_bindgen_futures::spawn_local(async move {
                    // Wait 1.5 seconds
                    let promise = js_sys::Promise::new(&mut |resolve, _| {
                        let window = web_sys::window().unwrap();
                        window
                            .set_timeout_with_callback_and_timeout_and_arguments_0(
                                &resolve, 1500,
                            )
                            .unwrap();
                    });
                    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;

                    // Open new window with the session URL after delay
                    if let Some(window) = web_sys::window() {
                        let session_url = format!("/session/{session_id}");
                        let _ = window.open_with_url_and_target(&session_url, "_blank");
                    }

                    // Reset form
                    session_name.set(String::new());
                    duration_hours.set(1);
                    duration_minutes.set(0);
                    success.set(false);
                });
            }
            Err(err) => {
                loading.set(false);
                error_msg.set(Some(format!("Failed to create session: {err}")));
            }
        }
    });

    // Cancel handler
    let on_cancel_click = {
        let on_cancel = props.on_cancel.clone();
        Callback::from(move |_| {
            if let Some(callback) = &on_cancel {
                callback.emit(());
            }
        })
    };

    // Duration preset handler
    let set_preset_duration = {
        let duration_hours = duration_hours.clone();
        let duration_minutes = duration_minutes.clone();
        let error_msg = error_msg.clone();

        Callback::from(move |(hours, minutes): (u32, u32)| {
            duration_hours.set(hours);
            duration_minutes.set(minutes);
            error_msg.set(None);
        })
    };

    // CSS classes based on modal prop
    let container_classes = if props.modal {
        "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
    } else {
        "w-full"
    };

    let form_classes = if props.modal {
        "bg-white rounded-lg shadow-xl p-6 w-full max-w-md mx-4"
    } else {
        "bg-white border border-gray-200 rounded-lg  p-6"
    };

    html! {
        <div class={container_classes}>
            <div class={form_classes}>
                // Header
                <div class="mb-6">
                    <h2 class="text-xl font-semibold text-gray-900 mb-2">
                        { "Create New Session" }
                    </h2>
                    <p class="text-sm text-gray-600">
                        { "Set up a new pointing poker session for your team" }
                    </p>
                </div>

                <form onsubmit={on_submit} class="space-y-4">
                    // Session Name Input
                    <div>
                        <label for="session-name" class="block text-sm font-medium text-gray-700 mb-1">
                            { "Session Name" }
                        </label>
                        <input
                            id="session-name"
                            type="text"
                            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                            placeholder="Sprint Planning, Story Estimation..."
                            value={(*session_name).clone()}
                            oninput={on_name_change}
                            disabled={*loading}
                        />
                    </div>

                    // Duration Section
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">
                            { "Session Duration" }
                        </label>

                        // Duration inputs
                        <div class="flex items-center space-x-2 mb-2">
                            <div class="flex items-center space-x-1">
                                <input
                                    type="number"
                                    min="0"
                                    max="24"
                                    class="w-16 px-2 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 text-center"
                                    value={duration_hours.to_string()}
                                    onchange={on_hours_change}
                                    disabled={*loading}
                                    aria-label="Duration Hours"
                                />
                                <span class="text-sm text-gray-600">{ "hours" }</span>
                            </div>

                            <div class="flex items-center space-x-1">
                                <input
                                    type="number"
                                    min="0"
                                    max="59"
                                    class="w-16 px-2 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 text-center"
                                    value={duration_minutes.to_string()}
                                    onchange={on_minutes_change}
                                    disabled={*loading}
                                    aria-label="Duration Minutes"
                                />
                                <span class="text-sm text-gray-600">{ "minutes" }</span>
                            </div>
                        </div>

                        // Quick preset buttons
                        <div class="flex flex-wrap gap-1">
                            <button
                                type="button"
                                class="px-2 py-1 text-xs bg-gray-100 hover:bg-gray-200 rounded border text-gray-700 disabled:opacity-50"
                                onclick={set_preset_duration.reform(|_| (0, 10))}
                                disabled={*loading}
                            >
                                { "10m" }
                            </button>
                            <button
                                type="button"
                                class="px-2 py-1 text-xs bg-gray-100 hover:bg-gray-200 rounded border text-gray-700 disabled:opacity-50"
                                onclick={set_preset_duration.reform(|_| (0, 15))}
                                disabled={*loading}
                            >
                                { "15m" }
                            </button>
                            <button
                                type="button"
                                class="px-2 py-1 text-xs bg-gray-100 hover:bg-gray-200 rounded border text-gray-700 disabled:opacity-50"
                                onclick={set_preset_duration.reform(|_| (0, 30))}
                                disabled={*loading}
                            >
                                { "30m" }
                            </button>
                                                   <button
                                type="button"
                                class="px-2 py-1 text-xs bg-gray-100 hover:bg-gray-200 rounded border text-gray-700 disabled:opacity-50"
                                onclick={set_preset_duration.reform(|_| (0, 45))}
                                disabled={*loading}
                            >
                                { "45m" }
                            </button>
                            <button
                                type="button"
                                class="px-2 py-1 text-xs bg-gray-100 hover:bg-gray-200 rounded border text-gray-700 disabled:opacity-50"
                                onclick={set_preset_duration.reform(|_| (1, 0))}
                                disabled={*loading}
                            >
                                { "1h" }
                            </button>
                        </div>
                    </div>

                    // Error Message
                    {
                        if let Some(error) = error_msg.as_ref() {
                            html! {
                                <div class="p-3 bg-red-50 border border-red-200 rounded-md">
                                    <p class="text-sm text-red-700">{ error }</p>
                                </div>
                            }
                        } else {
                            html! {}
                        }
                    }

                    // Success Message
                    {
                        if *success {
                            html! {
                                <div class="p-3 bg-green-50 border border-green-200 rounded-md">
                                    <p class="text-sm text-green-700">
                                        { "✅ Session created successfully!" }
                                    </p>
                                </div>
                            }
                        } else {
                            html! {}
                        }
                    }

                    // Form Actions
                    <div class="flex justify-end space-x-3 pt-4">
                        {
                            if props.on_cancel.is_some() {
                                html! {
                                    <button
                                        type="button"
                                        class="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50"
                                        onclick={on_cancel_click}
                                        disabled={*loading}
                                    >
                                        { "Cancel" }
                                    </button>
                                }
                            } else {
                                html! {}
                            }
                        }

                        <button
                            type="submit"
                            class="px-4 py-2 text-sm font-medium text-white bg-blue-600 border border-transparent rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
                            disabled={*loading || *success || !is_valid}
                        >
                            {
                                if *loading {
                                    html! {
                                        <>
                                            <svg class="animate-spin -ml-1 mr-2 h-4 w-4 text-white inline" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                                                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                                            </svg>
                                            { "Creating..." }
                                        </>
                                    }
                                } else {
                                    html! { "Create Session" }
                                }
                            }
                        </button>
                    </div>
                </form>
            </div>
        </div>
    }
}
