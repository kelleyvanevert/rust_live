#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

// impl Direction {
//     pub fn is_horizontal(self) -> bool {
//         match self {
//             Self::Right | Self::Left => true,
//             _ => false,
//         }
//     }

//     pub fn is_vertical(self) -> bool {
//         !self.is_horizontal()
//     }
// }
