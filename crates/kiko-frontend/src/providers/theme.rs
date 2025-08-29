use gloo_storage::{LocalStorage, Storage};
use web_sys::window;
use yew::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Theme {
    Light,
    Dark,
}

impl Theme {
    pub fn as_str(&self) -> &'static str {
        match self {
            Theme::Light => "light",
            Theme::Dark => "dark",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "dark" => Theme::Dark,
            _ => Theme::Light,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThemeContext {
    pub theme: Theme,
    pub toggle: Callback<()>,
}

#[derive(Properties, PartialEq)]
pub struct ThemeProviderProps {
    pub children: Children,
}

#[function_component(ThemeProvider)]
pub fn theme_provider(props: &ThemeProviderProps) -> Html {
    let theme = use_state(|| {
        LocalStorage::get::<String>("theme")
            .map(|s| Theme::from_str(&s))
            .unwrap_or_else(|_| {
                if let Some(window) = window() {
                    if window
                        .match_media("(prefers-color-scheme: dark)")
                        .ok()
                        .flatten()
                        .map(|mq| mq.matches())
                        .unwrap_or(false)
                    {
                        Theme::Dark
                    } else {
                        Theme::Light
                    }
                } else {
                    Theme::Light
                }
            })
    });

    let toggle = {
        let theme = theme.clone();
        Callback::from(move |_| {
            let new_theme = match *theme {
                Theme::Light => Theme::Dark,
                Theme::Dark => Theme::Light,
            };

            LocalStorage::set("theme", new_theme.as_str()).ok();

            if let Some(document) = web_sys::window().and_then(|w| w.document()) {
                if let Some(html) = document.document_element() {
                    let class_list = html.class_list();
                    match new_theme {
                        Theme::Dark => {
                            class_list.add_1("dark").ok();
                        }
                        Theme::Light => {
                            class_list.remove_1("dark").ok();
                        }
                    }
                }
            }

            theme.set(new_theme);
        })
    };

    use_effect_with(theme.clone(), |theme| {
        if let Some(document) = web_sys::window().and_then(|w| w.document()) {
            if let Some(html) = document.document_element() {
                let class_list = html.class_list();
                match **theme {
                    Theme::Dark => {
                        class_list.add_1("dark").ok();
                    }
                    Theme::Light => {
                        class_list.remove_1("dark").ok();
                    }
                }
            }
        }
    });

    let context = ThemeContext {
        theme: *theme,
        toggle,
    };

    html! {
        <ContextProvider<ThemeContext> context={context}>
            {props.children.clone()}
        </ContextProvider<ThemeContext>>
    }
}

#[hook]
pub fn use_theme() -> ThemeContext {
    use_context::<ThemeContext>().expect("use_theme must be used within a ThemeProvider")
}
