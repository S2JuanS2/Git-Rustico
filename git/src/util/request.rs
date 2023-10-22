use crate::consts::END_OF_STRING;
use crate::util::pkt_line::add_length_prefix;

pub enum RequestCommand {
    UploadPack,
    ReceivePack,
    UploadArchive,
}

impl RequestCommand {
    fn to_string(&self) -> &str {
        match self {
            RequestCommand::UploadPack => "git-upload-pack",
            RequestCommand::ReceivePack => "git-receive-pack",
            RequestCommand::UploadArchive => "git-upload-archive",
        }
    }
}

pub fn create_git_request(
    command: RequestCommand,
    repo: String,
    ip: String,
    port: String,
) -> String {
    let mut len: usize = 0;

    let command = format!("{} ", command.to_string());
    len += command.len();

    let project = format!("/{}{}", repo, END_OF_STRING);
    len += project.len(); // El len cuenta el END_OF_STRING

    let host = format!("host={}:{}{}", ip, port, END_OF_STRING);
    len += host.len(); // El len cuenta el END_OF_STRING

    let message = format!("{}{}{}", command, project, host);
    add_length_prefix(&message, len)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_git_request_upload_pack() {
        let message = create_git_request(
            RequestCommand::UploadPack,
            "project.git".to_string(),
            "myserver.com".to_string(),
            "9418".to_string(),
        );
        assert_eq!(
            message,
            "0038git-upload-pack /project.git\0host=myserver.com:9418\0"
        );
    }

    #[test]
    fn test_create_git_request_receive_pack() {
        let message = create_git_request(
            RequestCommand::ReceivePack,
            "project.git".to_string(),
            "127.0.0.2".to_string(),
            "12030".to_string(),
        );
        assert_eq!(
            message,
            "0037git-receive-pack /project.git\0host=127.0.0.2:12030\0"
        );
    }

    #[test]
    fn test_create_git_request_upload_archive() {
        let message = create_git_request(
            RequestCommand::UploadArchive,
            "project.git".to_string(),
            "250.250.250.250".to_string(),
            "8080".to_string(),
        );
        assert_eq!(
            message,
            "003egit-upload-archive /project.git\0host=250.250.250.250:8080\0"
        );
    }
}
