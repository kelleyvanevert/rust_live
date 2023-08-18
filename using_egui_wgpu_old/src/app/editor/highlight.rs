use super::line_data::{LineData, Token, WidgetInfo};

#[derive(Debug)]
pub enum CodeToken {
    Keyword { col: usize, text: String },
    Text { col: usize, text: String },
    Widget { col: usize, id: usize, width: usize },
}

fn is_keyword(word: &str) -> bool {
    word == "let" || word == "play"
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
                    Token::Widget(WidgetInfo { id, width, .. }) => {
                        if word.len() > 0 {
                            tokens.push(if is_keyword(&word) {
                                CodeToken::Keyword {
                                    col: col - word.len(),
                                    text: word,
                                }
                            } else {
                                CodeToken::Text {
                                    col: col - word.len(),
                                    text: word,
                                }
                            });

                            word = "".into();
                        }

                        if space.len() > 0 {
                            tokens.push(CodeToken::Text {
                                col: col - space.len(),
                                text: space,
                            });

                            space = "".into();
                        }

                        tokens.push(CodeToken::Widget {
                            col: col - width,
                            id,
                            width,
                        });
                    }
                    Token::Char(ch) => {
                        if ch == ' ' {
                            if word.len() > 0 {
                                tokens.push(if is_keyword(&word) {
                                    CodeToken::Keyword {
                                        col: col - word.len(),
                                        text: word,
                                    }
                                } else {
                                    CodeToken::Text {
                                        col: col - word.len(),
                                        text: word,
                                    }
                                });

                                word = "".into();
                            }

                            space.push(ch);
                        } else {
                            if space.len() > 0 {
                                tokens.push(CodeToken::Text {
                                    col: col - space.len(),
                                    text: space,
                                });

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
                    CodeToken::Keyword {
                        col: col - word.len(),
                        text: word,
                    }
                } else {
                    CodeToken::Text {
                        col: col - word.len(),
                        text: word,
                    }
                });
            }

            if space.len() > 0 {
                tokens.push(CodeToken::Text {
                    col: col - space.len(),
                    text: space,
                });
            }

            tokens
        })
        .enumerate()
        .collect()
}
