use std::net::TcpStream;
use std::path::Path;
use crate::commands::commit::builder_commit_log;
use crate::commands::init::git_init;
use crate::consts::{GIT_DIR, REF_HEADS, DIRECTORY, FILE};
use crate::util::files::{create_directory, create_file_replace, create_file};
use crate::util::objects::{read_tree_content, read_blob_content, 
    read_commit_content, builder_object_blob, builder_object_commit, builder_object_tree_clone};
use crate::util::objects::ObjectType;
use crate::errors::GitError;
use crate::models::client::Client;
use crate::util::connections::{
    packfile_negotiation, receive_packfile, start_client,
};
use crate::util::git_request::GitRequest;
use crate::util::references::reference_discovery;
use crate::util::request_command::RequestCommand;

/// Esta función se encarga de llamar a al comando clone con los parametros necesarios
/// ###Parametros:
/// 'args': Vector de strings que contiene los argumentos que se le pasan a la función clone
/// 'client': Cliente que contiene la información del cliente que se conectó
pub fn handle_clone(args: Vec<&str>, client: Client) -> Result<String, GitError> {
    let address: String = client.get_ip().to_string();
    if args.len() != 1 {
        return Err(GitError::CloneMissingRepoError);
    }
    let mut socket = start_client(&address)?;
    let parts = address.split(':').collect::<Vec<&str>>();
    let ip = parts[0].to_string();
    let port = parts[1].to_string();
    git_clone(&mut socket, ip, port, args[0].to_string())
}

/// Esta función se encarga de clonar un repositorio remoto
/// ###Parametros:
/// 'socket': Socket que se utiliza para comunicarse con el servidor
/// 'ip': Dirección ip del servidor
/// 'port': Puerto del servidor
/// 'repo': Nombre del repositorio que se quiere clonar
pub fn git_clone(
    socket: &mut TcpStream,
    ip: String,
    port: String,
    repo: String,
) -> Result<String, GitError> {
    println!("Clonando repositorio remoto: {}", repo);

    // Prepara la solicitud "git-upload-pack" para el servidor
    let message = GitRequest::generate_request_string(RequestCommand::UploadPack, repo, ip, port);

    // Reference Discovery
    let advertised = reference_discovery(socket, message)?;
    println!("advertised: {:?}", advertised);

    // Packfile Negotiation
    packfile_negotiation(socket, advertised.0)?;

    // Packfile Data
    let content = receive_packfile(socket)?;
    
    // Cantidad de objetos recibidos
    let count_objects = content.len();

    // ARREGLAR EL CONFIG PARA OBTENER EL PATH
    let path_dir_cloned = Path::new("test_repo");
    git_init("test_repo")?;
    let git_dir = format!("{}/{}", "test_repo", GIT_DIR);

    let mut i = 0;
    while i < count_objects {
        
        if content[i].0.obj_type == ObjectType::Commit {
            let commit_content = read_commit_content(&content[i].1)?;
            builder_object_commit(&commit_content, &git_dir)?;

            if let Some(line) = advertised.1.get(i){

                let line_str = String::from_utf8_lossy(line);
                let parts: Vec<&str> = line_str.split_whitespace().collect();
                let hash = parts[0];
                let branch = parts[1];

                if let Some(current_branch) = branch.rsplitn(2,'/').next(){
                    let branch_dir = format!("{}/{}/{}/{}", "test_repo", GIT_DIR, REF_HEADS, current_branch);
                    create_file(&branch_dir, hash)?;
                }
                builder_commit_log("test_repo", &commit_content, hash)?;
            }
            i += 1;
    
        }else if content[i].0.obj_type == ObjectType::Tree {
            let tree_content = read_tree_content(&content[i].1)?;
            builder_object_tree_clone(&git_dir, &tree_content)?;
            i = recovery_tree(tree_content, path_dir_cloned, &content, i, &git_dir)?;
            i += 1;
        }
    }
    Ok("Clonación exitosa!".to_string())
}

fn recovery_tree(tree_content: String,
    path_dir_cloned: &Path, 
    content: &Vec<(crate::util::objects::ObjectEntry, 
    Vec<u8>)>, mut i: usize, 
    repo: &str
    ) -> Result<usize, GitError>{
    
    for line in tree_content.lines(){
        let parts: Vec<&str> = line.split_whitespace().collect();

        let mode = parts[0];
        let file_name = parts[1];
        let _hash = parts[2];
        
        let path_dir_cloned = path_dir_cloned.join(file_name);
        if mode == FILE {
            i += 1;
            let blob_content = read_blob_content(&content[i].1)?;
            let blob_content_bytes = blob_content.clone();
            builder_object_blob(blob_content_bytes.into_bytes(), repo)?;
            if let Some(str_path) = path_dir_cloned.to_str(){
                create_file_replace(str_path, &blob_content)?;
            }
        }else if mode == DIRECTORY {
            create_directory(&path_dir_cloned).expect("Error");
            i += 1;
            let tree_content = read_tree_content(&content[i].1)?;
            builder_object_tree_clone(repo, &tree_content)?;
            i = recovery_tree(tree_content, &path_dir_cloned, content, i, repo)?;
        }
    }
    Ok(i)

}
