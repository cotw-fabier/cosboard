# UI Components

## Overview

This document defines widget patterns and component usage for COSMIC desktop applications using libcosmic. libcosmic provides a comprehensive set of widgets that follow the COSMIC design language.

## Widget Basics

### Element Type

All widgets return `Element<Message>`:

```rust
fn view(&self) -> cosmic::Element<Message> {
    cosmic::widget::text("Hello").into()
}

// Helper functions also return Element
fn view_button(&self) -> cosmic::Element<Message> {
    cosmic::widget::button::text("Click")
        .on_press(Message::Clicked)
        .into()
}
```

### Widget Composition

Widgets are composable and chainable:

```rust
cosmic::widget::button::text("Save")
    .on_press(Message::Save)
    .width(Length::Fixed(100.0))
    .class(cosmic::theme::Button::Suggested)
```

## Button Variants

### Text Button

```rust
// Standard text button
cosmic::widget::button::text("Click Me")
    .on_press(Message::Clicked)

// Disabled button (no on_press)
cosmic::widget::button::text("Disabled")
```

### Styled Buttons

```rust
// Suggested action (accent color)
cosmic::widget::button::suggested("Save")
    .on_press(Message::Save)

// Destructive action (warning color)
cosmic::widget::button::destructive("Delete")
    .on_press(Message::Delete)

// Standard button
cosmic::widget::button::standard("Cancel")
    .on_press(Message::Cancel)
```

### Icon Button

```rust
// Icon-only button
cosmic::widget::button::icon(
    cosmic::widget::icon::from_name("edit-symbolic")
)
.on_press(Message::Edit)

// Custom icon
cosmic::widget::button::icon(
    cosmic::widget::icon::from_name("window-close-symbolic")
)
.on_press(Message::Close)
.class(cosmic::theme::Button::Destructive)
```

### Link Button

```rust
cosmic::widget::button::link("Learn more")
    .on_press(Message::OpenLink)
```

## Text Input Variants

### Standard Text Input

```rust
cosmic::widget::text_input("Placeholder...", &self.input_value)
    .on_input(Message::InputChanged)
    .on_submit(Message::Submit)
```

### Search Input

```rust
cosmic::widget::search_input("Search...", &self.search_query)
    .on_input(Message::SearchChanged)
    .on_clear(Message::ClearSearch)
    .on_submit(Message::Search)
```

### Secure Input (Password)

```rust
cosmic::widget::secure_input("Password", &self.password, None, true)
    .on_input(Message::PasswordChanged)
    .on_submit(Message::Login)
```

### Editable Input

```rust
// Input that can toggle between edit and display mode
cosmic::widget::editable_input(
    "Enter name",
    &self.name,
    self.is_editing,
    Message::ToggleEdit,
)
.on_input(Message::NameChanged)
```

## Layout Widgets

### Column

```rust
cosmic::widget::column()
    .push(cosmic::widget::text("Item 1"))
    .push(cosmic::widget::text("Item 2"))
    .push(cosmic::widget::text("Item 3"))
    .spacing(cosmic::theme::spacing().space_s)
    .padding(cosmic::theme::spacing().space_m)
```

### Row

```rust
cosmic::widget::row()
    .push(cosmic::widget::text("Left"))
    .push(cosmic::widget::horizontal_space())
    .push(cosmic::widget::text("Right"))
    .spacing(cosmic::theme::spacing().space_s)
    .align_y(cosmic::iced::Alignment::Center)
```

### Container

```rust
cosmic::widget::container(content)
    .width(Length::Fill)
    .height(Length::Shrink)
    .padding(cosmic::theme::spacing().space_m)
    .class(cosmic::theme::Container::Card)
```

### Scrollable

```rust
cosmic::widget::scrollable(
    cosmic::widget::column::with_children(items)
)
.width(Length::Fill)
.height(Length::Fill)
```

## Navigation Widgets

### Navigation Bar

