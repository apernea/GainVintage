use nih_plug_egui::egui::{
    Align2, Color32, FontId, Painter, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui,
    Vec2, Widget, style::WidgetVisuals,
};
use std::ops::RangeInclusive;

/// A vertical peak meter widget for displaying dB level.
pub struct PeakMeter {
    level_range: RangeInclusive<f32>,
    level_db: f32,
    draw_label: bool,
    size: Vec2,
}

impl PeakMeter {
    pub fn new(level_range: RangeInclusive<f32>, level_db: f32) -> Self {
        Self {
            level_range,
            level_db,
            draw_label: true,
            size: Vec2::new(40.0, 200.0),
        }
    }

    /// Whether to show the dB label ticks.
    pub fn show_label(mut self, show: bool) -> Self {
        self.draw_label = show;
        self
    }

    /// Override the default size of the meter.
    pub fn with_size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }
}

impl Widget for PeakMeter {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, response) =
            ui.allocate_exact_size(self.size, Sense::focusable_noninteractive());

        if ui.is_rect_visible(rect) {
            let visuals = ui.style().interact(&response);
            let rect = rect.expand(visuals.expansion);
            self.paint_level_meter(ui.painter(), rect, visuals);
        }

        response
    }
}

impl PeakMeter {
    fn paint_level_meter(&self, painter: &Painter, rect: Rect, visuals: &WidgetVisuals) {
        // Draw the full background
        painter.rect(
            rect,
            visuals.corner_radius,
            Color32::from_black_alpha(40),
            Stroke::new(1.0, visuals.bg_fill),
            StrokeKind::Inside,
        );

        // === Visual mapping (FL-style perceptual) ===
        let min_db = *self.level_range.start();
        let max_db = *self.level_range.end();
        let clamped = self.level_db.clamp(min_db, max_db);

        // Normalize and perceptually scale
        let normalized = (clamped - min_db) / (max_db - min_db);
        let perceptual = normalized.powf(3.0); // compress low dB visually
        let fill_top = rect.bottom() - perceptual * rect.height();

        let level_rect = Rect::from_min_max(Pos2::new(rect.left(), fill_top), rect.right_bottom());

        // Color based on level
        let color = if clamped < -6.0 {
            Color32::GREEN
        } else if clamped < 0.0 {
            Color32::YELLOW
        } else {
            Color32::RED
        };

        painter.rect_filled(level_rect, 2.0, color);

        if self.draw_label {
            let spacing = 6.0;
            let mut tick = max_db;

            while tick >= min_db {
                let t_norm = (tick - min_db) / (max_db - min_db);
                let t_y = rect.bottom() - t_norm.powf(3.0) * rect.height();
                let label = format!("{:>3}", tick as i32);
                painter.text(
                    Pos2::new(rect.left() - 6.0, t_y),
                    Align2::RIGHT_CENTER,
                    label,
                    FontId::monospace(9.0),
                    Color32::GRAY,
                );
                tick -= spacing;
            }
        }
    }
}
