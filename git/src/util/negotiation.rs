use super::{advertised::AdvertisedRefs, connections::send_message, pkt_line};
use crate::{consts::NACK, errors::GitError};
use std::{io::Write, net::TcpStream};

pub fn upload_request(
    socket: &mut TcpStream,
    advertised: Vec<AdvertisedRefs>,
) -> Result<(), GitError> {
    for a in advertised {
        let message = match a {
            AdvertisedRefs::Ref {
                obj_id,
                ref_name: _,
            } => format!("want {}\n", obj_id),
            _ => continue,
        };
        let message = pkt_line::add_length_prefix(&message, message.len());
        send_message(socket, message, GitError::UploadRequest)?;
    }
    Ok(())
}

pub fn receive_nack(socket: &mut dyn Write) -> Result<(), GitError> {
    send_message(socket, NACK.to_string(), GitError::PackfileNegotiationNACK)
}
