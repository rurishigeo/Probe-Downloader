#[cfg(windows)]
extern crate winres;

#[cfg(windows)]
fn main() {
    slint_build::compile("ui/main.slint").unwrap();
    
    let mut res = winres::WindowsResource::new();
    res.set_icon("icon.ico");
    res.compile().unwrap();
}

#[cfg(unix)]
fn main() {
    slint_build::compile("ui/main.slint").unwrap();
}