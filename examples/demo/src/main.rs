use bad_tui::ui::{Block, Context, Element, Widget, UI};
use crossterm::{
    cursor::{self, position, MoveTo},
    event::{poll, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute, queue,
    style::Color,
    terminal::{self, disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use std::{
    io::{self, stdout, Stdout},
    time::Duration,
};

fn print_events(stdout: &mut Stdout) -> io::Result<()> {
    let mut ui = UI {
        stdout,
        root: Block::new((0, 0)),
    };

    let mut ctx = Context::new();

    // ui.root.push(Element::Widget(
    //     Widget::new("I'm a widget!")
    //         .padding(|pad| {
    //             pad.color(Color::Cyan)
    //                 .set(bad_tui::ui::AreaShort::Uniform(1))
    //         })
    //         .margin(|mar| mar.color(Color::Blue).symbol('!')),
    // ));

    // ui.root
    //     .push(Element::Widget(Widget::new("I'm a second widget!").margin(
    //         |mar| {
    //             mar.color(Color::Green)
    //                 .set(bad_tui::ui::AreaShort::Uniform(2))
    //                 .symbol('@')
    //         },
    //     )));

    // ui.root
    //     .push(Element::Widget(Widget::new("I'm a third widget!").margin(
    //         |mar| {
    //             mar.color(Color::Yellow)
    //                 .set(bad_tui::ui::AreaShort::HorVer(3, 4))
    //                 .symbol('#')
    //         },
    //     )));
    // ui.root
    //     .push(Element::Widget(Widget::new("I'm a second widget!").margin(
    //         |mar| {
    //             mar.color(Color::Green)
    //                 .set(bad_tui::ui::AreaShort::Uniform(2))
    //                 .symbol('@')
    //         },
    //     )));
    // ui.root.push(Element::Widget(
    //     Widget::new("I'm a fourth widget!")
    //         .padding(|pad| {
    //             pad.color(Color::Cyan)
    //                 .set(bad_tui::ui::AreaShort::Uniform(1))
    //         })
    //         .margin(|mar| mar.color(Color::Blue).symbol('!')),
    // ));
    // ui.root.push(Element::Widget(
    //     Widget::new("I'm a widget!")
    //         .padding(|pad| {
    //             pad.color(Color::Cyan)
    //                 .set(bad_tui::ui::AreaShort::Uniform(1))
    //         })
    //         .margin(|mar| mar.color(Color::Blue).symbol('!')),
    // ));
    let mut b = Block::new((0, 0));
    let mut b_third = Block::new((0, 0));
    // b_third.
    b_third.push(Element::Block(Block::new((0, 0))));
    b.push(Element::Block(b_third));
    // b.push(Element::Widget(
    //     Widget::new("I'm a widget!")
    //         .padding(|pad| {
    //             pad.color(Color::Cyan)
    //                 .set(bad_tui::ui::AreaShort::Uniform(1))
    //         })
    //         .margin(|mar| mar.color(Color::Blue).symbol('!')),
    // ));
    // b.push(Element::Block(Block::new((0, 0))));
    ui.root.push(Element::Block(b));
    // let mut b = Block::new((0, 0));
    // b.push(Element::Widget(
    //     Widget::new("I'm a widget!")
    //         .padding(|pad| {
    //             pad.color(Color::Cyan)
    //                 .set(bad_tui::ui::AreaShort::Uniform(1))
    //         })
    //         .margin(|mar| mar.color(Color::Blue).symbol('!')),
    // ));
    // b.push(Element::Block(Block::new((0, 0))));
    // ui.root.push(Element::Block(b));

    // ui.root
    //     .push(Element::Widget(Widget::new("I'm a third widget!")));

    ui.render(&mut ctx);

    loop {
        // Blocking read
        let event = read()?;
        ctx.process(&event);
        ui.process(&ctx);
        ui.render(&mut ctx);
        // println!("Event: {:?}\r", event);

        // if event == Event::Key(KeyCode::Char('c').into()) {
        //     println!("Cursor position: {:?}\r", position());
        // }

        // TODO: add to ctx.process
        if let Event::Resize(x, y) = event {
            let (_original_size, new_size) = flush_resize_events((x, y));
            ctx.max = new_size;
            // println!("Resize from: {:?}, to: {:?}\r", original_size, new_size);
        }

        if event == Event::Key(KeyCode::Esc.into()) {
            break;
        }
    }

    // if poll(Duration::from_millis(100))? {
    //             // It's guaranteed that `read` won't block, because `poll` returned
    //             // `Ok(true)`.
    //             println!("{:?}", read()?);
    //         } else {
    //             // Timeout expired, no `Event` is available
    //         }

    Ok(())
}

fn flush_resize_events(first_resize: (u16, u16)) -> ((u16, u16), (u16, u16)) {
    let mut last_resize = first_resize;
    while let Ok(true) = poll(Duration::from_millis(50)) {
        if let Ok(Event::Resize(x, y)) = read() {
            last_resize = (x, y);
        }
    }

    (first_resize, last_resize)
}

fn main() {
    enable_raw_mode().unwrap();

    let mut stdout = stdout();
    execute!(
        stdout,
        Clear(ClearType::All),
        EnableMouseCapture,
        cursor::EnableBlinking,
        cursor::SetCursorStyle::BlinkingBar,
        cursor::Hide
    )
    .unwrap();

    if let Err(e) = print_events(&mut stdout) {
        println!("Error: {:?}\r", e);
    }

    execute!(stdout, DisableMouseCapture).unwrap();

    disable_raw_mode().unwrap();
}
