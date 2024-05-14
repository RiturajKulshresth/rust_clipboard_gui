extern crate x11_clipboard;

use gtk::prelude::*;
use std::fs::File;
use std::io::Read;
use x11_clipboard::Clipboard;

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);

    window.set_title("JSON Button Display");
    window.set_border_width(10);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(350, 70);

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 5);
    window.add(&vbox);
    println!("Opening file: ./clipboard.json");

    if let Ok(mut json_file) = File::open("./clipboard.json") {
        let mut contents = String::new();
        json_file
            .read_to_string(&mut contents)
            .expect("Failed to read file");

        if let Ok(data) = serde_json::from_str::<Vec<serde_json::Value>>(&contents) {
            for entry in data {
                if let Some(item) = entry.get("item").and_then(|x| x.as_str()) {
                    let button = gtk::Button::with_label(item);
                    vbox.add(&button);

                    // Connect button click signal to a separate function
                    button.connect_clicked(move |_| {
                        on_button_clicked(entry);
                    });
                }
            }
        } else {
            println!("Failed to parse JSON data");
        }
    } else {
        println!("Failed to open file");
    }

    window.show_all();
}

// Separate function to handle button click
fn on_button_clicked(item: &str) {
    let clipboard = Clipboard::new().unwrap();
    let text_to_copy = item.to_string();
    let text_bytes = text_to_copy.as_bytes();

    let atom_clipboard = clipboard.setter.atoms.clipboard;
    let atom_utf8string = clipboard.setter.atoms.utf8_string;

    if let Err(err) = clipboard.store(atom_clipboard, atom_utf8string, text_bytes) {
        eprintln!("Error copying text to clipboard: {}", err);
    } else {
        println!("Text copied to clipboard: {}", text_to_copy);
    }
}

fn main() {
    let application = gtk::Application::new(
        Some("com.github.gtk-rs.examples.json_buttons"),
        Default::default(),
    );

    application.connect_activate(build_ui);

    application.run();
}
