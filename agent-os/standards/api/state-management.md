# State Management

## Overview

This document defines state management patterns for COSMIC desktop applications. COSMIC apps use the Elm architecture with Core state, navigation models, and configuration persistence.

## Application State Structure

### Core State

Every COSMIC app has a `Core` struct managed by the runtime:

```rust
pub struct App {
    /// COSMIC runtime core (required)
    core: Core,

    /// Navigation model (optional, for sidebar apps)
    nav_model: nav_bar::Model,

    /// Application-specific state
    current_page: Page,
    data: Vec<Item>,
    is_loading: bool,
    error: Option<String>,
}
```

### State Categories

Organize state by purpose:

```rust
pub struct App {
    // System state (managed by COSMIC)
    core: Core,

    // Navigation state
    nav_model: nav_bar::Model,
    current_page: Page,

    // UI state (transient)
    search_query: String,
    selected_item: Option<usize>,
    show_dialog: bool,

    // Data state (persistent)
    items: Vec<Item>,
    config: AppConfig,

    // Async state
    is_loading: bool,
    pending_operation: Option<OperationType>,
    error: Option<String>,
}
```

## Core State Access

### Reading Core State

```rust
impl App {
    fn view(&self) -> Element<Message> {
        // Access theme
        let theme = self.core.system_theme();

        // Access window state
        let is_maximized = self.core.window.is_maximized;

        // Access scale factor
        let scale = self.core.scale_factor();

        // ...
    }
}
```

### Modifying Core State

```rust
impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SetTitle(title) => {
                self.core.set_window_title(&title);
                Task::none()
            }
            Message::SetHeaderTitle(title) => {
                self.core.set_header_title(&title);
                Task::none()
            }
            _ => Task::none(),
        }
    }
}
```

## Navigation State

### Navigation Model

```rust
use cosmic::widget::nav_bar;

impl App {
    fn init_nav_model() -> nav_bar::Model {
        let mut model = nav_bar::Model::default();

        model.insert()
            .text("Home")
            .icon(cosmic::widget::icon::from_name("go-home-symbolic"))
            .data(Page::Home);

        model.insert()
            .text("Documents")
            .icon(cosmic::widget::icon::from_name("folder-documents-symbolic"))
            .data(Page::Documents);

        model.insert()
            .text("Settings")
            .icon(cosmic::widget::icon::from_name("emblem-system-symbolic"))
            .data(Page::Settings);

        model
    }
}
```

### Navigation Handling

```rust
impl Application for App {
    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav_model)
    }

    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<Message> {
        // Activate the selected nav item
        self.nav_model.activate(id);

        // Update current page from nav data
        if let Some(page) = self.nav_model.data::<Page>(id) {
            self.current_page = page.clone();
        }

        // Optionally load page-specific data
        match self.current_page {
            Page::Documents => {
                Task::perform(
                    async { load_documents().await },
                    Message::DocumentsLoaded,
                )
            }
            _ => Task::none(),
        }
    }
}
```

## Configuration Persistence

### CosmicConfigEntry

```rust
use cosmic_config::CosmicConfigEntry;

#[derive(Debug, Clone, CosmicConfigEntry, PartialEq)]
#[version = 1]
pub struct AppConfig {
    pub theme_mode: ThemeMode,
    pub sidebar_visible: bool,
    pub auto_save: bool,
    pub recent_files: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme_mode: ThemeMode::System,
            sidebar_visible: true,
            auto_save: true,
            recent_files: Vec::new(),
        }
    }
}
```

### Loading Configuration

```rust
use cosmic_config::{Config, ConfigGet};

impl App {
    fn load_config() -> AppConfig {
        match Config::new(App::APP_ID, 1) {
            Ok(config) => {
                AppConfig::get_entry(&config).unwrap_or_default()
            }
            Err(_) => AppConfig::default(),
        }
    }
}
```

### Saving Configuration

```rust
use cosmic_config::{Config, ConfigSet};

impl App {
    fn save_config(&self) -> Result<(), cosmic_config::Error> {
        let config = Config::new(Self::APP_ID, 1)?;
        self.config.set_entry(&config)?;
        Ok(())
    }
}

// In update
fn update(&mut self, message: Message) -> Task<Message> {
    match message {
        Message::ToggleSidebar => {
            self.config.sidebar_visible = !self.config.sidebar_visible;
            let _ = self.save_config();
            Task::none()
        }
        _ => Task::none(),
    }
}
```

### Watching Configuration Changes

```rust
impl Application for App {
    fn subscription(&self) -> cosmic::iced::Subscription<Message> {
        self.core()
            .watch_config::<AppConfig>(Self::APP_ID)
            .map(Message::ConfigUpdated)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ConfigUpdated(config) => {
                self.config = config;
                Task::none()
            }
            _ => Task::none(),
        }
    }
}
```

## UI State Patterns

### Selection State

```rust
pub struct App {
    items: Vec<Item>,
    selected_index: Option<usize>,
}

impl App {
    fn select_item(&mut self, index: usize) {
        if index < self.items.len() {
            self.selected_index = Some(index);
        }
    }

    fn selected_item(&self) -> Option<&Item> {
        self.selected_index.and_then(|i| self.items.get(i))
    }

    fn clear_selection(&mut self) {
        self.selected_index = None;
    }
}
```

### Form State

```rust
#[derive(Debug, Clone, Default)]
pub struct FormState {
    pub name: String,
    pub email: String,
    pub errors: HashMap<String, String>,
    pub is_submitting: bool,
}

impl FormState {
    pub fn validate(&mut self) -> bool {
        self.errors.clear();

        if self.name.trim().is_empty() {
            self.errors.insert("name".into(), "Name is required".into());
        }

        if !self.email.contains('@') {
            self.errors.insert("email".into(), "Invalid email".into());
        }

        self.errors.is_empty()
    }

    pub fn get_error(&self, field: &str) -> Option<&str> {
        self.errors.get(field).map(|s| s.as_str())
    }
}
```

