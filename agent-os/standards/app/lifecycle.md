# Application Lifecycle

## Overview

This document defines the application lifecycle patterns for COSMIC desktop applications using libcosmic. COSMIC apps follow the Elm architecture (Model-View-Update) with the `cosmic::Application` trait.

## Application Trait

Every COSMIC application implements the `Application` trait:

```rust
use cosmic::app::{Core, Task};
use cosmic::iced::Size;
use cosmic::Application;

pub struct App {
    core: Core,
    // Application-specific state
}

impl Application for App {
    /// Async executor type (usually Default)
    type Executor = cosmic::executor::Default;

    /// Initialization flags passed to init()
    type Flags = ();

    /// Message type for state updates
    type Message = Message;

    /// Unique application identifier (reverse domain notation)
    const APP_ID: &'static str = "org.cosmic.MyApp";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let app = Self {
            core,
            // Initialize state
        };
        (app, Task::none())
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            // Handle messages
            _ => Task::none(),
        }
    }

    fn view(&self) -> cosmic::Element<Self::Message> {
        // Render UI
        cosmic::widget::text("Hello, COSMIC!").into()
    }
}
```

## Entry Point

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure application settings
    let settings = cosmic::app::Settings::default()
        .size(Size::new(800.0, 600.0));

    // Run the application
    cosmic::app::run::<App>(settings, ())?;

    Ok(())
}
```

## Core State

The `Core` struct manages COSMIC-specific state:

```rust
pub struct App {
    /// Required: COSMIC runtime core
    core: Core,

    /// Optional: Navigation sidebar model
    nav_model: nav_bar::Model,

    /// Application-specific state
    data: Vec<Item>,
    current_page: Page,
    is_loading: bool,
}
```

### Core Capabilities

```rust
impl App {
    fn example_core_usage(&self) {
        // Access current theme
        let theme = self.core.system_theme();

        // Check window state
        let is_maximized = self.core.window.is_maximized;
        let scale_factor = self.core.scale_factor();

        // Set window title
        self.core.set_window_title("My App - Document.txt");

        // Set header title (different from window title)
        self.core.set_header_title("Document.txt");
    }
}
```

## Message Enum

Define all state-changing events as message variants:

```rust
#[derive(Debug, Clone)]
pub enum Message {
    // User actions
    OpenFile,
    SaveDocument,
    DeleteItem(usize),

    // Input changes
    InputChanged(String),
    SearchQueryChanged(String),

    // Navigation
    NavSelect(nav_bar::Id),
    PageChanged(Page),

    // Async results
    DataLoaded(Result<Vec<Item>, String>),
    FileSaved(Result<(), String>),

    // System events
    ConfigUpdated(AppConfig),
    ThemeChanged(cosmic::theme::ThemeType),

    // Window operations
    Surface(cosmic::surface::Action),

    // Subscriptions
    Tick(std::time::Instant),
}
```

## Update Function

Handle messages and update state:

```rust
fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
    match message {
        // Simple state update
        Message::InputChanged(value) => {
            self.input_value = value;
            Task::none()
        }

        // Navigation
        Message::NavSelect(id) => {
            self.nav_model.activate(id);
            self.update_page_from_nav();
            Task::none()
        }

        // Async operation
        Message::OpenFile => {
            self.is_loading = true;
            Task::perform(
                async { load_file().await },
                |result| Message::DataLoaded(result.map_err(|e| e.to_string())),
            )
        }

        // Handle async result
        Message::DataLoaded(result) => {
            self.is_loading = false;
            match result {
                Ok(data) => self.data = data,
                Err(e) => self.error = Some(e),
            }
            Task::none()
        }

        // Window surface action
        Message::Surface(action) => {
            cosmic::task::message(cosmic::Action::Cosmic(
                cosmic::app::Action::Surface(action),
            ))
        }

        _ => Task::none(),
    }
}
```

## View Function

Render the UI based on current state:

```rust
fn view(&self) -> cosmic::Element<Self::Message> {
    let content = if self.is_loading {
        self.view_loading()
    } else if let Some(error) = &self.error {
        self.view_error(error)
    } else {
        self.view_content()
    };

    content
}

fn view_loading(&self) -> cosmic::Element<Message> {
    cosmic::widget::container(
        cosmic::widget::text("Loading...")
    )
    .center(cosmic::iced::Length::Fill)
    .into()
}

