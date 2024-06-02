//! Constant variables.

pub static WHITE: u32 = 0xffffff;
pub static _RED: u32 = 0xff0000;
pub static _GREEN: u32 = 0x00ff00;
pub static _BLUE: u32 = 0x0000ff;
pub static _DARKBLUE: u32 = 0x00008b;
pub static LIGHTGREY: u32 = 0xd3d3d3;
pub static GREY: u32 = 0x808080;
pub static DARKGREY: u32 = 0x5a5a5a;
pub static BLACK: u32 = 0x000000;

pub static WINDOW_INIT_X_POS: i64 = 30;
pub static WINDOW_INIT_Y_POS: i64 = 50;

//static WINDOW_WIDTH: i64 = 1024;
//static WINDOW_HEIGHT: i64 = 768;
pub static WINDOW_WIDTH: i64 = 600;
pub static WINDOW_HEIGHT: i64 = 400;
pub static WINDOW_PADDING: i64 = 5;

// defined in noli/src/window.rs
pub static TITLE_BAR_HEIGHT: i64 = 24;

pub static CONTENT_AREA_WIDTH: i64 = WINDOW_WIDTH;
pub static CONTENT_AREA_HEIGHT: i64 = WINDOW_HEIGHT - TITLE_BAR_HEIGHT - TOOLBAR_HEIGHT;

pub static TOOLBAR_HEIGHT: i64 = 26;
pub static ADDRESSBAR_HEIGHT: i64 = 20;

pub static CHAR_WIDTH: i64 = 8;
pub static _CHAR_HEIGHT: i64 = 16;
