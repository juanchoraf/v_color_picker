use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::Duration;

use eframe::egui::{self, Align, Color32, Layout, RichText};

use crate::drawing::text_button;
use crate::update::{self, UpdateOutcome};

const CONTROL_WIDTH: f32 = 96.0;
const CONTROL_HEIGHT: f32 = 38.0;
const MODAL_WIDTH: f32 = 360.0;

struct PendingUpdate {
    receiver: Receiver<Result<UpdateOutcome, String>>,
}

#[derive(Clone)]
enum UpdateModal {
    Busy,
    Message(String),
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
        let control_size = egui::vec2(CONTROL_WIDTH, CONTROL_HEIGHT);
        let version_label = format!("v{}", update::current_version());

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 14.0;

            let update_response = ui
                .add_enabled_ui(!busy, |ui| text_button(ui, "Update", control_size))
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
        let Some(result) = self
            .pending
            .as_ref()
            .map(|pending| pending.receiver.try_recv())
        else {
            return;
        };

        match result {
            Ok(Ok(outcome)) => {
                self.pending = None;
                let message = match outcome {
                    UpdateOutcome::Current { current_version } => {
                        format!("v_color_picker is already up to date at v{current_version}.")
                    }
                    UpdateOutcome::Installed(update) => {
                        format!(
                            "Updated successfully to v{}. Please close and open again the app to apply the changes.",
                            update.latest_version
                        )
                    }
                };
                *status = message.clone();
                self.modal = Some(UpdateModal::Message(message));
            }
            Ok(Err(error)) => {
                self.pending = None;
                let message = format!("Update failed: {}", gui_safe_message(&error));
                *status = message.clone();
                self.modal = Some(UpdateModal::Message(message));
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                self.pending = None;
                let message = "Update stopped before finishing".to_owned();
                *status = message.clone();
                self.modal = Some(UpdateModal::Message(message));
            }
        }
    }

    fn start_update(&mut self, status: &mut String) {
        if self.pending.is_some() {
            return;
        }

        *status = "Checking for updates".to_owned();
        self.modal = Some(UpdateModal::Busy);
        let (sender, receiver) = mpsc::channel();

        thread::spawn(move || {
            let result = update::install_latest_release().map_err(|error| format!("{error:#}"));
            let _ = sender.send(result);
        });

        self.pending = Some(PendingUpdate { receiver });
    }

    fn render_modal(&mut self, ctx: &egui::Context) {
        let Some(modal) = self.modal.clone() else {
            return;
        };
        let mut close_modal = false;

        egui::Modal::new(egui::Id::new("v_color_picker_update_modal"))
            .backdrop_color(Color32::from_black_alpha(150))
            .show(ctx, |ui| {
                ui.set_width(MODAL_WIDTH);
                ui.heading("Update");
                ui.add_space(10.0);

                match modal {
                    UpdateModal::Busy => {
                        ui.horizontal(|ui| {
                            ui.add(egui::Spinner::new());
                            ui.label(RichText::new("Checking for updates").strong());
                        });
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
                }
            });

        if close_modal {
            self.modal = None;
        }
    }
}

impl Default for GuiUpdate {
    fn default() -> Self {
        Self::new()
    }
}

fn gui_safe_message(message: &str) -> String {
    message
        .replace("github", "release source")
        .replace("GITHUB", "release source")
}