```rust
// In init()
fn init_nav(&mut self) {
    self.nav_model.insert()
        .text("Home")
        .icon(cosmic::widget::icon::from_name("go-home-symbolic"))
        .data(Page::Home);

    self.nav_model.insert()
        .text("Documents")
        .icon(cosmic::widget::icon::from_name("folder-documents-symbolic"))
        .data(Page::Documents);

    self.nav_model.insert()
        .text("Settings")
        .icon(cosmic::widget::icon::from_name("emblem-system-symbolic"))
        .data(Page::Settings);
}

// In Application trait
fn nav_model(&self) -> Option<&nav_bar::Model> {
    Some(&self.nav_model)
}
```

### Header Bar

The header bar is managed by the Application trait:

```rust
impl Application for App {
    fn header_start(&self) -> Vec<cosmic::Element<Message>> {
        // Left side of header
        vec![]
    }

    fn header_center(&self) -> Vec<cosmic::Element<Message>> {
        // Center of header
        vec![]
    }

    fn header_end(&self) -> Vec<cosmic::Element<Message>> {
        // Right side of header
        vec![]
    }
}
```

## Selection Widgets

### Dropdown

```rust
let options = vec!["Option 1", "Option 2", "Option 3"];

cosmic::widget::dropdown(
    &options,
    self.selected_index,
    Message::OptionSelected,
)
```

### Segmented Button

```rust
cosmic::widget::segmented_button::horizontal(&self.segmented_model)
    .on_activate(Message::SegmentActivated)

// Initialize model
let mut model = segmented_button::Model::default();
model.insert().text("Tab 1").data(0);
model.insert().text("Tab 2").data(1);
model.insert().text("Tab 3").data(2);
```

### Checkbox

```rust
cosmic::widget::checkbox("Enable feature", self.feature_enabled)
    .on_toggle(Message::FeatureToggled)
```

### Toggle

```rust
cosmic::widget::toggler(self.is_enabled)
    .on_toggle(Message::Toggled)
    .label("Enable notifications")
```

### Radio Button

```rust
cosmic::widget::radio(
    "Option A",
    OptionValue::A,
    Some(self.selected_option),
    Message::OptionSelected,
)
```

## Display Widgets

### Text

```rust
// Regular text
cosmic::widget::text("Hello, World!")

// Heading
cosmic::widget::text::heading("Section Title")

// With styling
cosmic::widget::text("Warning message")
    .class(cosmic::theme::Text::Accent)
```

### Icon

```rust
// Named icon (from icon theme)
cosmic::widget::icon::from_name("folder-symbolic")
    .size(24)

// SVG icon
cosmic::widget::icon::from_svg_bytes(include_bytes!("../assets/icon.svg"))
```

### Image

```rust
cosmic::widget::image::viewer(image_handle)
    .width(Length::Fill)
```

### Divider

```rust
// Horizontal divider
cosmic::widget::divider::horizontal::default()

// Vertical divider
cosmic::widget::divider::vertical::default()
```

## Settings Widgets

### Settings Item

```rust
cosmic::widget::settings::item(
    "Setting Name",
    cosmic::widget::toggler(self.setting_value)
        .on_toggle(Message::SettingToggled),
)
.description("Description of what this setting does")
```

### Settings Section

```rust
cosmic::widget::settings::section()
    .title("General")
    .add(cosmic::widget::settings::item(
        "Dark Mode",
        cosmic::widget::toggler(self.dark_mode).on_toggle(Message::DarkModeToggled),
    ))
    .add(cosmic::widget::settings::item(
        "Notifications",
        cosmic::widget::toggler(self.notifications).on_toggle(Message::NotificationsToggled),
    ))
```

## Interactive Widgets

### Context Menu

```rust
cosmic::widget::context_menu(
    content,
    Some(vec![
        cosmic::widget::menu::Item::Button("Edit", None, Message::Edit),
        cosmic::widget::menu::Item::Button("Delete", None, Message::Delete),
        cosmic::widget::menu::Item::Divider,
        cosmic::widget::menu::Item::Button("Properties", None, Message::Properties),
    ]),
)
```

### Popover

```rust
cosmic::widget::popover(trigger_widget)
    .popup(popup_content)
    .on_close(Message::ClosePopover)
```

### Tooltip

```rust
cosmic::widget::tooltip(
    button,
    cosmic::widget::text("Helpful tooltip"),
    cosmic::widget::tooltip::Position::Bottom,
)
```

