use crate::controllers::controller_client::Controller;
use crate::errors::GitError;
use crate::views::buttons::*;
use crate::views::entries::*;
use gtk::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

const RESPONSE: &str = "\n======================================================================================================\n";

const HELP: &str = "En el archivo de configuración del cliente se debe indicar en el src la ruta donde se creará el repositorio Git \n\n 
                    RustTeam <3";

#[derive(Clone)]
pub struct View {
    controller: Rc<RefCell<Controller>>,
    window: gtk::Window,
    window_dialog_clone: gtk::Window,
    window_dialog_cat_file: gtk::Window,
    window_dialog_hash_object: gtk::Window,
    window_dialog_fetch: gtk::Window,
    window_dialog_push: gtk::Window,
    window_dialog_pull: gtk::Window,
    buttons: HashMap<String, gtk::Button>,
    entries: HashMap<String, Rc<gtk::Entry>>,
    response: Rc<gtk::TextView>,
    label_user: gtk::Label,
    label_mail: gtk::Label,
    label_branch: gtk::Label,
    label_path: gtk::Label,
    label_branches: gtk::Label,
}

impl View {
    pub fn new(controller: Controller) -> Result<View, GitError> {
        if gtk::init().is_err() {
            return Err(GitError::GtkFailedInitiliaze);
        }
        let glade_src = include_str!("git_ppal.glade");
        let builder = gtk::Builder::from_string(glade_src);

        let buttons_ids = get_buttons();
        let entries_ids = get_entries();

        let mut buttons: HashMap<String, gtk::Button> = HashMap::new();
        for button_id in buttons_ids {
            let button: gtk::Button = builder
                .object(&button_id)
                .ok_or(GitError::ObjectBuildFailed)?;
            buttons.insert(button_id, button);
        }
        let mut entries: HashMap<String, Rc<gtk::Entry>> = HashMap::new();

        for entry_id in entries_ids {
            let entry: Rc<gtk::Entry> = Rc::new(
                builder
                    .object(&entry_id)
                    .ok_or(GitError::ObjectBuildFailed)?,
            );
            entries.insert(entry_id, entry);
        }
        let window: gtk::Window = builder
            .object("window_ppal")
            .ok_or(GitError::ObjectBuildFailed)?;
        let window_dialog_clone: gtk::Window = builder
            .object("window_dialog_clone")
            .ok_or(GitError::ObjectBuildFailed)?;
        let window_dialog_hash_object: gtk::Window = builder
            .object("window_dialog_hash-object")
            .ok_or(GitError::ObjectBuildFailed)?;
        let window_dialog_cat_file: gtk::Window = builder
            .object("window_dialog_cat-file")
            .ok_or(GitError::ObjectBuildFailed)?;
        let window_dialog_fetch: gtk::Window = builder
            .object("window_dialog_fetch")
            .ok_or(GitError::ObjectBuildFailed)?;
        let window_dialog_push: gtk::Window = builder
            .object("window_dialog_push")
            .ok_or(GitError::ObjectBuildFailed)?;
        let window_dialog_pull: gtk::Window = builder
            .object("window_dialog_pull")
            .ok_or(GitError::ObjectBuildFailed)?;
        let response: Rc<gtk::TextView> = Rc::new(
            builder
                .object("console")
                .ok_or(GitError::ObjectBuildFailed)?,
        );
        let label_user: gtk::Label = builder.object("user").ok_or(GitError::ObjectBuildFailed)?;
        let label_branch: gtk::Label = builder.object("label_branch").ok_or(GitError::ObjectBuildFailed)?;
        let label_mail: gtk::Label = builder.object("mail").ok_or(GitError::ObjectBuildFailed)?;
        let label_path: gtk::Label = builder.object("path").ok_or(GitError::ObjectBuildFailed)?;
        let label_branches: gtk::Label = builder.object("list_branches").ok_or(GitError::ObjectBuildFailed)?;

        let controller = Rc::new(RefCell::new(controller));
        Ok(View {
            controller,
            window,
            window_dialog_clone,
            window_dialog_hash_object,
            window_dialog_cat_file,
            window_dialog_fetch,
            window_dialog_push,
            window_dialog_pull,
            buttons,
            entries,
            response,
            label_user,
            label_mail,
            label_branch,
            label_path,
            label_branches,
        })
    }
    fn set_label_user(&mut self) {
        let controller = Rc::clone(&self.controller);
        let binding = controller.borrow_mut();
        let user_name = binding.get_name_client();
        self.label_user.set_text(user_name);
    }
    fn set_label_mail(&mut self) {
        let controller = Rc::clone(&self.controller);
        let binding = controller.borrow_mut();
        let user_mail = binding.get_mail_client();
        self.label_mail.set_text(user_mail);
    }
    fn set_label_branch(&mut self) {
        let controller = Rc::clone(&self.controller);
        let binding = controller.borrow_mut();
        let current_branch = binding.get_current_branch();
        self.label_branch.set_text(current_branch);
    }
    fn set_label_path(&mut self) {
        let controller = Rc::clone(&self.controller);
        let binding = controller.borrow_mut();
        let text_format = binding.get_path_client();
        self.label_path.set_text(text_format);
    }

