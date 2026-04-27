use epaint::text::Fonts;
fn check(f: &Fonts) {
    let _ = f.layout_job(epaint::text::LayoutJob::default());
}
