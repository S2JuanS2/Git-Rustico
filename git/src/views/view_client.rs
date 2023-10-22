use crate::controllers::controller_client::Controller;
use crate::errors::GitError;
use gtk::prelude::*;
use std::rc::Rc;

pub struct View {
    controller: Controller,
}

impl View {
    pub fn new(controller: Controller) -> View {
        View { controller }
    }

    pub fn start_view(self) -> Result<(), GitError> {
        if gtk::init().is_err() {
            return Err(GitError::GtkFailedInitiliaze);
        }
        let glade_src = include_str!("git_ppal.glade");

        let builder = gtk::Builder::from_string(glade_src);

        let window: gtk::Window = match builder.object("window1") {
            Some(window) => window,
            None => {
                return Err(GitError::ObjectBuildFailed);
            }
        };
        let button_clear: gtk::Button = match builder.object("button_clear") {
            Some(button_clear) => button_clear,
            None => {
                return Err(GitError::ObjectBuildFailed);
            }
        };
        let button_send: gtk::Button = match builder.object("button_send") {
            Some(button_send) => button_send,
            None => {
                return Err(GitError::ObjectBuildFailed);
            }
        };
        let entry: gtk::Entry = match builder.object("entry_console") {
            Some(entry) => entry,
            None => {
                return Err(GitError::ObjectBuildFailed);
            }
        };

        let response: Rc<gtk::TextView> = Rc::new(match builder.object("console") {
            Some(response) => response,
            None => {
                return Err(GitError::ObjectBuildFailed);
            }
        });

        let response_for_button_send = Rc::clone(&response);

        button_send.connect_clicked(move |_| {
            let command = entry.text().to_string();
            entry.set_text("");

            let command_input = format!("{}\n\n", &command);

            let buffer = response_for_button_send.buffer().unwrap();
            let mut end_iter = buffer.end_iter();
            buffer.insert(&mut end_iter, &command_input);

            match self.controller.send_command(command) {
                Ok(_) => (),
                Err(_) => return (),
            };
        });
        button_clear.connect_clicked(move |_| {
            let buffer = response.buffer().unwrap();
            buffer.set_text("");
        });

        window.show_all();

        gtk::main();

        Ok(())
    }
}
