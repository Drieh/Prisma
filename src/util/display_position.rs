#[derive(Clone, Copy, PartialEq, Debug)]
pub enum DisplayPosition {
    Static,
    Absolute,
    Relative,
    Fixed,
    None,
}
