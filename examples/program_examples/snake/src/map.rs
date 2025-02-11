use std::collections::HashMap;
use std::io::{stdout, Write};

use crossterm::{queue, Colorize, Goto, PrintStyledFont, Result};
use rand;
use rand::distributions::{IndependentSample, Range};

use super::variables::{Position, Size};

pub struct Map {
    pub size: Size,
    pub foot_pos: Position,
}

impl Map {
    pub fn new(size: Size) -> Self {
        Map {
            size: size,
            foot_pos: Position::new(0, 0),
        }
    }

    // render the map on the screen.
    pub fn render_map(&mut self, free_positions: &mut HashMap<String, Position>) -> Result<()> {
        for y in 0..self.size.height {
            for x in 0..self.size.height {
                if (y == 0 || y == self.size.height - 1) || (x == 0 || x == self.size.width - 1) {
                    queue!(
                        stdout(),
                        Goto(x as u16, y as u16),
                        PrintStyledFont("█".magenta())
                    )?;
                } else {
                    free_positions.insert(format!("{},{}", x, y), Position::new(x, y));
                }
            }
        }
        Ok(())
    }

    pub fn is_out_of_bounds(&self, new_pos: Position) -> bool {
        if (new_pos.x == 0 || new_pos.x == self.size.width)
            || (new_pos.y == 0 || new_pos.y == self.size.height)
        {
            return true;
        }

        return false;
    }

    pub fn spawn_food(&mut self, free_positions: &HashMap<String, Position>) -> Result<()> {
        let index = Range::new(0, free_positions.len()).ind_sample(&mut rand::thread_rng());
        self.foot_pos = free_positions.values().skip(index).next().unwrap().clone();
        self.draw_food()
    }

    fn draw_food(&self) -> Result<()> {
        queue!(
            stdout(),
            Goto(self.foot_pos.x as u16, self.foot_pos.y as u16),
            PrintStyledFont("$".green())
        )
    }
}
