pub mod colors {
    pub const TYPE_WARNING: u8 = 214;
    pub const MESG_WARNING: u8 = 246;

    pub const TYPE_ERROR: u8 = 202;
    pub const MESG_ERROR: u8 = 209;

    pub const TYPE_INFO: u8 = 111;
    pub const MESG_INFO: u8 = 153;

    pub const TYPE_SUCCESS: u8 = 148;
    pub const MESG_SUCCESS: u8 = 190;
}

pub mod format {
    use super::colors;
    use console::style;

    pub fn warning(msg: String) -> String {
        format!(
            "{}{}",
            style("[warning] ").color256(colors::TYPE_WARNING),
            style(format!("{}", msg)).color256(colors::MESG_WARNING),
        )
    }

    pub fn error(msg: String) -> String {
        format!(
            "{}{}",
            style("  [error] ").color256(colors::TYPE_ERROR),
            style(format!("{}", msg)).color256(colors::MESG_ERROR),
        )
    }

    pub fn info(msg: String) -> String {
        format!(
            "{}{}",
            style("   [info] ").color256(colors::TYPE_INFO),
            style(format!("{}", msg)).color256(colors::MESG_INFO),
        )
    }

    pub fn success(msg: String) -> String {
        format!(
            "{}{}",
            style("[success] ").color256(colors::TYPE_SUCCESS),
            style(format!("{}", msg)).color256(colors::MESG_SUCCESS),
        )
    }
}

pub mod out {
    use super::format;

    pub fn warning(msg: String) {
        println!("{}", format::warning(msg));
    }
    pub fn error(msg: String) {
        println!("{}", format::error(msg));
    }
    pub fn info(msg: String) {
        println!("{}", format::info(msg));
    }
    pub fn success(msg: String) {
        println!("{}", format::success(msg));
    }
}

pub mod err {
    use super::format;

    pub fn warning(msg: String) {
        eprintln!("{}", format::warning(msg));
    }
    pub fn error(msg: String) {
        eprintln!("{}", format::error(msg));
    }
    pub fn info(msg: String) {
        eprintln!("{}", format::info(msg));
    }
    pub fn success(msg: String) {
        eprintln!("{}", format::success(msg));
    }
}
