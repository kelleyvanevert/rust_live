use live_editor_state::{LineData, Token};

pub enum CodeToken {
    Keyword { col: usize, text: String },
    Text { col: usize, text: String },
    Widget { col: usize, id: usize, width: usize },
}

fn is_keyword(word: &str) -> bool {
    word == "def"
}

pub fn syntax_highlight(data: &LineData) -> Vec<(usize, Vec<CodeToken>)> {
    data.lines()
        .iter()
        .map(|line| {
            let mut col = 0;

            let mut tokens: Vec<CodeToken> = vec![];

            let mut space: String = "".into();
            let mut word: String = "".into();

            for &cell in line.iter() {
                match cell {
                    Token::Widget { id, width } => {
                        if word.len() > 0 {
                            tokens.push(if is_keyword(&word) {
                                CodeToken::Keyword { col, text: word }
                            } else {
                                CodeToken::Text { col, text: word }
                            });

                            word = "".into();
                        }

                        if space.len() > 0 {
                            tokens.push(CodeToken::Text { col, text: space });

                            space = "".into();
                        }

                        tokens.push(CodeToken::Widget { col, id, width });
                    }
                    Token::Char(ch) => {
                        if ch == ' ' {
                            if word.len() > 0 {
                                tokens.push(if is_keyword(&word) {
                                    CodeToken::Keyword { col, text: word }
                                } else {
                                    CodeToken::Text { col, text: word }
                                });

                                word = "".into();
                            }

                            space.push(ch);
                        } else {
                            if space.len() > 0 {
                                tokens.push(CodeToken::Text { col, text: space });

                                space = "".into();
                            }

                            word.push(ch);
                        }
                    }
                }

                col += cell.width();
            }

            if word.len() > 0 {
                tokens.push(if is_keyword(&word) {
                    CodeToken::Keyword { col, text: word }
                } else {
                    CodeToken::Text { col, text: word }
                });
            }

            if space.len() > 0 {
                tokens.push(CodeToken::Text { col, text: space });
            }

            tokens
        })
        .enumerate()
        .collect()
}
