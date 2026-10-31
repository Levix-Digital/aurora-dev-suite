//! The shell: navigation, theme, shared state and the preview banner (research R11).

use aurora_core::domain::{EngineReport, EnginesOverview, Settings, Theme};
use leptos::ev::KeyboardEvent;
use leptos::html;
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::sync::Arc;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;

use crate::backend::Backend;
use crate::views;

/// The places of the window; navigation is this one signal, no router.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Portfolio,
    Project,
    Engines,
    Inbox,
    Settings,
}

impl View {
    pub const ALL: [View; 5] = [
        View::Portfolio,
        View::Project,
        View::Engines,
        View::Inbox,
        View::Settings,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            View::Portfolio => "Portfolio",
            View::Project => "Project",
            View::Engines => "Engines",
            View::Inbox => "Inbox",
            View::Settings => "Settings",
        }
    }

    /// The digit of its Alt shortcut.
    pub const fn key(self) -> char {
        match self {
            View::Portfolio => '1',
            View::Project => '2',
            View::Engines => '3',
            View::Inbox => '4',
            View::Settings => '5',
        }
    }
}

/// The engines as last reported, and whether a refresh is running.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct EnginesState {
    pub overview: Option<EnginesOverview>,
    pub refreshing: bool,
    pub error: Option<String>,
}

impl EnginesState {
    /// One engine's report arrived before the others: show it at once.
    fn apply_report(&mut self, report: EngineReport) {
        if let Some(overview) = &mut self.overview {
            match overview.reports.iter_mut().find(|r| r.kind == report.kind) {
                Some(slot) => *slot = report,
                None => overview.reports.push(report),
            }
        }
    }
}

/// State shared by the views.
#[derive(Clone, Copy)]
pub struct AppContext {
    pub view: RwSignal<View>,
    pub settings: RwSignal<Settings>,
    pub engines: RwSignal<EnginesState>,
    pub backend: StoredValue<Backend>,
    /// The project the Project view shows, kept while Aurora runs (spec 004 FR-009).
    pub project: RwSignal<Option<String>>,
}

impl AppContext {
    /// Opens the Project view on the project `id`.
    pub fn open_project(self, id: String) {
        self.project.set(Some(id));
        self.view.set(View::Project);
    }
}

#[component]
pub fn App() -> impl IntoView {
    let backend = Backend::detect();
    let preview = backend.is_preview();
    let ctx = AppContext {
        view: RwSignal::new(View::Portfolio),
        settings: RwSignal::new(Settings::default()),
        engines: RwSignal::new(EnginesState::default()),
        backend: StoredValue::new(backend),
        project: RwSignal::new(None),
    };
    provide_context(ctx);

    start(ctx);
    follow_theme(ctx.settings);

    let main_ref: NodeRef<html::Main> = NodeRef::new();
    let _shortcuts = window_event_listener(leptos::ev::keydown, move |ev: KeyboardEvent| {
        if !ev.alt_key() || ev.ctrl_key() || ev.meta_key() {
            return;
        }
        let key = ev.key();
        if let Some(view) = View::ALL.into_iter().find(|v| key == v.key().to_string()) {
            ev.prevent_default();
            ctx.view.set(view);
            if let Some(main) = main_ref.get() {
                let _ = main.focus();
            }
        }
    });

    view! {
        <div class="shell">
            <aside class="sidebar">
                <div class="brand">"Aurora" <span>" Dev Suite"</span></div>
                <Navigation view=ctx.view />
                <div class="footer">
                    {preview.then(|| view! { <p>"Preview with sample data"</p> })}
                    <p>{concat!("v", env!("CARGO_PKG_VERSION"))}</p>
                </div>
            </aside>
            <main id="main" tabindex="-1" node_ref=main_ref>
                {preview
                    .then(|| {
                        view! {
                            <div class="banner preview-banner" role="note">
                                "Preview: this window shows sample data, not your projects or engines."
                            </div>
                        }
                    })}
                {move || match ctx.view.get() {
                    View::Portfolio => view! { <views::portfolio::Portfolio /> }.into_any(),
                    View::Project => view! { <views::project::ProjectView /> }.into_any(),
                    View::Engines => view! { <views::engines::Engines /> }.into_any(),
                    View::Inbox => view! { <views::inbox::Inbox /> }.into_any(),
                    View::Settings => view! { <views::settings::SettingsView /> }.into_any(),
                }}
            </main>
        </div>
    }
}

