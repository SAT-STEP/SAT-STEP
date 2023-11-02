use egui::{text::LayoutJob, Color32, FontId};

// #[derive(Clone)]
// pub enum CnfVariables {
//     Decimal(DecimalVar),
//     Bit(BitVar),
//     Equality(EqVar),
// }

pub trait CnfVariable {
    //fn new(identifier: i32) -> Self;

    fn human_readable(
        &self,
        text_job: &mut LayoutJob,
        large_font: FontId,
        small_font: FontId,
        text_color: Color32,
    );

    fn to_cnf(&self) -> i32;
}
