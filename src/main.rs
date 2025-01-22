use iced::{
    color,
    widget::{button, column, row, Button, Column, Row, Svg},
    Border, Color, Element, Length, Theme,
};
use rand::Rng;

fn main() {
    let _ = iced::application(Conway::title, Conway::update, Conway::view).run();
}

#[derive(Clone, Copy, Debug)]
enum Message {
    Update,
    PlayPause,
    Stop,
}

#[derive(Clone, Copy)]
struct Cell {
    living: bool,
}

struct Conway {
    cells_tab: [[Cell; Self::SIZE]; Self::SIZE],
    playing: bool,
}

impl Conway {
    const SIZE: usize = 20;
    const LIVING_DENSITY: f64 = 0.25;

    fn check_neighbours(&self, x: usize, y: usize) -> usize {
        let mut living_neighbours = 0;

        for dx in [-1, 0, 1] {
            for dy in [-1, 0, 1] {
                if dx == 0 && dy == 0 {
                    continue;
                }

                let nx = x.wrapping_add(dx as usize);
                let ny = y.wrapping_add(dy as usize);

                if nx < Self::SIZE && ny < Self::SIZE && self.cells_tab[nx][ny].living {
                    living_neighbours += 1;
                }
            }
        }

        living_neighbours
    }

    fn default_button() -> Button<'static, Message> {
        Button::new("").height(Length::Fill).width(Length::Fill)
    }

    fn title(&self) -> String {
        "Conway".into()
    }

    fn view(&self) -> Element<Message> {
        let mut column_conway = Column::new();
        for y in 0..Self::SIZE {
            let mut row = Row::new();
            for x in 0..Self::SIZE {
                let living = self.cells_tab[x][y].living;
                row = row.push(
                    Self::default_button().style(move |_theme: &Theme, _status| button::Style {
                        background: Some(if living {
                            Color::BLACK.into()
                        } else {
                            Color::WHITE.into()
                        }),
                        border: Border {
                            color: color!(0xBFBFBF),
                            width: 1.0,
                            ..Border::default()
                        },
                        ..button::Style::default()
                    }),
                );
            }
            column_conway = column_conway.push(row);
        }
        let control_row = row![
            button(Svg::<Theme>::from_path(if self.playing {
                "../images/play.svg"
            } else {
                "../images/pause.svg"
            }))
            .on_press(Message::PlayPause),
            button(Svg::from_path("../images/stop.svg")).on_press(Message::Stop),
            button("Update").on_press(Message::Update)
        ];
        column![column_conway, control_row].into()
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Update => {
                for x in 0..Self::SIZE {
                    for y in 0..Self::SIZE {
                        if !(self.cells_tab[x][y].living) && self.check_neighbours(x, y) == 3 {
                            self.cells_tab[x][y].living = true;
                        }
                        if self.cells_tab[x][y].living
                            && !(self.check_neighbours(x, y) == 2
                                || self.check_neighbours(x, y) == 3)
                        {
                            self.cells_tab[x][y].living = false;
                        }
                    }
                }
            }
            Message::Stop => self.playing = false,
            Message::PlayPause => self.playing = !self.playing,
        }
    }
}

impl Default for Conway {
    fn default() -> Self {
        let mut rng = rand::thread_rng();
        let mut cells_tab = [[Cell { living: false }; Self::SIZE]; Self::SIZE];

        for x in 0..Self::SIZE {
            for y in 0..Self::SIZE {
                cells_tab[x][y] = Cell {
                    living: rng.gen_bool(Self::LIVING_DENSITY),
                }
            }
        }

        Self {
            cells_tab,
            playing: false,
        }
    }
}