    fn response_write_buffer(result: Result<String, GitError>, response: Rc<gtk::TextView>, cmd: &str) {
        if let Some(buffer) = response.buffer() {
            let mut end_iter = buffer.end_iter();
            match result {
                Ok(response) => {
                    let response_format = format!("{}\n$ {} \n{}", RESPONSE, cmd, response);
                    buffer.insert(&mut end_iter, &response_format);
                }
                Err(e) => {
                    let error_message = format!(
                        "{}\n$ {} \nError al enviar el comando.\n[Error] {}\n",
                        RESPONSE,
                        cmd,
                        e.message()
                    );
                    buffer.insert(&mut end_iter, &error_message);
                }
            }
        }
    }

    fn connect_button_cmd(
        &mut self,
        entry_cmd: &str,
        button_cmd: &str,
        git_cmd: String,
        window: gtk::Window,
    ) {
        let controller = Rc::clone(&self.controller);
        let response = Rc::clone(&self.response);
        if let Some(entry) = self.entries.get(entry_cmd) {
            let entry_clone = Rc::clone(entry);
            if let Some(button) = self.buttons.get(button_cmd) {
                button.connect_clicked(move |_| {
                    window.hide();
                    let entry_format = format!("{} {}", git_cmd, entry_clone.text());
                    entry_clone.set_text("");
                    let result = controller.borrow_mut().send_command(&entry_format);
                    Self::response_write_buffer(result, Rc::clone(&response), &entry_format);
                });
            }
        };
    }