fn view_content(&self) -> cosmic::Element<Message> {
    let items: Vec<_> = self.data
        .iter()
        .enumerate()
        .map(|(i, item)| self.view_item(i, item))
        .collect();

    cosmic::widget::scrollable(
        cosmic::widget::column::with_children(items)
            .spacing(cosmic::theme::spacing().space_s)
    )
    .into()
}
```

## Navigation

### Navigation Model

```rust
impl App {
    fn init_nav_model() -> nav_bar::Model {
        let mut nav_model = nav_bar::Model::default();

        nav_model.insert()
            .text("Home")
            .icon(cosmic::widget::icon::from_name("go-home-symbolic"))
            .data(Page::Home);

        nav_model.insert()
            .text("Settings")
            .icon(cosmic::widget::icon::from_name("emblem-system-symbolic"))
            .data(Page::Settings);

        nav_model
    }
}
```

### Navigation Trait Methods

```rust
impl Application for App {
    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav_model)
    }

    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<Self::Message> {
        self.nav_model.activate(id);

        if let Some(page) = self.nav_model.data::<Page>(id) {
            self.current_page = page.clone();
        }

        Task::none()
    }
}
```

## Task-Based Async

Use `Task` for async operations:

```rust
// Simple async operation
Task::perform(
    async { fetch_data().await },
    |result| Message::DataLoaded(result),
)

// With error mapping
Task::perform(
    async {
        load_file(path).await
            .map_err(|e| e.to_string())
    },
    Message::FileLoaded,
)

// Batch multiple tasks
Task::batch([
    Task::perform(async { load_a().await }, Message::ALoaded),
    Task::perform(async { load_b().await }, Message::BLoaded),
])

// No-op task
Task::none()
```

## Subscriptions

Subscribe to external events:

```rust
impl Application for App {
    fn subscription(&self) -> cosmic::iced::Subscription<Self::Message> {
        let mut subscriptions = vec![];

        // Watch for config changes
        subscriptions.push(
            self.core()
                .watch_config::<AppConfig>(Self::APP_ID)
                .map(Message::ConfigUpdated)
        );

        // Timer subscription
        if self.timer_active {
            subscriptions.push(
                cosmic::iced::time::every(Duration::from_secs(1))
                    .map(Message::Tick)
            );
        }

        cosmic::iced::Subscription::batch(subscriptions)
    }
}
```

## Header Customization

```rust
impl Application for App {
    fn header_start(&self) -> Vec<cosmic::Element<Self::Message>> {
        vec![
            cosmic::widget::button::text("Menu")
                .on_press(Message::OpenMenu)
                .into()
        ]
    }

    fn header_end(&self) -> Vec<cosmic::Element<Self::Message>> {
        vec![
            cosmic::widget::button::icon(
                cosmic::widget::icon::from_name("edit-find-symbolic")
            )
            .on_press(Message::ToggleSearch)
            .into()
        ]
    }

    fn header_center(&self) -> Vec<cosmic::Element<Self::Message>> {
        if self.show_search {
            vec![
                cosmic::widget::search_input("Search...", &self.search_query)
                    .on_input(Message::SearchChanged)
                    .into()
            ]
        } else {
            vec![]
        }
    }
}
```

## Context Drawer

Side panel for additional options:

```rust
impl Application for App {
    fn context_drawer(&self) -> Option<cosmic::app::context_drawer::ContextDrawer<Self::Message>> {
        if !self.show_context_drawer {
            return None;
        }

        Some(cosmic::app::context_drawer::context_drawer(
            self.view_context_content(),
            Message::CloseContextDrawer,
        ))
    }
}
```

## Application Settings

Configure the application window:

```rust
let settings = cosmic::app::Settings::default()
    .size(Size::new(1024.0, 768.0))      // Initial size
    .size_limits(Limits::NONE            // Size constraints
        .min_width(400.0)
        .min_height(300.0))
    .default_text_size(14.0)             // Default font size
    .theme(cosmic::theme::Theme::dark()) // Initial theme
    .debug(false);                       // Debug overlay

cosmic::app::run::<App>(settings, ())?;
```

## Best Practices Checklist

- [ ] Implement all required `Application` trait methods
- [ ] Use `Core` for window and theme management
- [ ] Define comprehensive `Message` enum
- [ ] Handle async operations with `Task::perform`
- [ ] Use subscriptions for external events
- [ ] Separate view into focused helper functions
- [ ] Initialize navigation model if using sidebar
- [ ] Set appropriate application settings

## References

- [libcosmic Application Trait](https://pop-os.github.io/libcosmic/cosmic/app/trait.Application.html)
- [COSMIC App Template](https://github.com/pop-os/cosmic-app-template)
- [Iced Architecture](https://docs.iced.rs/iced/)
