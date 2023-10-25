use crate::controllers::controller_client::Controller;
use crate::errors::GitError;
use gtk::prelude::*;
use std::rc::Rc;

pub struct View {
    controller: Controller,
    window: gtk::Window,
    button_clear: gtk::Button,
    button_send: gtk::Button,
    entry: gtk::Entry,
    response: Rc<gtk::TextView>,
}

impl View {
    pub fn new(controller: Controller) -> Result<View, GitError> {
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

        Ok(View {
            controller,
            window,
            button_clear,
            button_send,
            entry,
            response,
        })
    }

    pub fn start_view(self) -> Result<(), GitError> {
        let response_for_button_send = Rc::clone(&self.response);

        self.button_send.connect_clicked(move |_| {
            let command = self.entry.text().to_string();
            self.entry.set_text("");

            let command_input = format!("{}\n\n", &command);

            if let Some(buffer) = response_for_button_send.buffer() {
                let mut end_iter = buffer.end_iter();
                buffer.insert(&mut end_iter, &command_input);
            };

            //Arreglar.
            let result = self.controller.send_command(&command);
            let _ = match result {
                Ok(_) => Ok(()),
                Err(e) => {
                    eprintln!("Error al enviar el comando: {}", command);
                    eprintln!("Info del error: {}", e.message());
                    Err(e)
                }
            };
        });
        self.button_clear.connect_clicked(move |_| {
            if let Some(buffer) = self.response.buffer() {
                buffer.set_text("");
            };
        });

        self.window.show_all();

        gtk::main();

        Ok(())
    }
}