    fn connect_button_cat_file(&self) {
        let dialog = self.window_dialog_cat_file.clone();
        if let Some(button) = self.buttons.get(BUTTON_CAT_FILE) {
            button.connect_clicked(move |_| {
                dialog.show_all();
            });
        }
    }
    fn connect_button_hash_object(&self) {
        let dialog = self.window_dialog_hash_object.clone();

        if let Some(button) = self.buttons.get(BUTTON_HASH_OBJECT) {
            button.connect_clicked(move |_| {
                dialog.show_all();
            });
        }
    }
    fn connect_button_clone(&self) {
        let dialog = self.window_dialog_clone.clone();

        if let Some(button) = self.buttons.get(BUTTON_CLONE) {
            button.connect_clicked(move |_| {
                dialog.show_all();
            });
        }
    }
    fn connect_button_fetch(&self) {
        let dialog = self.window_dialog_fetch.clone();

        if let Some(button) = self.buttons.get(BUTTON_FETCH) {
            button.connect_clicked(move |_| {
                dialog.show_all();
            });
        }
    }
    fn connect_button_push(&self) {
        let dialog = self.window_dialog_push.clone();

        if let Some(button) = self.buttons.get(BUTTON_PUSH) {
            button.connect_clicked(move |_| {
                dialog.show_all();
            });
        }
    }
    fn connect_button_pull(&self) {
        let dialog = self.window_dialog_pull.clone();

        if let Some(button) = self.buttons.get(BUTTON_PULL) {
            button.connect_clicked(move |_| {
                dialog.show_all();
            });
        }
    }
    fn connect_button_send(&self) {
        let response = Rc::clone(&self.response);
        let controller = Rc::clone(&self.controller);
        if let Some(entry) = self.entries.get(ENTRY_CONSOLE) {
            let entry_send = Rc::clone(entry);
            if let Some(button) = self.buttons.get(BUTTON_SEND) {
                button.connect_clicked(move |_| {
                    let command = entry_send.text().to_string();
                    entry_send.set_text("");
                    let result = controller.borrow_mut().send_command(&command);
                    Self::response_write_buffer(result, Rc::clone(&response), &command);
                });
            }
        };
    }
    fn connect_button_clear(&self) {
        if let Some(button) = self.buttons.get(BUTTON_CLEAR) {
            if let Some(buffer) = self.response.buffer() {
                button.connect_clicked(move |_| {
                    buffer.set_text("");
                });
            }
        }
    }
    fn connect_button_help(&self) {
        if let Some(button) = self.buttons.get(BUTTON_HELP) {
            if let Some(buffer) = self.response.buffer() {
                button.connect_clicked(move |_| {
                    buffer.set_text(HELP);
                });
            }
        }
    }
    fn clicked_buttons(&self){
        for button in self.buttons.values() {
            let controller = Rc::clone(&self.controller);
            let label_branch = self.label_branch.clone();
            let label_path = self.label_path.clone();
            let label_branches = self.label_branches.clone();
            button.connect_clicked(move |_| {
                let _ = controller.borrow_mut().set_current_branch();
                controller.borrow_mut().set_label_branch(&label_branch);
                controller.borrow_mut().set_label_path(&label_path);
                controller.borrow_mut().set_branch_list(&label_branches);
            });
        }
    }
    fn connect_button_with_entry(&self, entry_cmd: &str, button_cmd: &str, git_cmd: String) {
        let controller = Rc::clone(&self.controller);
        let response = Rc::clone(&self.response);
        if let Some(entry) = self.entries.get(entry_cmd) {
            let entry_branch = Rc::clone(entry);
            if let Some(button) = self.buttons.get(button_cmd) {
                button.connect_clicked(move |_| {
                    let entry_format = format!("{} {}", git_cmd, entry_branch.text());
                    entry_branch.set_text("");
                    let result = controller.borrow_mut().send_command(&entry_format);
                    Self::response_write_buffer(result, Rc::clone(&response), &entry_format);
                });
            }
        };
    }
    fn connect_button_with_not_entry(&self, button_cmd: &str, git_cmd: String) {
        let controller = Rc::clone(&self.controller);
        let response = Rc::clone(&self.response);

        if let Some(button) = self.buttons.get(button_cmd) {
            button.connect_clicked(move |_| {
                let result = controller.borrow_mut().send_command(&git_cmd);
                Self::response_write_buffer(result, Rc::clone(&response), &git_cmd);
            });
        }
    }
    fn destroy_dialogs(&self) {
        let window = self.window_dialog_push.clone();
        self.window_dialog_push.connect_delete_event(move |_, _| {
            window.hide_on_delete()
        });
        let window = self.window_dialog_pull.clone();
        self.window_dialog_pull.connect_delete_event(move |_, _| {
            window.hide_on_delete()
        });
        let window = self.window_dialog_fetch.clone();
        self.window_dialog_fetch.connect_delete_event(move |_, _| {
            window.hide_on_delete()
        });
        let window = self.window_dialog_clone.clone();
        self.window_dialog_clone.connect_delete_event(move |_, _| {
            window.hide_on_delete()
        });
        let window = self.window_dialog_cat_file.clone();
        self.window_dialog_cat_file.connect_delete_event(move |_, _| {
            window.hide_on_delete()
        });
        let window = self.window_dialog_hash_object.clone();
        self.window_dialog_hash_object.connect_delete_event(move |_, _| {
            window.hide_on_delete()
        });
    }
    
