use crossterm::{
    cursor,
    event::{Event, MouseButton},
    queue,
    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
    terminal::size,
};
use std::{
    cell::{Ref, RefCell},
    cmp,
    io::{Stdout, Write},
    ops::{Index, IndexMut},
    rc::Rc,
    slice::SliceIndex,
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
struct Pixel {
    char: char,
    color: Color,
    changed: bool,
}

impl Default for Pixel {
    fn default() -> Self {
        Self {
            char: ' ',
            color: Color::White,
            changed: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Context {
    pub max: (u16, u16),
    pub click_pos: Option<(u16, u16)>,
    // pub offset: u8,
    pub bg_color: Color,
    virtual_display: VirtualDisplay,
}

#[derive(Clone, Debug)]
struct VirtualDisplay(Vec<VirtualDisplayRow>);

#[derive(Clone, Debug)]
struct VirtualDisplayRow(Vec<Pixel>);

impl Index<u16> for VirtualDisplay {
    type Output = VirtualDisplayRow;

    fn index(&self, index: u16) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl IndexMut<u16> for VirtualDisplay {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

impl Index<u16> for VirtualDisplayRow {
    type Output = Pixel;

    fn index(&self, index: u16) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl IndexMut<u16> for VirtualDisplayRow {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

impl Context {
    pub fn new() -> Self {
        let (width, height) = size().unwrap();
        Context {
            bg_color: Color::Red,
            click_pos: None,
            max: (width, height),
            virtual_display: VirtualDisplay(vec![
                VirtualDisplayRow(vec![
                    Pixel::default();
                    width.into()
                ]);
                height.into()
            ]),
        }
    }

    pub fn set_size(&mut self, new_size: (u16, u16)) {
        self.max = new_size;
        self.virtual_display = VirtualDisplay(vec![
            VirtualDisplayRow(vec![
                Pixel::default();
                new_size.0.into()
            ]);
            new_size.1.into()
        ]);
    }

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
    pub fn render(&mut self, ctx: &mut Context) {
        self.root.calc_self();
        self.root.render(self.stdout, ctx);
        queue!(
            self.stdout,
            SetBackgroundColor(Color::Reset),
            SetForegroundColor(Color::Reset)
        )
        .unwrap();

        for (col_pos, column) in ctx.virtual_display.0.iter_mut().enumerate() {
            for (row_pos, px) in column.0.iter_mut().enumerate() {
                if !px.changed {
                    continue;
                }
                queue!(
                    self.stdout,
                    cursor::MoveTo(col_pos as u16, row_pos as u16),
                    SetForegroundColor(px.color),
                    crossterm::style::Print(px.char)
                )
                .unwrap();
                px.changed = false;
            }
        }
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
    // parent: Option<Rc<Block<'a>>>,
    pub pos: (u16, u16),
    pub size: (u16, u16),
    inner_pos: (u16, u16),
    available_margin: (u16, u16),
    pub direction: Direction,
}

pub enum Direction {
    Vertical,
    Horizontal,
}

impl<'a> Block<'a> {
    pub fn render(&mut self, stdout: &mut Stdout, ctx: &mut Context) {
        // let (max_width, max_height) = max;
        // Print the top border
        // queue!(stdout,);
        // queue!(
        //     stdout,
        //     cursor::MoveTo(self.pos.0, self.pos.1),
        //     Print("#".repeat((self.size.0).into()))
        // )
        // .unwrap();
        for row in self.pos.0..self.pos.0 + self.size.0 {
            for col in self.pos.1..self.pos.1 + self.size.1 {
                ctx.virtual_display[row][col] = Pixel {
                    char: '#',
                    color: Color::White,
                    changed: true,
                };
            }
        }

        for row in self.pos.0 + 1..self.pos.0 + self.size.0 - 1 {
            for col in self.pos.1 + 1..self.pos.1 + self.size.1 - 1 {
                ctx.virtual_display[row][col] = Pixel {
                    char: ' ',
                    color: Color::White,
                    changed: true,
                };
            }
        }
        // Print the bottom border
        // queue!(
        //     stdout,
        //     cursor::MoveTo(self.pos.0, self.pos.1 + self.size.1),
        //     Print("#".repeat((self.size.0).into()))
        // )
        // .unwrap();

        // queue!(stdout, cursor::MoveTo(self.pos.0 + 1, self.pos.1 + 1)).unwrap();
        let max = (
            self.size.0.checked_sub(2).unwrap_or(0),
            self.size.1.checked_sub(3).unwrap_or(0),
        );
        for el in self.contents.iter_mut() {
            match el.get_mut() {
                Element::Block(block) => {
                    block.render(stdout, ctx);
                }
                Element::Widget(widget) => {
                    widget.render(stdout);
                }
            }
        }
    }

    pub fn calc_self(&mut self) {
        self.size = (4, 4);
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

    pub fn calc_parent(&mut self, parent_pos: (u16, u16)) {
        self.pos.0 += parent_pos.0;
        self.pos.1 += parent_pos.1;
        println!("{:?}", self.inner_pos);
        self.calc_self();

        for el in self.contents.iter_mut() {
            match el.get_mut() {
                Element::Block(block) => {
                    block.calc_parent(self.inner_pos);
                }
                // TODO
                Element::Widget(_) => {
                    // widget.process(click_pos);
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
            // parent: None,
            pos,
            size: (3, 3),
            inner_pos: (2, 2),
            available_margin: (0, 0),
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
                block.pos = (block.pos.0 + self.pos.0, block.pos.1 + self.pos.1);
                block.calc_parent(self.inner_pos);
                // match self.direction {
                //     Direction::Horizontal => self.inner_pos.0 += block.size.0,
                //     Direction::Vertical => self.inner_pos.1 += block.size.1,
                // }
            }
            Element::Widget(ref mut widget) => {
                let adjusted = (
                    cmp::min(self.available_margin.0, widget.margin.left),
                    cmp::min(self.available_margin.1, widget.margin.top),
                );
                widget.pos = (
                    widget.pos.0 + self.pos.0 + self.inner_pos.0 - adjusted.0,
                    widget.pos.1 + self.pos.1 + self.inner_pos.1 - adjusted.1,
                );
                widget.calc_self();
                match self.direction {
                    Direction::Horizontal => {
                        self.inner_pos.0 += widget.size.0 - adjusted.0;
                        self.available_margin.0 = widget.margin.left;
                    }
                    Direction::Vertical => {
                        self.inner_pos.1 += widget.size.1 - adjusted.1;
                        self.available_margin.1 = widget.margin.top;
                    }
                }
            }
        };
        // w.process(self.pos, ui.click_pos);
        self.contents.push(r);
        self.contents[self.contents.len() - 1].borrow()
    }
}

pub struct Area {
    pub top: u16,
    pub right: u16,
    pub bottom: u16,
    pub left: u16,
    pub symbol: char,
    pub color: Color,
    pub background: Color,
}

pub enum AreaShort {
    Uniform(u16),
    HorVer(u16, u16),
    All(u16, u16, u16, u16),
}

impl Area {
    pub fn set(mut self, short: AreaShort) -> Self {
        (self.top, self.right, self.bottom, self.left) = match short {
            AreaShort::Uniform(uni) => (uni, uni, uni, uni),
            AreaShort::HorVer(hor, ver) => (hor, ver, hor, ver),
            AreaShort::All(t, r, b, l) => (t, r, b, l),
        };
        self
    }

    pub fn symbol(mut self, symbol: char) -> Self {
        self.symbol = symbol;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

impl Default for Area {
    fn default() -> Self {
        Self {
            top: 1,
            right: 1,
            bottom: 1,
            left: 1,
            symbol: ' ',
            color: Color::White,
            background: Color::Black,
        }
    }
}

pub struct Widget {
    pub text: String,
    // pub color: Color,
    // pub bg: Option<Color>,
    pub padding: Area,
    pub margin: Area,
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
            padding: Area::default().symbol('$'),
            size: (3, 3),
            margin: Area::default().symbol('#'),
        }
    }

    pub fn padding<F>(mut self, f: F) -> Self
    where
        F: FnOnce(Area) -> Area,
    {
        self.padding = f(self.padding);
        self
    }

    pub fn margin<F>(mut self, f: F) -> Self
    where
        F: FnOnce(Area) -> Area,
    {
        self.margin = f(self.margin);
        self
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
                SetForegroundColor(self.margin.color),
                // TODO: make symbol a string and only accept char to set symbol?
                Print(self.margin.symbol.to_string().repeat(
                    (self.margin.left
                        + self.margin.right
                        + self.padding.left
                        + self.padding.right
                        + self.text.len() as u16) as usize
                ))
            )
            .unwrap();
            row += 1;
        }
        // padding
        let mut row = self.pos.1 + self.margin.top;
        while row < self.pos.1 + self.size.1 - self.margin.top {
            queue!(
                stdout,
                cursor::MoveTo(self.pos.0 + self.margin.left, row),
                SetForegroundColor(self.padding.color),
                Print(self.padding.symbol.to_string().repeat(
                    (self.padding.left + self.padding.right + self.text.len() as u16) as usize
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
                self.pos.0 + self.margin.left + self.padding.left,
                self.pos.1 + self.margin.top + self.padding.top
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
        self.size.0 = self.margin.left
            + self.margin.right
            + self.padding.left
            + self.padding.right
            + self.text.len() as u16;
        self.size.1 = self.margin.top * 2 + self.padding.top * 2 + 1;
        self.size
    }

    pub fn process(&mut self, click_pos: Option<(u16, u16)>) {
        if let Some((click_col, click_row)) = click_pos {
            if click_col >= self.pos.0 + self.margin.left
                && click_row >= self.pos.1 + self.margin.top
                && click_col < self.pos.0 + self.size.0 - self.margin.right
                && click_row < self.pos.1 + self.size.1 - self.margin.bottom
            {
                self.clicked = true;
            };
        } else {
            self.clicked = false;
        }
    }
}
