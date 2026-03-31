use crate::monitor_canvas;
use crate::sway;
use crate::types::{Mode, OutputInfo};
use iced::widget::{button, column, container, pick_list, radio, row, text};
use iced::{Element, Length, Subscription};

#[derive(Debug, Clone)]
pub enum Message {
    CheckOutputs,
    ChangeResolution(Mode),
    SetMirror(bool),
    Apply,
    CanvasMessage(monitor_canvas::CanvasMessage),
}

pub struct App {
    outputs: Vec<OutputInfo>,
    selected: Option<usize>,
    mirror_mode: bool,
    saved_positions: Vec<(i32, i32)>,
    status: String,
}

impl Default for App {
    fn default() -> Self {
        let (outputs, status) = match sway::get_outputs() {
            Ok(outputs) => (outputs, String::new()),
            Err(e) => (vec![], format!("Error: {}", e)),
        };
        Self {
            selected: if outputs.is_empty() { None } else { Some(0) },
            outputs,
            mirror_mode: false,
            saved_positions: vec![],
            status,
        }
    }
}

impl App {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::CheckOutputs => {
                if let Ok(new_outputs) = sway::get_outputs() {
                    let current_names: Vec<&str> =
                        self.outputs.iter().map(|o| o.name.as_str()).collect();
                    let new_names: Vec<&str> =
                        new_outputs.iter().map(|o| o.name.as_str()).collect();
                    if current_names != new_names {
                        self.outputs = new_outputs;
                        self.selected = if self.outputs.is_empty() {
                            None
                        } else {
                            Some(0)
                        };
                        self.mirror_mode = false;
                        self.saved_positions.clear();
                        self.status = "Outputs changed.".to_string();
                    }
                }
            }
            Message::CanvasMessage(monitor_canvas::CanvasMessage::SelectOutput(idx)) => {
                if idx < self.outputs.len() {
                    self.selected = Some(idx);
                }
            }
            Message::CanvasMessage(monitor_canvas::CanvasMessage::DragEnd { index, x, y }) => {
                if index < self.outputs.len() {
                    self.outputs[index].x = x;
                    self.outputs[index].y = y;
                    self.snap_output(index);
                }
            }
            Message::ChangeResolution(mode) => {
                if let Some(sel) = self.selected {
                    if sel < self.outputs.len() {
                        self.outputs[sel].width = mode.width;
                        self.outputs[sel].height = mode.height;
                        self.outputs[sel].current_mode = mode;
                        self.recalculate_positions();
                    }
                }
            }
            Message::SetMirror(mirror) => {
                if mirror && !self.mirror_mode {
                    self.saved_positions = self.outputs.iter().map(|o| (o.x, o.y)).collect();
                    self.mirror_mode = true;
                } else if !mirror && self.mirror_mode {
                    for (i, output) in self.outputs.iter_mut().enumerate() {
                        if let Some(&(x, y)) = self.saved_positions.get(i) {
                            output.x = x;
                            output.y = y;
                        }
                    }
                    self.mirror_mode = false;
                    self.saved_positions.clear();
                }
            }
            Message::Apply => match sway::apply_config(&self.outputs, self.mirror_mode) {
                Ok(()) => {
                    self.status = "Configuration applied.".to_string();
                    if let Ok(outputs) = sway::get_outputs() {
                        self.outputs = outputs;
                        if let Some(sel) = self.selected {
                            if sel >= self.outputs.len() {
                                self.selected = if self.outputs.is_empty() {
                                    None
                                } else {
                                    Some(0)
                                };
                            }
                        }
                    }
                }
                Err(e) => self.status = format!("Apply failed: {}", e),
            },
        }
    }

    fn recalculate_positions(&mut self) {
        let max_h = self.outputs.iter().map(|o| o.height).max().unwrap_or(0);
        let mut x = 0;
        for output in &mut self.outputs {
            output.x = x;
            output.y = max_h - output.height;
            x += output.width;
        }
    }

    fn snap_output(&mut self, idx: usize) {
        let snap_threshold = 100;
        let out = &self.outputs[idx];
        let (mut best_x, mut best_y) = (out.x, out.y);
        let (mut min_dx, mut min_dy) = (snap_threshold, snap_threshold);
        let (ox, oy, ow, oh) = (out.x, out.y, out.width, out.height);

        for (i, other) in self.outputs.iter().enumerate() {
            if i == idx {
                continue;
            }
            let (ax, ay, aw, ah) = (other.x, other.y, other.width, other.height);

            // Snap left edge to right edge of other
            let d = (ox - (ax + aw)).abs();
            if d < min_dx {
                min_dx = d;
                best_x = ax + aw;
            }
            // Snap right edge to left edge of other
            let d = ((ox + ow) - ax).abs();
            if d < min_dx {
                min_dx = d;
                best_x = ax - ow;
            }

            // Snap bottom edges together
            let d = ((oy + oh) - (ay + ah)).abs();
            if d < min_dy {
                min_dy = d;
                best_y = ay + ah - oh;
            }
            // Snap top edges together
            let d = (oy - ay).abs();
            if d < min_dy {
                min_dy = d;
                best_y = ay;
            }
            // Snap top to bottom of other
            let d = (oy - (ay + ah)).abs();
            if d < min_dy {
                min_dy = d;
                best_y = ay + ah;
            }
            // Snap bottom to top of other
            let d = ((oy + oh) - ay).abs();
            if d < min_dy {
                min_dy = d;
                best_y = ay - oh;
            }
        }

        self.outputs[idx].x = best_x;
        self.outputs[idx].y = best_y;
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::run(|| {
            iced::stream::channel(
                1,
                |mut sender: iced::futures::channel::mpsc::Sender<Message>| async move {
                    let (tx, mut rx) = iced::futures::channel::mpsc::channel::<()>(1);
                    std::thread::spawn(move || {
                        let conn = match swayipc::Connection::new() {
                            Ok(c) => c,
                            Err(_) => return,
                        };
                        let events = match conn.subscribe([swayipc::EventType::Output]) {
                            Ok(e) => e,
                            Err(_) => return,
                        };
                        let mut tx = tx;
                        for _ in events.flatten() {
                            if tx.try_send(()).is_err() {
                                break;
                            }
                        }
                    });
                    use iced::futures::{SinkExt, StreamExt};
                    while rx.next().await.is_some() {
                        let _ = sender.send(Message::CheckOutputs).await;
                    }
                },
            )
        })
    }

    pub fn view(&self) -> Element<'_, Message> {
        let canvas = monitor_canvas::monitor_canvas(&self.outputs, self.selected)
            .map(Message::CanvasMessage);

        let mirror_controls = row![
            radio("Extend", false, Some(self.mirror_mode), Message::SetMirror).spacing(5),
            radio("Mirror", true, Some(self.mirror_mode), Message::SetMirror).spacing(5),
        ]
        .spacing(20)
        .align_y(iced::Alignment::Center);

        let controls: Element<Message> = if let Some(sel) = self.selected {
            let output = &self.outputs[sel];

            let info = text(format!(
                "{} ({} {})",
                output.name, output.make, output.model
            ))
            .size(16);

            let modes: Vec<Mode> = if self.mirror_mode {
                common_modes(&self.outputs)
            } else {
                output.modes.clone()
            };

            let resolution_row = row![
                pick_list(
                    modes,
                    Some(output.current_mode.clone()),
                    Message::ChangeResolution,
                )
                .placeholder("Select resolution..."),
                button("Apply").on_press(Message::Apply),
            ]
            .spacing(10)
            .align_y(iced::Alignment::Center);

            column![info, resolution_row, text(&self.status).size(13)]
                .spacing(10)
                .align_x(iced::Alignment::Center)
                .into()
        } else {
            text("No outputs detected.").size(16).into()
        };

        container(
            column![mirror_controls, canvas, controls]
                .spacing(15)
                .padding(15)
                .align_x(iced::Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

fn common_modes(outputs: &[OutputInfo]) -> Vec<Mode> {
    if outputs.is_empty() {
        return vec![];
    }
    outputs[0]
        .modes
        .iter()
        .filter(|mode| {
            outputs[1..].iter().all(|output| {
                output
                    .modes
                    .iter()
                    .any(|m| m.width == mode.width && m.height == mode.height)
            })
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{test_output, Mode};

    fn make_app(outputs: Vec<OutputInfo>) -> App {
        let selected = if outputs.is_empty() { None } else { Some(0) };
        App {
            outputs,
            selected,
            mirror_mode: false,
            saved_positions: vec![],
            status: String::new(),
        }
    }

    #[test]
    fn recalculate_positions_bottom_aligns() {
        let mut app = make_app(vec![
            test_output("A", 0, 0, 1920, 1080),
            test_output("B", 0, 0, 2560, 1440),
        ]);
        app.recalculate_positions();

        assert_eq!(app.outputs[0].x, 0);
        assert_eq!(app.outputs[0].y, 1440 - 1080);
        assert_eq!(app.outputs[1].x, 1920);
        assert_eq!(app.outputs[1].y, 0);
    }

    #[test]
    fn recalculate_positions_same_height() {
        let mut app = make_app(vec![
            test_output("A", 0, 0, 1920, 1080),
            test_output("B", 0, 0, 1920, 1080),
        ]);
        app.recalculate_positions();

        assert_eq!(app.outputs[0].x, 0);
        assert_eq!(app.outputs[0].y, 0);
        assert_eq!(app.outputs[1].x, 1920);
        assert_eq!(app.outputs[1].y, 0);
    }

    #[test]
    fn snap_output_snaps_to_right_edge() {
        let mut app = make_app(vec![
            test_output("A", 0, 0, 1920, 1080),
            test_output("B", 1950, 0, 2560, 1080),
        ]);
        app.snap_output(1);

        assert_eq!(app.outputs[1].x, 1920);
    }

    #[test]
    fn snap_output_snaps_to_left_edge() {
        let mut app = make_app(vec![
            test_output("A", 0, 0, 1920, 1080),
            test_output("B", -2590, 0, 2560, 1080),
        ]);
        app.snap_output(1);

        assert_eq!(app.outputs[1].x, -2560);
    }

    #[test]
    fn snap_output_snaps_bottom_edges() {
        let mut app = make_app(vec![
            test_output("A", 0, 0, 1920, 1080),
            test_output("B", 1920, -330, 2560, 1440),
        ]);
        // B bottom at -330+1440=1110, A bottom at 1080. Diff = 30, within threshold.
        app.snap_output(1);

        assert_eq!(app.outputs[1].y, 1080 - 1440);
    }

    #[test]
    fn snap_output_no_snap_beyond_threshold() {
        let mut app = make_app(vec![
            test_output("A", 0, 0, 1920, 1080),
            test_output("B", 2200, 0, 2560, 1080),
        ]);
        app.snap_output(1);

        assert_eq!(app.outputs[1].x, 2200);
        assert_eq!(app.outputs[1].y, 0);
    }

    #[test]
    fn common_modes_returns_shared_resolutions() {
        let mode_a = Mode {
            width: 1920,
            height: 1080,
            refresh: 60000,
        };
        let mode_b = Mode {
            width: 2560,
            height: 1440,
            refresh: 60000,
        };

        let mut out1 = test_output("A", 0, 0, 1920, 1080);
        out1.modes = vec![mode_a.clone(), mode_b.clone()];

        let mut out2 = test_output("B", 1920, 0, 2560, 1440);
        out2.modes = vec![mode_a.clone()];

        let common = common_modes(&[out1, out2]);
        assert_eq!(common, vec![mode_a]);
    }

    #[test]
    fn common_modes_empty_outputs() {
        assert_eq!(common_modes(&[]), vec![]);
    }

    #[test]
    fn common_modes_single_output_returns_all() {
        let mut out = test_output("A", 0, 0, 1920, 1080);
        out.modes = vec![
            Mode {
                width: 1920,
                height: 1080,
                refresh: 60000,
            },
            Mode {
                width: 1280,
                height: 720,
                refresh: 60000,
            },
        ];
        let common = common_modes(&[out.clone()]);
        assert_eq!(common, out.modes);
    }

    #[test]
    fn mirror_toggle_saves_and_restores_positions() {
        let mut app = make_app(vec![
            test_output("A", 0, 0, 1920, 1080),
            test_output("B", 1920, 0, 2560, 1080),
        ]);

        app.update(Message::SetMirror(true));
        assert!(app.mirror_mode);
        assert_eq!(app.saved_positions, vec![(0, 0), (1920, 0)]);

        app.update(Message::SetMirror(false));
        assert!(!app.mirror_mode);
        assert_eq!(app.outputs[0].x, 0);
        assert_eq!(app.outputs[1].x, 1920);
    }

    #[test]
    fn change_resolution_recalculates_positions() {
        let mut app = make_app(vec![
            test_output("A", 0, 0, 1920, 1080),
            test_output("B", 1920, 0, 2560, 1440),
        ]);
        app.selected = Some(0);

        let new_mode = Mode {
            width: 2560,
            height: 1440,
            refresh: 60000,
        };
        app.update(Message::ChangeResolution(new_mode));

        assert_eq!(app.outputs[0].x, 0);
        assert_eq!(app.outputs[0].width, 2560);
        assert_eq!(app.outputs[1].x, 2560);
    }
}
