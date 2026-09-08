use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::Duration;

use eframe::egui::{self, Align, Color32, Layout, RichText};

use crate::drawing::{ButtonIcon, icon_button, text_button};
use crate::update::{self, UpdateOutcome};

const CONTROL_WIDTH: f32 = 96.0;
const CONTROL_HEIGHT: f32 = 38.0;
const MODAL_WIDTH: f32 = 360.0;

struct PendingUpdate {
    receiver: Receiver<UpdateEvent>,
}

#[derive(Clone)]
enum UpdateModal {
    Progress(update::UpdateProgress),
    Installed(String),
    Message(String),
}

enum UpdateEvent {
    Progress(update::UpdateProgress),
    Finished(Result<UpdateOutcome, String>),
}

pub struct GuiUpdate {
    pending: Option<PendingUpdate>,
    modal: Option<UpdateModal>,
}

pub fn render_bottom_right(ui: &mut egui::Ui, updates: &mut GuiUpdate, status: &mut String) {
    updates.render(ui, status);
}

impl GuiUpdate {
    pub fn new() -> Self {
        Self {
            pending: None,
            modal: None,
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui, status: &mut String) {
        self.poll(status);
        let busy = self.pending.is_some();
        let version_label = format!("v{}", update::current_version());

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 14.0;

            let update_response = ui
                .add_enabled_ui(!busy, |ui| {
                    icon_button(ui, ButtonIcon::Update, "Update", CONTROL_WIDTH)
                })
                .inner
                .on_hover_text("Check for updates and install if available");
            if update_response.clicked() {
                self.start_update(status);
            }

            ui.label(RichText::new(version_label).color(ui.visuals().weak_text_color()));
        });

        self.render_modal(ui.ctx());

        if self.pending.is_some() {
            ui.ctx().request_repaint_after(Duration::from_millis(250));
        }
    }

    fn poll(&mut self, status: &mut String) {
        loop {
            let Some(result) = self
                .pending
                .as_ref()
                .map(|pending| pending.receiver.try_recv())
            else {
                return;
            };

            match result {
                Ok(UpdateEvent::Progress(progress)) => {
                    *status = progress.message.to_owned();
                    self.modal = Some(UpdateModal::Progress(progress));
                }
                Ok(UpdateEvent::Finished(Ok(outcome))) => {
                    self.pending = None;
                    match outcome {
                        UpdateOutcome::Current { current_version } => {
                            let message = format!(
                                "v_color_picker is already up to date at v{current_version}."
                            );
                            *status = message.clone();
                            self.modal = Some(UpdateModal::Message(message));
                        }
                        UpdateOutcome::Installed(update) => {
                            let message = format!(
                                "Updated successfully to v{}. Restart the app to apply the changes.",
                                update.latest_version
                            );
                            *status = message.clone();
                            self.modal = Some(UpdateModal::Installed(message));
                        }
                    }
                    return;
                }
                Ok(UpdateEvent::Finished(Err(error))) => {
                    self.pending = None;
                    let message = format!("Update failed: {}", gui_safe_message(&error));
                    *status = message.clone();
                    self.modal = Some(UpdateModal::Message(message));
                    return;
                }
                Err(TryRecvError::Empty) => return,
                Err(TryRecvError::Disconnected) => {
                    self.pending = None;
                    let message = "Update stopped before finishing".to_owned();
                    *status = message.clone();
                    self.modal = Some(UpdateModal::Message(message));
                    return;
                }
            }
        }
    }

    fn start_update(&mut self, status: &mut String) {
        if self.pending.is_some() {
            return;
        }

        let initial_progress = update::UpdateProgress::checking();
        *status = initial_progress.message.to_owned();
        self.modal = Some(UpdateModal::Progress(initial_progress));
        let (sender, receiver) = mpsc::channel();

        thread::spawn(move || {
            let progress_sender = sender.clone();
            let result = update::install_latest_release_with_progress(move |progress| {
                let _ = progress_sender.send(UpdateEvent::Progress(progress));
            })
            .map_err(|error| format!("{error:#}"));
            let _ = sender.send(UpdateEvent::Finished(result));
        });

        self.pending = Some(PendingUpdate { receiver });
    }

    fn render_modal(&mut self, ctx: &egui::Context) {
        let Some(modal) = self.modal.clone() else {
            return;
        };
        let mut close_modal = false;
        let mut restart_error = None;

        egui::Modal::new(egui::Id::new("v_color_picker_update_modal"))
            .backdrop_color(Color32::from_black_alpha(150))
            .show(ctx, |ui| {
                ui.set_width(MODAL_WIDTH);
                ui.heading("Update");
                ui.add_space(10.0);

                match modal {
                    UpdateModal::Progress(progress) => {
                        ui.label(RichText::new(progress.message).strong());
                        ui.add_space(10.0);
                        render_progress_bar(ui, &progress);
                    }
                    UpdateModal::Message(message) => {
                        ui.label(message);
                        ui.add_space(14.0);
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if text_button(ui, "Ok", egui::vec2(CONTROL_WIDTH, CONTROL_HEIGHT))
                                .clicked()
                            {
                                close_modal = true;
                            }
                        });
                    }
                    UpdateModal::Installed(message) => {
                        ui.label(message);
                        ui.add_space(14.0);
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if text_button(ui, "Restart App", egui::vec2(128.0, CONTROL_HEIGHT))
                                .clicked()
                            {
                                if let Err(error) = restart_app(ctx) {
                                    restart_error = Some(format!(
                                        "App restart failed: {}",
                                        gui_safe_message(&format!("{error:#}"))
                                    ));
                                }
                            }
                            if text_button(ui, "Later", egui::vec2(CONTROL_WIDTH, CONTROL_HEIGHT))
                                .clicked()
                            {
                                close_modal = true;
                            }
                        });
                    }
                }
            });

        if let Some(message) = restart_error {
            self.modal = Some(UpdateModal::Message(message));
        } else if close_modal {
            self.modal = None;
        }
    }
}

impl Default for GuiUpdate {
    fn default() -> Self {
        Self::new()
    }
}

fn render_progress_bar(ui: &mut egui::Ui, progress: &update::UpdateProgress) {
    let mut bar = egui::ProgressBar::new(progress.fraction.unwrap_or(0.5))
        .desired_width(ui.available_width())
        .animate(true);

    if progress.fraction.is_some() {
        bar = bar.show_percentage();
    } else {
        bar = bar.text("Working in background");
    }

    ui.add(bar);
}

fn restart_app(ctx: &egui::Context) -> anyhow::Result<()> {
    update::relaunch_current_app()?;
    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    Ok(())
}

fn gui_safe_message(message: &str) -> String {
    message
        .replace("github", "release source")
        .replace("GITHUB", "release source")
}
