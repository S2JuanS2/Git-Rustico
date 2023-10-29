use crate::controllers::controller_client::Controller;
use crate::errors::GitError;
use gtk::prelude::*;
use std::rc::Rc;

const DIV: &str = "-----------------------------------------";

#[derive(Clone)]
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
        let button_clear: gtk::Button = builder.object("button_clear").ok_or(GitError::ObjectBuildFailed)?;        
        let button_send: gtk::Button = builder.object("button_send").ok_or(GitError::ObjectBuildFailed)?;
        let entry: gtk::Entry = builder.object("entry_console").ok_or(GitError::ObjectBuildFailed)?;
        let response: Rc<gtk::TextView> = Rc::new(
            builder
                .object("console")
                .ok_or(GitError::ObjectBuildFailed)?,
        );
        Ok(View {
            controller,
            window,
            button_clear,
            button_send,
            entry,
            response,
        })
    }
    fn connect_buttons(self){

        let response_for_button_send = Rc::clone(&self.response);

        self.button_send.connect_clicked(move |_| {
            let command = self.entry.text().to_string();
            self.entry.set_text("");

            let result = self.controller.send_command(&command);

            if let Some(buffer) = response_for_button_send.buffer() {
                let mut end_iter = buffer.end_iter();
                match result {
                    Ok(response) => {
                        let response_format = format!("\n{}\n{}",DIV,response);
                        buffer.insert(&mut end_iter, &response_format);
                    }
                    Err(e) => {
                        let error_message = format!(
                            "\n{}\nError al enviar el comando '{}'\n[Error] {}\n",
                            DIV,
                            command,
                            e.message()
                        );
                        buffer.insert(&mut end_iter, &error_message);
                    }
                }
            }
        });
        self.button_clear.connect_clicked(move |_| {
            if let Some(buffer) = self.response.buffer() {
                buffer.set_text("");
            };
        });
    }
    
    pub fn start_view(self) -> Result<(), GitError> {
        let this = self.clone();
        this.connect_buttons();

        self.window.connect_destroy(|_| {
            gtk::main_quit();
        });

        self.window.show_all();
        gtk::main();

        Ok(())
    }
}
