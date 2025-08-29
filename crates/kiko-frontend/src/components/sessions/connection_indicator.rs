use yew::prelude::*;

use crate::hooks::ConnectionState;

#[derive(Properties, PartialEq)]
pub struct ConnectionIndicatorProps {
    pub state: ConnectionState,
}

#[function_component(ConnectionIndicator)]
pub fn connection_indicator(props: &ConnectionIndicatorProps) -> Html {
    let state = &props.state;

    html! {
        <div class="flex items-center space-x-2">
            <div class={classes!("w-3", "h-3", "rounded-full", match state {
                ConnectionState::Connected => "bg-green-500",
                ConnectionState::Connecting => "bg-yellow-500",
                ConnectionState::Disconnected => "bg-gray-500",
                ConnectionState::Error(_) => "bg-red-500",
            })}></div>
            <span class={classes!(
                "text-sm",
                match &state {
                    ConnectionState::Connected => "text-green-600",
                    ConnectionState::Connecting => "text-yellow-600",
                    ConnectionState::Disconnected => "text-gray-600",
                    ConnectionState::Error(_) => "text-red-600",
                }
            )}>{
                match &state {
                    ConnectionState::Connected => "Connected",
                    ConnectionState::Connecting => "Connecting...",
                    ConnectionState::Disconnected => "Disconnected",
                    ConnectionState::Error(_) => "Connection Error",
                }
            }</span>
        </div>
    }
}
