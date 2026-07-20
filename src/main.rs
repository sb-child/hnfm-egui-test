use hnfm_egui_test::app::AppLayout;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "hnfm egui test",
        options,
        Box::new(|cc| Ok(Box::new(AppLayout::new(cc)))),
    )?;
    println!("program ends");
    Ok(())
}
