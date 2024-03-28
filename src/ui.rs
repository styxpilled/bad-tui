use crossterm::{
    cursor,
    event::{Event, MouseButton},
    queue,
    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
};
use std::{
    cell::{Ref, RefCell},
    io::{Stdout, Write},
    rc::Rc,
};

const BG_RESET: SetBackgroundColor = SetBackgroundColor(Color::Reset);
const FG_RESET: SetForegroundColor = SetForegroundColor(Color::Reset);

pub struct UI<'a> {
    // pub elements: Vec<Block<'a>>,
    pub stdout: &'a mut Stdout,
    pub root: Block<'a>,
    // pub state: &mut State,
    // pub pos: Option<(u8, u8)>,
}

#[derive(Clone, Debug)]
pub struct Context {
    pub max: (u16, u16),
    pub click_pos: Option<(u16, u16)>,
    // pub offset: u8,
    pub bg_color: Color,
}

impl Context {
    pub fn process(&mut self, event: &Event) {
        self.click_pos = match event {
            Event::Mouse(event) => match event.kind {
                crossterm::event::MouseEventKind::Down(MouseButton::Left) => {
                    Some((event.column, event.row))
                }
                crossterm::event::MouseEventKind::Drag(MouseButton::Left) => {
                    Some((event.column, event.row))
                }
                _ => None,
            },
            _ => None,
        };
    }
}

impl<'a> UI<'a> {
    pub fn render(&mut self, ctx: &Context) {
        self.root.calc_self();
        self.root.render(self.stdout, ctx.max);
        queue!(
            self.stdout,
            SetBackgroundColor(Color::Reset),
            SetForegroundColor(Color::Reset)
        )
        .unwrap();
    }

    pub fn process(&mut self, ctx: &Context) {
        self.root.process(ctx.click_pos);
    }
}

pub enum Element<'a> {
    Block(Block<'a>),
    Widget(Widget),
}

///! ORDER IS: COLUMN, ROW
///! WIDTH, HEIGHT
pub struct Block<'a> {
    contents: Vec<RefCell<Element<'a>>>,
    parent: Option<Rc<Block<'a>>>,
    pub pos: (u16, u16),
    pub size: (u16, u16),
    inner_pos: (u16, u16),
    pub direction: Direction,
}

pub enum Direction {
    Vertical,
    Horizontal,
}

impl<'a> Block<'a> {
    pub fn render(&mut self, stdout: &mut Stdout, max: (u16, u16)) {
        // let (max_width, max_height) = max;
        let max_width = self.size.0 + 2;
        let max_height = self.size.1 + 2;
        // Print the top border
        queue!(
            stdout,
            cursor::MoveTo(self.pos.0, self.pos.1),
            Print("#".repeat((max_width - self.pos.0).into()))
        )
        .unwrap();
        let mut row = self.pos.0;
        while row < max_height {
            // Print the left border
            queue!(stdout, cursor::MoveTo(self.pos.0, row), Print("#")).unwrap();
            // Print the right border
            queue!(stdout, cursor::MoveTo(max_width - 1, row), Print("#")).unwrap();
            row += 1;
        }
        // Print the bottom border
        queue!(
            stdout,
            cursor::MoveTo(self.pos.0, max_height),
            Print("#".repeat((max_width - self.pos.0).into()))
        )
        .unwrap();

        queue!(stdout, cursor::MoveTo(self.pos.0 + 1, self.pos.1 + 1)).unwrap();
        let max = (max_width - 2, max_height - 3);
        for el in self.contents.iter_mut() {
            match el.get_mut() {
                Element::Block(block) => {
                    block.render(stdout, max);
                }
                Element::Widget(widget) => {
                    widget.render(stdout);
                }
            }
        }
    }

    pub fn calc_self(&mut self) {
        self.size = (3, 2);
        for el in self.contents.iter_mut() {
            match el.get_mut() {
                Element::Block(block) => {
                    block.calc_self();
                    self.size.0 += block.size.0;
                    self.size.1 += block.size.1;
                }
                Element::Widget(widget) => {
                    widget.calc_self();
                    self.size.0 += widget.size.0;
                    self.size.1 += widget.size.1;
                }
            }
        }
    }

    pub fn process(&mut self, click_pos: Option<(u16, u16)>) {
        for el in self.contents.iter_mut() {
            match el.get_mut() {
                Element::Block(block) => {
                    block.process(click_pos);
                }
                Element::Widget(widget) => {
                    widget.process(click_pos);
                }
            }
        }
    }