/// Buttons in a list; the arrow keys move between them, Enter or Space opens.
#[component]
fn Navigation(view: RwSignal<View>) -> impl IntoView {
    let buttons: [NodeRef<html::Button>; 5] = std::array::from_fn(|_| NodeRef::new());
    let on_keydown = move |ev: KeyboardEvent| {
        let step: isize = match ev.key().as_str() {
            "ArrowDown" => 1,
            "ArrowUp" => -1,
            _ => return,
        };
        ev.prevent_default();
        let focused = document().active_element();
        let current = buttons
            .iter()
            .position(|button| {
                button
                    .get()
                    .zip(focused.clone())
                    .is_some_and(|(b, f)| b.unchecked_ref::<web_sys::Element>() == &f)
            })
            .unwrap_or(0);
        let next = (current as isize + step).rem_euclid(buttons.len() as isize) as usize;
        if let Some(button) = buttons[next].get() {
            let _ = button.focus();
        }
    };
    view! {
        <nav aria-label="Main">
            <ul class="nav" on:keydown=on_keydown>
                {View::ALL
                    .into_iter()
                    .zip(buttons)
                    .map(|(target, node_ref)| {
                        view! {
                            <li>
                                <button
                                    node_ref=node_ref
                                    aria-current=move || (view.get() == target).then_some("page")
                                    aria-keyshortcuts=format!("Alt+{}", target.key())
                                    on:click=move |_| view.set(target)
                                >
                                    <span>{target.label()}</span>
                                    <span class="key" aria-hidden="true">
                                        {format!("Alt+{}", target.key())}
                                    </span>
                                </button>
                            </li>
                        }
                    })
                    .collect_view()}
            </ul>
        </nav>
    }
}

/// Loads the settings and the engines, and follows the engine events.
fn start(ctx: AppContext) {
    let backend = ctx.backend.get_value();
    let engines = ctx.engines;
    backend.subscribe_engines(
        Arc::new(move |report| engines.update(|state| state.apply_report(report))),
        Arc::new(move |overview| {
            engines.update(|state| {
                state.overview = Some(overview);
                state.refreshing = false;
                state.error = None;
            });
        }),
    );
    spawn_local(async move {
        match backend.get_settings().await {
            Ok(settings) => ctx.settings.set(settings),
            Err(error) => leptos::logging::warn!("settings: {}", error.message),
        }
        engines.update(|state| state.refreshing = true);
        let result = backend.engines_overview().await;
        engines.update(|state| {
            state.refreshing = false;
            match result {
                Ok(overview) => state.overview = Some(overview),
                Err(error) => state.error = Some(error.message),
            }
        });
    });
}

/// Applies the theme to `<html data-theme>`, following the system when the setting says so.
fn follow_theme(settings: RwSignal<Settings>) {
    let query = window()
        .match_media("(prefers-color-scheme: dark)")
        .ok()
        .flatten();
    let system_dark = RwSignal::new(query.as_ref().is_none_or(web_sys::MediaQueryList::matches));
    if let Some(query) = query {
        let on_change = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
            if let Ok(Some(list)) = window().match_media("(prefers-color-scheme: dark)") {
                system_dark.set(list.matches());
            }
        });
        let _ =
            query.add_event_listener_with_callback("change", on_change.as_ref().unchecked_ref());
        on_change.forget();
    }
    Effect::new(move |_| {
        let theme = settings.get().theme.resolve(system_dark.get());
        let name = match theme {
            Theme::Dark => "dark",
            Theme::Light => "light",
        };
        if let Some(root) = document().document_element() {
            let _ = root.set_attribute("data-theme", name);
        }
    });
}
