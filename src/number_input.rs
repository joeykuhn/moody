use std::ops::RangeInclusive;
use serde::{Serialize, Deserialize};
use crossterm::event::{KeyCode};
use ratatui::style::{Color, Modifier, Style};
use ratatui::Frame;
use ratatui::widgets::Paragraph;
use ratatui::layout::Rect;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct NumberInput {
    pub value : String,
    range : Option<RangeInclusive<u64>>,
}

impl NumberInput {

    pub fn with_range (&mut self, range: RangeInclusive<u64>) {
        self.range = Some(range);
    }

    pub fn candidate_below_maximum(&self, candidate : &str) -> bool {
        let Some(range) = &self.range else {
            return true;
        };
        match candidate.parse::<u64>() {
            Ok(value) => value <= *range.end(),
            Err(_) => false,
        }
    }

    pub fn candidate_above_minimum(&self, candidate : &str) -> bool {
        let Some(range) = &self.range else {
            return true;
        };

        match candidate.parse::<u64>() {
            Ok(value) => value >= *range.start(),
            Err(_) => false,
        }
    }

    pub fn in_range(&self) -> bool {
        let Some(range) = &self.range else {
            return true;
        };
        
        match self.value.parse::<u64>() {
            Ok(value) => (value >= *range.start()) && (value <= *range.end()),
            Err(_) => false,
        }

    }

    pub fn handle_key (&mut self, key : &crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let mut cand = self.value.clone();
                cand.push(c);
                if self.candidate_below_maximum(&cand) {
                    self.value = cand;
                }
            }
            KeyCode::Backspace => {
                self.value.pop();
            }
            _ => ()
        }
    }

    

    pub fn render (&self, frame: &mut Frame, area: Rect, focused: bool) {
        let mut style = Style::default();
        if focused {
            style = style.add_modifier(Modifier::REVERSED)
        } 

        if !self.in_range() {
            style = style.bg(Color::Red);
        }

        let paragraph = Paragraph::new(self.value.as_str())
                                    .style(style);
        frame.render_widget(paragraph, area);
    }
}