    pub fn new(pos: (u16, u16)) -> Self {
        Self {
            parent: None,
            pos,
            size: (3, 3),
            inner_pos: (2, 2),
            contents: vec![],
            direction: Direction::Horizontal,
        }
    }

    pub fn push(&mut self, w: Element<'a>) -> Ref<Element<'a>> {
        // if !self.widgets.is_empty() {
        //     self.offset += self.pad as u8;
        // }
        let mut r = RefCell::new(w);
        match r.get_mut() {
            Element::Block(ref mut block) => {
                block.pos = (
                    block.pos.0 + self.pos.0 + self.inner_pos.0,
                    block.pos.1 + self.pos.1 + self.inner_pos.1,
                );
                block.calc_self();
                match self.direction {
                    Direction::Horizontal => self.inner_pos.0 += block.size.0,
                    Direction::Vertical => self.inner_pos.1 += block.size.1,
                }
            }
            Element::Widget(ref mut widget) => {
                widget.pos = (
                    widget.pos.0 + self.pos.0 + self.inner_pos.0,
                    widget.pos.1 + self.pos.1 + self.inner_pos.1,
                );
                widget.calc_self();
                match self.direction {
                    Direction::Horizontal => self.inner_pos.0 += widget.size.0 - widget.margin.0,
                    Direction::Vertical => self.inner_pos.1 += widget.size.1 - widget.margin.1,
                }
            }
        };
        // w.process(self.pos, ui.click_pos);
        self.contents.push(r);
        self.contents[self.contents.len() - 1].borrow()
    }
}

pub struct Widget {
    pub text: String,
    // pub color: Color,
    // pub bg: Option<Color>,
    pub padding: (u16, u16),
    pub margin: (u16, u16),
    // Including margin & padding!
    pub size: (u16, u16),
    pub pos: (u16, u16),
    clicked: bool,
}

impl Widget {
    pub fn new<S: Into<String>>(text: S) -> Self {
        Widget {
            text: text.into(),
            // color,
            // bg: None,
            pos: (0, 0),
            clicked: false,
            padding: (1, 1),
            size: (3, 3),
            margin: (1, 1),
        }
    }

    pub fn render(&self, stdout: &mut Stdout) {
        let fg = if self.clicked {
            Color::Red
        } else {
            Color::Black
        };

        // margin
        let mut row = self.pos.1;
        while row < self.size.1 + self.pos.1 {
            queue!(
                stdout,
                cursor::MoveTo(self.pos.0, row),
                Print("@".repeat(
                    (self.margin.0 + self.text.len() as u16 + 2 + self.padding.0) as usize
                ))
            )
            .unwrap();
            row += 1;
        }
        // padding
        let mut row = self.pos.1 + self.margin.1;
        while row < self.pos.1 + self.size.1 - self.margin.1 {
            queue!(
                stdout,
                cursor::MoveTo(self.pos.0 + self.margin.0, row),
                Print("$".repeat(
                    (self.padding.0 + self.text.len() as u16 + 2 - self.margin.0) as usize
                ))
            )
            .unwrap();
            row += 1;
        }

        queue!(
            stdout,
            FG_RESET,
            BG_RESET,
            cursor::MoveTo(
                self.pos.0 + self.margin.0 + self.padding.0,
                self.pos.1 + self.margin.1 + self.padding.1
            ),
            // Print(" ".repeat(self.margin.0.into())),
            SetBackgroundColor(Color::White),
            SetForegroundColor(fg),
            // Print(" ".repeat(self.padding.0.into())),
            Print(&self.text),
            // Print(format!(
            //     "{:?} {:?} {:?}",
            //     self.padding, self.margin, self.pos
            // )),
            // Print(" ".repeat(self.padding.0.into())),
            FG_RESET,
            BG_RESET,
            // Print(" ".repeat(self.margin.0.into())),
        )
        .unwrap();

        stdout.flush().unwrap();
    }

    pub fn calc_self(&mut self) -> (u16, u16) {
        // only supports 1-height text for now
        self.size.0 = self.margin.0 * 2 + self.padding.0 * 2 + self.text.len() as u16;
        self.size.1 = self.margin.1 * 2 + self.padding.1 * 2 + 1;
        self.size
    }

    pub fn process(&mut self, click_pos: Option<(u16, u16)>) {
        if let Some((click_col, click_row)) = click_pos {
            if click_col >= self.pos.0 + self.margin.0
                && click_row >= self.pos.1 + self.margin.1
                && click_col < self.pos.0 + self.size.0 - self.margin.0
                && click_row < self.pos.1 + self.size.1 - self.margin.1
            {
                self.clicked = true;
            };
        } else {
            self.clicked = false;
        }
    }
}