### Dialog State

```rust
pub struct App {
    dialog: Option<DialogState>,
}

#[derive(Debug, Clone)]
pub enum DialogState {
    Confirm {
        title: String,
        message: String,
        on_confirm: Message,
    },
    EditItem {
        item_index: usize,
        draft: ItemDraft,
    },
}

fn update(&mut self, message: Message) -> Task<Message> {
    match message {
        Message::ShowConfirmDialog(title, message, on_confirm) => {
            self.dialog = Some(DialogState::Confirm {
                title,
                message,
                on_confirm: *on_confirm,
            });
            Task::none()
        }
        Message::CloseDialog => {
            self.dialog = None;
            Task::none()
        }
        Message::ConfirmDialog => {
            if let Some(DialogState::Confirm { on_confirm, .. }) = &self.dialog {
                let msg = on_confirm.clone();
                self.dialog = None;
                return self.update(msg);
            }
            Task::none()
        }
        _ => Task::none(),
    }
}
```

## Async State Patterns

### Loading State

```rust
pub struct App {
    data_state: DataState,
}

#[derive(Debug, Clone)]
pub enum DataState {
    NotLoaded,
    Loading,
    Loaded(Vec<Item>),
    Error(String),
}

fn update(&mut self, message: Message) -> Task<Message> {
    match message {
        Message::LoadData => {
            self.data_state = DataState::Loading;
            Task::perform(
                async { load_data().await },
                |result| match result {
                    Ok(data) => Message::DataLoaded(data),
                    Err(e) => Message::DataError(e.to_string()),
                },
            )
        }
        Message::DataLoaded(data) => {
            self.data_state = DataState::Loaded(data);
            Task::none()
        }
        Message::DataError(error) => {
            self.data_state = DataState::Error(error);
            Task::none()
        }
        _ => Task::none(),
    }
}

fn view(&self) -> Element<Message> {
    match &self.data_state {
        DataState::NotLoaded => self.view_empty(),
        DataState::Loading => self.view_loading(),
        DataState::Loaded(data) => self.view_data(data),
        DataState::Error(msg) => self.view_error(msg),
    }
}
```

### Optimistic Updates

```rust
fn update(&mut self, message: Message) -> Task<Message> {
    match message {
        Message::DeleteItem(index) => {
            // Optimistic: remove immediately
            let item = self.items.remove(index);

            // Async: persist deletion
            Task::perform(
                async move { delete_item_from_storage(&item).await },
                move |result| {
                    if result.is_err() {
                        Message::RestoreItem(index, item.clone())
                    } else {
                        Message::Noop
                    }
                },
            )
        }
        Message::RestoreItem(index, item) => {
            // Restore on failure
            self.items.insert(index, item);
            self.error = Some("Failed to delete item".into());
            Task::none()
        }
        _ => Task::none(),
    }
}
```

## Context Drawer State

```rust
pub struct App {
    context_drawer_open: bool,
    context_drawer_content: Option<ContextContent>,
}

#[derive(Debug, Clone)]
pub enum ContextContent {
    ItemDetails(usize),
    Settings,
    Help,
}

impl Application for App {
    fn context_drawer(&self) -> Option<ContextDrawer<Message>> {
        if !self.context_drawer_open {
            return None;
        }

        let content = match &self.context_drawer_content {
            Some(ContextContent::ItemDetails(index)) => {
                self.view_item_details(*index)
            }
            Some(ContextContent::Settings) => {
                self.view_drawer_settings()
            }
            _ => return None,
        };

        Some(cosmic::app::context_drawer::context_drawer(
            content,
            Message::CloseContextDrawer,
        ))
    }
}
```

## Best Practices

### State Organization

```rust
// GOOD: Organized state
pub struct App {
    // System
    core: Core,

    // Navigation
    nav_model: nav_bar::Model,
    current_page: Page,

    // Data
    items: Vec<Item>,
    config: AppConfig,

    // UI
    search: String,
    selected: Option<usize>,
    dialog: Option<Dialog>,

    // Async
    loading: HashSet<LoadingKey>,
    errors: HashMap<String, String>,
}

// BAD: Flat, disorganized state
pub struct App {
    core: Core,
    items: Vec<Item>,
    is_loading: bool,
    search: String,
    nav_model: nav_bar::Model,
    error: Option<String>,
    selected: Option<usize>,
    // ... mixed concerns
}
```

### Immutable Updates

```rust
fn update(&mut self, message: Message) -> Task<Message> {
    match message {
        // GOOD: Clear state transitions
        Message::AddItem(item) => {
            self.items.push(item);
            Task::none()
        }

        // GOOD: Replace entire collection
        Message::SetItems(items) => {
            self.items = items;
            Task::none()
        }

        // BAD: Hidden mutations
        Message::ProcessItems => {
            for item in &mut self.items {
                item.process();  // Side effects hidden in loop
            }
            Task::none()
        }
    }
}
```

## Best Practices Checklist

- [ ] Use Core for system state
- [ ] Use nav_bar::Model for navigation
- [ ] Use CosmicConfigEntry for persistent config
- [ ] Organize state by category
- [ ] Handle loading/error states explicitly
- [ ] Watch for external config changes
- [ ] Keep UI state transient
- [ ] Use enums for state machines

## References

- [libcosmic Core](https://pop-os.github.io/libcosmic/cosmic/app/struct.Core.html)
- [cosmic-config](https://docs.rs/cosmic-config/)
- [Elm Architecture](https://guide.elm-lang.org/architecture/)