### Slider

```rust
cosmic::widget::slider(0.0..=100.0, self.volume, Message::VolumeChanged)
    .step(1.0)
```

## Specialized Widgets

### Calendar

```rust
cosmic::widget::calendar(
    &self.calendar_model,
    |date| Message::DateSelected(date),
)
```

### Toaster (Notifications)

```rust
// In Application trait
fn view(&self) -> cosmic::Element<Message> {
    cosmic::widget::toaster(
        &self.toasts,
        content,
    )
}

// Add toast
self.toasts.push(cosmic::widget::Toast::new("Operation completed"));
```

### Spin Button

```rust
cosmic::widget::spin_button(
    &self.value.to_string(),
    Message::Increment,
    Message::Decrement,
)
```

## Widget Patterns

### Card Pattern

```rust
fn view_card(&self, item: &Item) -> cosmic::Element<Message> {
    let content = cosmic::widget::column()
        .push(cosmic::widget::text::heading(&item.title))
        .push(cosmic::widget::text(&item.description))
        .spacing(cosmic::theme::spacing().space_xs);

    cosmic::widget::container(content)
        .padding(cosmic::theme::spacing().space_m)
        .class(cosmic::theme::Container::Card)
        .into()
}
```

### List Item Pattern

```rust
fn view_list_item(&self, index: usize, item: &Item) -> cosmic::Element<Message> {
    cosmic::widget::row()
        .push(cosmic::widget::icon::from_name(&item.icon))
        .push(
            cosmic::widget::column()
                .push(cosmic::widget::text(&item.name))
                .push(cosmic::widget::text(&item.subtitle).size(12))
        )
        .push(cosmic::widget::horizontal_space())
        .push(
            cosmic::widget::button::icon(
                cosmic::widget::icon::from_name("go-next-symbolic")
            )
            .on_press(Message::SelectItem(index))
        )
        .spacing(cosmic::theme::spacing().space_s)
        .align_y(cosmic::iced::Alignment::Center)
        .into()
}
```

### Form Pattern

```rust
fn view_form(&self) -> cosmic::Element<Message> {
    cosmic::widget::column()
        .push(
            cosmic::widget::text_input("Name", &self.name)
                .on_input(Message::NameChanged)
        )
        .push(
            cosmic::widget::text_input("Email", &self.email)
                .on_input(Message::EmailChanged)
        )
        .push(
            cosmic::widget::row()
                .push(cosmic::widget::horizontal_space())
                .push(
                    cosmic::widget::button::standard("Cancel")
                        .on_press(Message::Cancel)
                )
                .push(
                    cosmic::widget::button::suggested("Save")
                        .on_press(Message::Save)
                )
                .spacing(cosmic::theme::spacing().space_s)
        )
        .spacing(cosmic::theme::spacing().space_m)
        .into()
}
```

## Best Practices

### Extract View Functions

```rust
// GOOD: Split into focused functions
fn view(&self) -> cosmic::Element<Message> {
    cosmic::widget::column()
        .push(self.view_header())
        .push(self.view_content())
        .push(self.view_footer())
        .into()
}

// BAD: Everything in one function
fn view(&self) -> cosmic::Element<Message> {
    cosmic::widget::column()
        .push(/* 50 lines of header */)
        .push(/* 100 lines of content */)
        .push(/* 30 lines of footer */)
        .into()
}
```

### Use Spacing Constants

```rust
// GOOD: Use theme spacing
.spacing(cosmic::theme::spacing().space_m)
.padding(cosmic::theme::spacing().space_s)

// BAD: Magic numbers
.spacing(16)
.padding(8)
```

## Best Practices Checklist

- [ ] Use appropriate button variants (suggested, destructive, standard)
- [ ] Use theme spacing constants
- [ ] Extract view code into focused helper functions
- [ ] Use settings widgets for preference screens
- [ ] Provide tooltips for icon-only buttons
- [ ] Use semantic container classes

## References

- [libcosmic Widget Documentation](https://pop-os.github.io/libcosmic/cosmic/widget/index.html)
- [COSMIC App Examples](https://github.com/pop-os/libcosmic/tree/master/examples)
