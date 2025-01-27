use iced::{executor, Color};
use iced::widget::{button, column, text, scrollable};
use iced::{Application, Command, Element, Length, Settings, Theme};
use iced::{alignment, theme};

use std::process::{Command as ShellCommand, Output};
use iced::application::StyleSheet;

fn main() -> iced::Result {
    DnfWidget::run(Settings {
        window: iced::window::Settings {
            size: (300, 300),
            resizable: false,
            transparent: true,
            decorations: true, // Change when done..
            ..Default::default()
        },
        ..Default::default()
    })
}

#[derive(Default)]
struct DnfWidget {
    updates: String,
    status: String,
    is_updating: bool,
}

#[derive(Debug, Clone)]
enum Message {
    CheckUpdates,
    UpdatesChecked(String),
    Upgrade,
    UpgradeDone(String)
}

impl Application for DnfWidget {
    type Message = Message;
    type Executor = executor::Default;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (DnfWidget::default(), Command::perform(check_for_updates(), Message::UpdatesChecked))
    }

    fn title(&self) -> String {
        String::from("DNF Updates")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::CheckUpdates => {
                return Command::perform(check_for_updates(), Message::UpdatesChecked);
            }
            Message::UpdatesChecked(updated_packages) => {
                self.updates = updated_packages;
            }
            Message::Upgrade => {
                self.is_updating = true;
                self.status = "Upgrading packages...".to_string();
                return Command::perform(dnf_upgrade(), Message::UpgradeDone);
            }
            Message::UpgradeDone(result) => {
                self.is_updating = false;
                self.status = result;
            }
        }
        Command::none()
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }


    // Define the view of the app
    fn view(&self) -> Element<Self::Message> {
        let updates_view = scrollable(
            text(if self.updates.is_empty() {
                "Checking for updates..." // Text while updates are loading
            } else {
                &self.updates
            })
                .size(16),
        )
            .height(Length::Fill);

        // Change the label of the button based on whether we are updating
        let button_label = if self.is_updating {
            "Upgrading..." // When upgrading
        } else {
            "Upgrade Now" // Normal state
        };

        // The button always returns `Message::Upgrade`, but is disabled if we're upgrading
        let upgrade_button = if self.is_updating || self.updates.contains("No packages available") {
            button(text(button_label))
                .style(theme::Button::Primary)
                .padding(10)
        } else {
            button(text(button_label))
                .on_press(Message::Upgrade)
                .style(theme::Button::Primary)
                .padding(10)
        };

        let status_text = text(&self.status)
            .size(16)
            .horizontal_alignment(alignment::Horizontal::Center);


        column![
            text("DNF Updates").size(24),
            updates_view,
            upgrade_button,
            status_text,
        ]
            .spacing(20)
            .padding(20)
            .into()
    }
}


async fn check_for_updates() -> String {
    match ShellCommand::new("dnf")
        .arg("check-update")
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                // Parse the output to determine if packages are listed
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.trim().is_empty() {
                    // No packages are listed, no updates are available
                    "No packages available".to_string()
                } else {
                    // Updates are available
                    format!("Packages available:\n{}", stdout)
                }
            } else {
                // Check the stderr to see if it's a repository failure or other error
                format!(
                    "Failed to fetch updates:\n{}",
                    String::from_utf8_lossy(&output.stderr)
                )
            }
        }
        Err(_) => "Failed to fetch updates. Is DNF installed?".to_string(),
    }
}

async fn dnf_upgrade() -> String {
    match ShellCommand::new("sudo")
        .arg("dnf")
        .arg("upgrade")
        .arg("y")
        .output()
    {
        Ok(output) => parse_dnf_output(output),
        Err(_) => "Failed to upgrade packages. Is DNF installed?".to_string(),
    }
}

fn parse_dnf_output(output: Output) -> String {
    if output.status.success() {
        String::from_utf8_lossy(&output.stdout).to_string()
    } else {
        String::from_utf8_lossy(&output.stderr).to_string()
    }
}