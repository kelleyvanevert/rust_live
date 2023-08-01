use std::cmp::Ordering;

use super::{line_data::LineData, pos::Pos};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    pub id: usize,
    pub anchor: Option<Pos>,
    pub caret: Pos,
    pub desired_col: Option<i32>,
}

impl Selection {
    pub fn has_selection(&self, lines: &LineData) -> Option<(Pos, Pos)> {
        self.anchor.and_then(|anchor| {
            if anchor == self.caret {
                None
            } else {
                Some(Pos::order(self.caret, anchor))
            }
        })
    }

    pub fn move_caret_to(&mut self, caret: Pos, selecting: bool) {
        if !selecting {
            self.anchor = None;
        } else if self.anchor == Some(caret) {
            // keep invariant: anchor != caret
            self.anchor = None;
        } else if self.anchor.is_none() {
            // anchor before move
            self.anchor = Some(self.caret);
        } else {
            // anchor was already set previously
        }

        self.caret = caret;
    }
}

impl PartialOrd for Selection {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Selection {
    // Orders component-wise: first by `caret`, then by `anchor`
    // Not that we really need this, because in the editor's selection set, selections will never  overlap --- this is just to make `Ord` work correctly in general
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.caret.cmp(&other.caret) {
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => self.anchor.cmp(&other.anchor),
        }
    }
}

// impl From<Pos> for Selection {
//     fn from(caret: Pos) -> Selection {
//         Selection {
//             anchor: None,
//             caret,
//         }
//     }
// }
