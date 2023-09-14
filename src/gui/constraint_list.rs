use egui::{Response, Ui, ScrollArea};

pub fn constraint_list(ui: &mut Ui) -> Response {
    let constraints: Vec<&[i32]> = vec![&[123, 43, 829, 432], &[-123, 32, 543], &[53]];
    ui.vertical(|ui| {
        ScrollArea::vertical()
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for constraint in constraints {
                    ui.label(format!("{:?}", constraint));
                }
        });
    }).response
}
