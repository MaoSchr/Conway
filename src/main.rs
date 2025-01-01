use iced::{
    widget::{button, Column, Row},
    Color, Element, Theme,
};

fn main() {
    let _ = iced::application(Conway::title, Conway::update, Conway::view).run();
}

#[derive(Clone, Copy, Debug)]
enum Message {}

#[derive(Clone, Copy)]
struct Cell {
    living: bool,
    x: usize,
    y: usize,
}

struct Conway {
    cells_tab: [[Cell; Self::SIZE]; Self::SIZE],
}

impl Conway {
    const SIZE: usize = 10;

    fn title(&self) -> String {
        "Conway".into()
    }

    fn view(&self) -> Element<Message> {
        let mut column = Column::new();
        for y in 0..Self::SIZE {
            let mut row = Row::new();
            for x in 0..Self::SIZE {
                row = row.push(button("").style(move |_theme: &Theme, _status| {
                    if self.cells_tab[x][y].living {
                        button::Style::default().with_background(Color::BLACK)
                    } else {
                        button::Style::default().with_background(Color::WHITE)
                    }
                }));
            }
            column = column.push(row);
        }
        column.into()
    }

    fn update(&mut self, message: Message) {}
}

impl Default for Conway {
    fn default() -> Self {
        let mut cells_tab = [[Cell {
            living: false,
            x: 0,
            y: 0,
        }; Self::SIZE]; Self::SIZE];

        for x in 0..Self::SIZE {
            for y in 0..Self::SIZE {
                cells_tab[x][y] = Cell {
                    living: rand::random(),
                    x,
                    y,
                }
            }
        }

        Self { cells_tab }
    }
}
