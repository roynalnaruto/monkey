use std::convert::TryFrom;
use std::io::{stdout, Write};

use crossterm::{
    cursor::{MoveTo, MoveToNextLine, RestorePosition, SavePosition},
    style::{style, Attribute, Color, Print, PrintStyledContent, StyledContent},
    terminal::{Clear, ClearType, SetTitle},
    QueueableCommand, Result,
};

const RECENT_BLOCKS_TO_DISPLAY: u16 = 5;
const BLOCKCHAIN_DISPLAY_ROW: u16 = 30;

lazy_static! {
    static ref NOTICE_VALID_BLOCK: StyledContent<String> = {
        style("Valid Block".to_string())
            .with(Color::Green)
            .attribute(Attribute::Bold)
    };
    static ref NOTICE_INVALID_BLOCK: StyledContent<String> = {
        style("Invalid Block".to_string())
            .with(Color::Red)
            .attribute(Attribute::Bold)
    };
    static ref GENESIS_BLOCK_DISPLAY: StyledContent<String> = {
        style("GENESIS".to_string())
            .with(Color::DarkYellow)
            .attribute(Attribute::Bold)
    };
}

pub struct Display {
    blockchain_display_column: u16,
    block_count: u16,
}

impl Display {
    pub fn new() -> Self {
        Display {
            blockchain_display_column: 10,
            block_count: 1,
        }
    }

    pub fn init() -> Result<()> {
        let mut stdout = stdout();

        let example_words_style = style("this monkey is awesome").with(Color::DarkYellow);

        stdout
            .queue(SetTitle("Monkey - The p2p toy blockchain"))?
            .queue(MoveToNextLine(3))?
            .queue(Print(
                "Welcome to Monkey! In Monkey world, a block consists of a set of 4 English words.",
            ))?
            .queue(MoveToNextLine(2))?
            .queue(Print(
                "Please enter 4 English words separated by spaces. Example:",
            ))?
            .queue(MoveToNextLine(1))?
            .queue(PrintStyledContent(example_words_style))?
            .queue(MoveToNextLine(2))?
            .queue(Print("Start creating monkey blocks!"))?
            .queue(MoveToNextLine(2))?
            .queue(SavePosition)?
            .queue(MoveTo(2, BLOCKCHAIN_DISPLAY_ROW + 5))?
            .queue(PrintStyledContent(GENESIS_BLOCK_DISPLAY.clone()))?
            .queue(RestorePosition)?;

        stdout.flush()?;

        Ok(())
    }

    pub fn notice_invalid_block() -> Result<()> {
        let mut stdout = stdout();

        stdout
            .queue(MoveTo(0, 2))?
            .queue(Clear(ClearType::CurrentLine))?
            .queue(PrintStyledContent(NOTICE_INVALID_BLOCK.clone()))?
            .queue(RestorePosition)?
            .queue(Clear(ClearType::CurrentLine))?;

        stdout.flush()?;

        Ok(())
    }

    pub fn notice_valid_block() -> Result<()> {
        let mut stdout = stdout();

        stdout
            .queue(MoveTo(0, 2))?
            .queue(Clear(ClearType::CurrentLine))?
            .queue(PrintStyledContent(NOTICE_VALID_BLOCK.clone()))?
            .queue(RestorePosition)?
            .queue(Clear(ClearType::CurrentLine))?;

        stdout.flush()?;

        Ok(())
    }

    pub fn new_block(&mut self, lines: Vec<String>) -> Result<()> {
        let mut stdout = stdout();

        let styled_content = |line: String| {
            style(line)
                .with(Color::DarkYellow)
                .attribute(Attribute::Bold)
        };

        if self.block_count > RECENT_BLOCKS_TO_DISPLAY {
            for i in 0..9 {
                stdout
                    .queue(MoveTo(
                        self.blockchain_display_column,
                        BLOCKCHAIN_DISPLAY_ROW + i + 1,
                    ))?
                    .queue(Clear(ClearType::CurrentLine))?;
            }

            stdout.flush()?;

            self.block_count = 0;
            self.blockchain_display_column = 2;
        }

        for (index, line) in lines.iter().enumerate() {
            let i: u16 = u16::try_from(index).unwrap();

            stdout
                .queue(MoveTo(
                    self.blockchain_display_column,
                    BLOCKCHAIN_DISPLAY_ROW + i + 1,
                ))?
                .queue(PrintStyledContent(styled_content(line.to_string())))?;
        }

        stdout.flush()?;

        self.block_count += 1;
        self.blockchain_display_column += 18;

        Ok(())
    }
}
