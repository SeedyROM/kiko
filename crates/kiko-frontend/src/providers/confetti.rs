use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub struct ConfettiContext {
    pub trigger: Callback<()>,
}

#[derive(Properties, PartialEq)]
pub struct ConfettiProviderProps {
    pub children: Children,
}

#[function_component(ConfettiProvider)]
pub fn confetti_provider(props: &ConfettiProviderProps) -> Html {
    let trigger_callback = use_state(|| None::<Callback<()>>);

    let set_trigger = {
        let trigger_callback = trigger_callback.clone();
        Callback::from(move |callback: Callback<()>| {
            trigger_callback.set(Some(callback));
        })
    };

    let trigger = {
        let trigger_callback = trigger_callback.clone();
        Callback::from(move |_| {
            if let Some(callback) = &*trigger_callback {
                callback.emit(());
            }
        })
    };

    let context = ConfettiContext { trigger };

    html! {
        <ContextProvider<ConfettiContext> context={context}>
            <crate::components::Confetti trigger={set_trigger} />
            {for props.children.iter()}
        </ContextProvider<ConfettiContext>>
    }
}

#[hook]
pub fn use_confetti() -> ConfettiContext {
    use_context::<ConfettiContext>().expect("use_confetti must be used within ConfettiProvider")
}
