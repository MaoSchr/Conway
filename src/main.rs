use iced::{
    color,
    widget::{button, column, container, radio, row, slider, text, Button, Column, Row, Svg},
    Background, Border, Color, Element, Length, Theme,
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
    Simulation,
    Settings,
    FillingMethodChanged(FillingMethod),
    Generationchange(u16),
    InitCellsNumber(u16),
    InitGenNumber(u16),
    InitDensity(f64),
}

#[derive(Clone, Copy, Debug)]
enum Screen {
    Init,
    Simul,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
enum FillingMethod {
    Density,
    NumberOfCells,
}

#[derive(Clone, Copy)]
struct Cell {
    living: bool,
}

struct Conway {
    nb_init_cells: u16,
    nb_max_generation: u16,
    cells_tab: [[Cell; Self::SIZE]; Self::SIZE],
    playing: bool,
    generation: u16,
    screen: Screen,
    filling_method: FillingMethod,
    living_density: f64,
}

impl Conway {
    const SIZE: usize = 50;

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

    fn build_cells_with_density(&mut self) {
        let mut count_cells = 0;
        let mut rng = rand::thread_rng();
        let mut cells_tab = [[Cell { living: false }; Self::SIZE]; Self::SIZE];

        for x in 0..Self::SIZE {
            for y in 0..Self::SIZE {
                cells_tab[x][y] = Cell {
                    living: rng.gen_bool(self.living_density),
                };
                count_cells += 1;
            }
        }
        self.nb_init_cells = count_cells;
        self.cells_tab = cells_tab;
    }

    fn build_cells_with_number_of_cells(&mut self) {
        let mut count_cells = 0;
        let mut rng = rand::thread_rng();

        while count_cells < self.nb_init_cells {
            let x = rng.gen_range(0..Self::SIZE);
            let y = rng.gen_range(0..Self::SIZE);
            if !self.cells_tab[x][y].living {
                self.cells_tab[x][y].living = true;
                count_cells += 1;
            }
        }
    }

    fn default_button() -> Button<'static, Message> {
        Button::new("").height(Length::Fill).width(Length::Fill)
    }

    fn title(&self) -> String {
        match self.screen {
            Screen::Init => "Conway initialisation".into(),
            Screen::Simul => "Conway simulation".into(),
        }
    }

    fn view(&self) -> Element<Message> {
        let screen = match self.screen {
            Screen::Init => self.init(),
            Screen::Simul => self.simulation(),
        };
        container(screen).into()
    }

    fn init(&self) -> Element<Message> {
        let mut init = Column::new();
        let choose_nb_gen_row = row![
            slider(0..=255, self.nb_max_generation, Message::InitGenNumber,),
            text(self.nb_max_generation.to_string())
        ];

        init = init.push(text("Nombre de générations"));
        init = init.push(choose_nb_gen_row);

        let density_radio = radio(
            "Density method",
            FillingMethod::Density,
            Some(self.filling_method),
            Message::FillingMethodChanged,
        );

        let nb_cells_radio = radio(
            "Choose directly the number of cells",
            FillingMethod::NumberOfCells,
            Some(self.filling_method),
            Message::FillingMethodChanged,
        );

        let fillingmethod_row = row![density_radio, nb_cells_radio];

        init = init.push(fillingmethod_row);
        match self.filling_method {
            FillingMethod::Density => {
                let fillingmethod_choice_row = row![
                    slider(0.01..=0.99, self.living_density, Message::InitDensity),
                    text(self.living_density.to_string())
                ];
                init = init.push(fillingmethod_choice_row);
                init = init.push(button("Simulation").on_press(Message::Simulation));
                return init.into();
            }
            FillingMethod::NumberOfCells => {
                let fillingmethod_choice_row = row![
                    slider(1..=100, self.nb_init_cells, Message::InitCellsNumber),
                    text(self.nb_init_cells.to_string())
                ];
                init = init.push(fillingmethod_choice_row);
                init = init.push(button("Simulation").on_press(Message::Simulation));
                return init.into();
            }
        }
    }

    fn simulation(&self) -> Element<Message> {
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
            button("Update").on_press(Message::Update),
            button("Paramètres").on_press(Message::Settings),
            //Svg::from_path("../images/pause.png"),
            slider(
                1..=self.nb_max_generation,
                self.generation,
                Message::Generationchange
            ),
            text(self.generation.to_string())
        ];
        column![column_conway, control_row].into()
    }

    fn update_cells(&mut self) {
        for x in 0..Self::SIZE {
            for y in 0..Self::SIZE {
                if !(self.cells_tab[x][y].living) && self.check_neighbours(x, y) == 3 {
                    self.cells_tab[x][y].living = true;
                }
                if self.cells_tab[x][y].living
                    && !(self.check_neighbours(x, y) == 2 || self.check_neighbours(x, y) == 3)
                {
                    self.cells_tab[x][y].living = false;
                }
            }
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Update => {
                Self::update_cells(self);
                self.generation += 1;
            }
            Message::Stop => self.playing = false,
            Message::PlayPause => self.playing = !self.playing,
            Message::Generationchange(value) => {
                if value >= self.generation {
                    Self::update_cells(self);
                    self.generation = value;
                }
            }
            Message::InitCellsNumber(value) => self.nb_init_cells = value,
            Message::InitDensity(value) => self.living_density = value,
            Message::InitGenNumber(value) => self.nb_max_generation = value,
            Message::FillingMethodChanged(method) => self.filling_method = method,
            Message::Simulation => {
                match self.filling_method {
                    FillingMethod::Density => Self::build_cells_with_density(self),
                    FillingMethod::NumberOfCells => Self::build_cells_with_number_of_cells(self),
                };
                self.screen = Screen::Simul;
            }
            Message::Settings => self.screen = Screen::Init,
        };
    }
}

impl Default for Conway {
    fn default() -> Self {
        let mut count_cells: u16 = 0;
        let density = 0.25;
        let mut rng = rand::thread_rng();
        let mut cells_tab = [[Cell { living: false }; Self::SIZE]; Self::SIZE];

        for x in 0..Self::SIZE {
            for y in 0..Self::SIZE {
                cells_tab[x][y] = Cell {
                    living: rng.gen_bool(density),
                };
                count_cells += 1;
            }
        }

        Self {
            cells_tab,
            playing: false,
            generation: 0,
            screen: Screen::Init,
            nb_init_cells: count_cells,
            nb_max_generation: 100,
            living_density: 0.25,
            filling_method: FillingMethod::Density,
        }
    }
}
