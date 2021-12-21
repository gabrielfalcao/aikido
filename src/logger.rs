use console::style;

pub mod colors {
    pub const TYPE_WARNING: u8 = 214;
    pub const MESG_WARNING: u8 = 246;

    pub const TYPE_ERROR: u8 = 202;
    pub const MESG_ERROR: u8 = 209;

    pub const TYPE_INFO: u8 = 202;
    pub const MESG_INFO: u8 = 209;

    pub const TYPE_SUCCESS: u8 = 184;
    pub const MESG_SUCCESS: u8 = 192;
}

pub fn warning(msg: String) {
    println!(
        "{}{}",
        style("[warning] ").color256(colors::TYPE_WARNING),
        style(format!("{}", msg)).color256(colors::MESG_WARNING),
    );
}

pub fn error(msg: String) {
    println!(
        "{}{}",
        style("[error] ").color256(colors::TYPE_ERROR),
        style(format!("{}", msg)).color256(colors::MESG_ERROR),
    );
}

pub fn info(msg: String) {
    println!(
        "{}{}",
        style("[info] ").color256(colors::TYPE_INFO),
        style(format!("{}", msg)).color256(colors::MESG_INFO),
    );
}

pub fn success(msg: String) {
    println!(
        "{}{}",
        style("[success] ").color256(colors::TYPE_SUCCESS),
        style(format!("{}", msg)).color256(colors::MESG_SUCCESS),
    );
}
