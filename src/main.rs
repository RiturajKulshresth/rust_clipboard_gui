extern crate x11_clipboard;
extern crate gtk;
extern crate serde_json;
extern crate inotify;

use gtk::prelude::*;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use std::cell::RefCell;
use x11_clipboard::Clipboard;
use inotify::{Inotify, WatchMask, EventMask};

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);
    window.set_title("JSON Button Display");
    window.set_border_width(10);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(350, 70);

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 5);
    window.add(&vbox);

    println!("Opening file: ./clipboard.json");

    let clipboard = Rc::new(RefCell::new(
        Clipboard::new().expect("Failed to initialize clipboard")
    ));

    // Initialize inotify instance
    let mut inotify = Inotify::init().expect("Failed to initialize inotify");
    let watch_descriptor = inotify.add_watch("clipboard.json", WatchMask::MODIFY).expect("Failed to add watch");

    if let Ok(mut json_file) = File::open("clipboard.json") {
        let mut contents = String::new();
        json_file
            .read_to_string(&mut contents)
            .expect("Failed to read file");

        if let Ok(data) = serde_json::from_str::<Vec<serde_json::Value>>(&contents) {
            for entry in data {
                if let Some(item) = entry.get("item").and_then(|x| x.as_str()) {
                    let entry_clone = entry.clone(); // Clone entry before moving into closure
                    let button = gtk::Button::with_label(item);
                    vbox.add(&button);

                    let clipboard_clone = Rc::clone(&clipboard);

                    button.connect_clicked(move |_| {
                        if let Some(item) = entry_clone.get("item").and_then(|x| x.as_str()) {
                            let text_to_copy = item.to_string();
                            let text_bytes = text_to_copy.as_bytes();

                            let clipboard = clipboard_clone.borrow_mut();
                            let atom_clipboard = clipboard.setter.atoms.clipboard;
                            let atom_utf8string = clipboard.setter.atoms.utf8_string;

                            if let Err(err) =
                                clipboard.store(atom_clipboard, atom_utf8string, text_bytes)
                            {
                                eprintln!("Error copying text to clipboard: {}", err);
                            } else {
                                println!("Text copied to clipboard: {}", text_to_copy);
                            }
                        }
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

    // Watch for file modifications
    glib::timeout_add_local(std::time::Duration::from_millis(1000), move || {
        let mut buffer = [0; 4096];
        let events = inotify.read_events_blocking(&mut buffer).expect("Failed to read events");
        for event in events {
            if event.mask.contains(EventMask::MODIFY) {
                // Reload the file and update the GUI
                std::thread::sleep(std::time::Duration::from_millis(100));
                if let Ok(mut json_file) = File::open("clipboard.json") {
                    let mut contents = String::new();
                    json_file
                        .read_to_string(&mut contents)
                        .expect("Failed to read file");

                    // Clear existing buttons
                    vbox.foreach(|child| vbox.remove(child));

                    if let Ok(data) = serde_json::from_str::<Vec<serde_json::Value>>(&contents) {
                        for entry in data {
                            if let Some(item) = entry.get("item").and_then(|x| x.as_str()) {
                                let entry_clone = entry.clone(); // Clone entry before moving into closure
                                let button = gtk::Button::with_label(item);
                                vbox.add(&button);

                                let clipboard_clone = Rc::clone(&clipboard);

                                button.connect_clicked(move |_| {
                                    if let Some(item) = entry_clone.get("item").and_then(|x| x.as_str()) {
                                        let text_to_copy = item.to_string();
                                        let text_bytes = text_to_copy.as_bytes();

                                        let clipboard = clipboard_clone.borrow_mut();
                                        let atom_clipboard = clipboard.setter.atoms.clipboard;
                                        let atom_utf8string = clipboard.setter.atoms.utf8_string;

                                        if let Err(err) =
                                            clipboard.store(atom_clipboard, atom_utf8string, text_bytes)
                                        {
                                            eprintln!("Error copying text to clipboard: {}", err);
                                        } else {
                                            println!("Text copied to clipboard: {}", text_to_copy);
                                        }
                                    }
                                });
                            }
                        }
                    } else {
                        println!("Failed to parse JSON data");
                    }
                } else {
                    println!("Failed to open file");
                }
            }
        }
        return true.into()
        // glib::Continue(true)
    });
}

fn main() {
    let application = gtk::Application::new(
        Some("com.github.gtk-rs.examples.json_buttons"),
        Default::default(),
    );

    application.connect_activate(build_ui);

    application.run();
}
