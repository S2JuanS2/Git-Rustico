use crate::controllers::controller_client::Controller;
use crate::errors::GitError;
use gtk::prelude::*;
use std::rc::Rc;

const DIV: &str = "Response: ";

#[derive(Clone)]
pub struct View {
    controller: Controller,
    window: gtk::Window,
    button_clear: gtk::Button,
    button_send: gtk::Button,
    button_init: gtk::Button,
    button_branch: gtk::Button,
    entry: Rc<gtk::Entry>,
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
        let button_clear: gtk::Button = builder
            .object("button_clear")
            .ok_or(GitError::ObjectBuildFailed)?;
        let button_send: gtk::Button = builder
            .object("button_send")
            .ok_or(GitError::ObjectBuildFailed)?;
        let button_init: gtk::Button = builder
            .object("button_init")
            .ok_or(GitError::ObjectBuildFailed)?;
        let button_branch: gtk::Button = builder
            .object("button_branch")
            .ok_or(GitError::ObjectBuildFailed)?;
        let entry: Rc<gtk::Entry> = Rc::new(
            builder
                .object("entry_console")
                .ok_or(GitError::ObjectBuildFailed)?,
        );
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
            button_init,
            button_branch,
            entry,
            response,
        })
    }
    fn response_write_buffer(result: Result<String, GitError>, response: Rc<gtk::TextView>) {
        if let Some(buffer) = response.buffer() {
            let mut end_iter = buffer.end_iter();
            match result {
                Ok(response) => {
                    let response_format = format!("{}\n{}", DIV, response);
                    buffer.insert(&mut end_iter, &response_format);
                }
                Err(e) => {
                    let error_message = format!(
                        "{}\nError al enviar el comando.\n[Error] {}\n",
                        DIV,
                        e.message()
                    );
                    buffer.insert(&mut end_iter, &error_message);
                }
            }
        }
    }
    fn connect_button_init(&self) {
        let controller = self.controller.clone();
        let response = Rc::clone(&self.response);

        self.button_init.connect_clicked(move |_| {
            let result = controller.send_command("git init");

            Self::response_write_buffer(result, Rc::clone(&response));
        });
    }
    fn connect_button_clear(&self) {
        if let Some(buffer) = self.response.buffer() {
            self.button_clear.connect_clicked(move |_| {
                buffer.set_text("");
            });
        }
    }
    fn connect_button_branch(&self) {
        let controller = self.controller.clone();
        let response = Rc::clone(&self.response);

        self.button_branch.connect_clicked(move |_| {
            let result = controller.send_command("git branch");

            Self::response_write_buffer(result, Rc::clone(&response));
        });
    }
    fn connect_button_send(&self) {
        let response = Rc::clone(&self.response);
        let entry = Rc::clone(&self.entry);
        let controller = self.controller.clone();

        self.button_send.connect_clicked(move |_| {
            let command = entry.text().to_string();
            entry.set_text("");
            let result = controller.send_command(&command);
            Self::response_write_buffer(result, Rc::clone(&response));
        });
    }
    fn connect_buttons(self) {
        self.connect_button_init();
        self.connect_button_clear();
        self.connect_button_send();
        self.connect_button_branch();
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
