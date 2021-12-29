use crate::ioutils::log_to_file;
pub fn log_error(message: String) {
    log_to_file("ironpunk.log", message).unwrap()
}