    fn connect_buttons(&mut self) {
        self.connect_button_with_entry(ENTRY_CHECKOUT, BUTTON_CHECKOUT, "git checkout".to_string());
        self.connect_button_with_entry(ENTRY_ADD_RM, BUTTON_ADD, "git add".to_string());
        self.connect_button_with_entry(ENTRY_ADD_RM, BUTTON_RM, "git rm".to_string());
        self.connect_button_with_entry(ENTRY_COMMIT, BUTTON_COMMIT, "git commit -m".to_string());
        self.connect_button_with_entry(ENTRY_MERGE, BUTTON_MERGE, "git merge".to_string());
        self.connect_button_with_entry(ENTRY_BRANCH, BUTTON_BRANCH, "git branch -l".to_string());
        self.connect_button_with_entry(ENTRY_BRANCH, BUTTON_BRANCH_CREATE, "git branch".to_string());
        self.connect_button_with_entry(ENTRY_BRANCH, BUTTON_BRANCH_REMOVE, "git branch -d".to_string());
        self.connect_button_with_entry(ENTRY_REMOTE, BUTTON_REMOTE, "git remote".to_string());
        self.connect_button_with_entry(ENTRY_LS, BUTTON_LS_FILES, "git ls-files".to_string());
        self.connect_button_with_entry(ENTRY_LS, BUTTON_LS_TREE, "git ls-tree".to_string());
        self.connect_button_with_entry(ENTRY_CHECK_IGNORE, BUTTON_CHECK_IGNORE, "git check-ignore".to_string());
        self.connect_button_with_entry(ENTRY_TAG, BUTTON_TAG, "git tag".to_string());
        self.connect_button_with_entry(ENTRY_TAG, BUTTON_TAG_CREATE, "git tag -a".to_string());
        self.connect_button_with_entry(ENTRY_TAG, BUTTON_TAG_DELETE, "git tag -d".to_string());
        self.connect_button_with_entry(ENTRY_REBASE, BUTTON_REBASE, "git rebase".to_string());

        let commands = [
            (BUTTON_INIT, "git init".to_string()),
            (BUTTON_STATUS, "git status".to_string()),
            (BUTTON_SHOW_REF, "git show-ref".to_string()),
            (BUTTON_LOG, "git log".to_string()),
        ];

        for (button, command) in commands {
            self.connect_button_with_not_entry(button, command);
        }

        self.connect_button_send();
        self.connect_button_clear();
        self.connect_button_help();
        self.connect_button_clone();
        self.connect_button_cat_file();
        self.connect_button_hash_object();
        self.connect_button_fetch();
        self.connect_button_push();
        self.connect_button_pull();

        let window_clone = self.window_dialog_clone.clone();
        let window_cat_file = self.window_dialog_cat_file.clone();
        let window_hash_object = self.window_dialog_hash_object.clone();
        let window_fetch = self.window_dialog_fetch.clone();
        let window_push = self.window_dialog_push.clone();
        let window_pull = self.window_dialog_pull.clone();

        self.connect_button_cmd(
            ENTRY_FETCH,
            BUTTON_CMD_FETCH,
            "git fetch".to_string(),
            window_fetch,
        );
        self.connect_button_cmd(
            ENTRY_CLONE,
            BUTTON_CMD_CLONE,
            "git clone".to_string(),
            window_clone,
        );
        self.connect_button_cmd(
            ENTRY_CAT_FILE,
            BUTTON_CMD_CAT_FILE_P,
            "git cat-file -p".to_string(),
            window_cat_file.clone(),
        );
        self.connect_button_cmd(
            ENTRY_CAT_FILE,
            BUTTON_CMD_CAT_FILE_T,
            "git cat-file -t".to_string(),
            window_cat_file.clone(),
        );
        self.connect_button_cmd(
            ENTRY_CAT_FILE,
            BUTTON_CMD_CAT_FILE_S,
            "git cat-file -s".to_string(),
            window_cat_file,
        );
        self.connect_button_cmd(
            ENTRY_HASH_OBJECT,
            BUTTON_CMD_HASH_OBJECT,
            "git hash-object".to_string(),
            window_hash_object,
        );
        self.connect_button_cmd(
            ENTRY_PUSH,
            BUTTON_CMD_PUSH,
            "git push".to_string(),
            window_push,
        );
        self.connect_button_cmd(
            ENTRY_PULL,
            BUTTON_CMD_PULL,
            "git pull".to_string(),
            window_pull,
        );
    }

    pub fn start_view(&mut self) -> Result<(), GitError> {
        self.connect_buttons();

        self.window.connect_destroy(|_| {
            gtk::main_quit();
        });
        
        self.destroy_dialogs();
        self.clicked_buttons();
        self.set_label_user();
        self.set_label_mail();
        self.set_label_branch();
        self.set_label_path();

        self.window.show_all();
        gtk::main();

        Ok(())
    }
}
