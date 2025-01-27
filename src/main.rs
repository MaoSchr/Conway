use iced::{
    color, time,
    widget::{button, column, container, radio, row, slider, text, Button, Column, Row, Svg},
    Border, Color, Element, Length, Subscription, Theme,
};
use rand::Rng;

fn main() {
    let _ = iced::application(Conway::title, Conway::update, Conway::view)
        .subscription(Conway::subscription)
        .run();
}

#[derive(Copy, Clone, Debug)]
enum Message {
    Update,
    PlayPause,
    Stop,
    Simulation,
    Settings,
    Réinitialiser,
    ActiverDésactiver(usize, usize),
    FillingMethodChanged(FillingMethod),
    Generationchange(u32),
    InitCellsNumber(u32),
    InitGenNumber(u32),
    InitDensity(u32),
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
    nb_init_cells: u32,
    nb_max_generation: u32,
    cells_tab: [[Cell; Self::SIZE]; Self::SIZE],
    playing: bool,
    generation: u32,
    screen: Screen,
    filling_method: FillingMethod,
    living_density: u32,
    number_of_living_cells: u32,
    initial_tab: [[Cell; Self::SIZE]; Self::SIZE],
}
impl Conway {
    const SIZE: usize = 50;

    fn subscription(&self) -> Subscription<Message> {
        let sub = time::every(time::Duration::from_millis(500));
        if self.playing {
            sub.map(|_| Message::Update)
        } else {
            Subscription::none()
        }
    }
    fn check_neighbours(&self, x: usize, y: usize) -> usize {
        let mut living_neighbours = 0;
        let _size = Self::SIZE as isize;
        for dx in -1isize..=1isize {
            for dy in -1isize..=1isize {
                if dx == 0 && dy == 0 {
                    continue;
                }

                let nx = ((x as isize) + dx).rem_euclid(Self::SIZE as isize) as usize;
                let ny = ((y as isize) + dy).rem_euclid(Self::SIZE as isize) as usize;

                if self.cells_tab[nx as usize][ny as usize].living {
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
                    living: rng.gen_bool(self.living_density as f64 / 100.0),
                };
                if cells_tab[x][y].living {
                    count_cells += 1;
                }
            }
        }
        self.nb_init_cells = count_cells;
        self.number_of_living_cells = count_cells;
        self.cells_tab = cells_tab;
        self.initial_tab = cells_tab;
    }

    fn build_cells_with_number_of_cells(&mut self) {
        let mut count_cells = 0;
        let mut rng = rand::thread_rng();
        for x in 0..Self::SIZE {
            for y in 0..Self::SIZE {
                self.cells_tab[x][y].living = false;
            }
        }
        while count_cells < self.nb_init_cells {
            let x = rng.gen_range(0..Self::SIZE);
            let y = rng.gen_range(0..Self::SIZE);
            if !self.cells_tab[x][y].living {
                self.cells_tab[x][y].living = true;
                count_cells += 1;
            }
        }
        self.initial_tab = self.cells_tab;
    }

    fn réinitialiser(&mut self) {
        self.cells_tab = self.initial_tab;
        self.playing = true;
        self.generation = 1;
        self.screen = Screen::Simul;
        self.number_of_living_cells = self.nb_init_cells;
    }

    fn default_button(x: usize, y: usize) -> Button<'static, Message> {
        Button::new("")
            .height(Length::Fill)
            .width(Length::Fill)
            .on_press(Message::ActiverDésactiver(x, y))
    }

    fn title(&self) -> String {
        match self.screen {
            Screen::Init => "Jeu de Conway - Paramètres".into(),
            Screen::Simul => "Jeu de Conway - Simulation".into(),
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
            slider(0..=1000, self.nb_max_generation, Message::InitGenNumber,),
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
                    slider(1..=99, self.living_density, Message::InitDensity),
                    text(format!("{}%", self.living_density))
                ];
                init = init.push(fillingmethod_choice_row);
                init = init.push(button("Simulation").on_press(Message::Simulation));
                return init.into();
            }
            FillingMethod::NumberOfCells => {
                let fillingmethod_choice_row = row![
                    slider(100..=2000, self.nb_init_cells, Message::InitCellsNumber),
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
                    Self::default_button(x, y).style(move |_theme: &Theme, _status| {
                        button::Style {
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
                        }
                    }),
                );
            }
            column_conway = column_conway.push(row);
        }
        let control_row = row![
            button("Update").on_press(Message::Update),
            button(Svg::from_path("images/pause.svg")).on_press(Message::PlayPause),
            button(Svg::from_path("images/stop.svg")).on_press(Message::Stop),
            text(self.nb_max_generation.to_string()),
            button("Paramètres").on_press(Message::Settings),
            button("Réinitialiser").on_press(Message::Réinitialiser)
        ];
        let info_row = row![
            text("1".to_string()),
            slider(
                1..=self.nb_max_generation,
                self.generation,
                Message::Generationchange,
            ),
            text("Génération:"),
            text(self.generation.to_string()),
            text("\t"),
            text("Cellules vivantes:"),
            text(self.number_of_living_cells.to_string())
        ];
        column![column_conway, control_row, info_row].into()
    }

    fn update_cells(&mut self) {
        let mut next_cells_tab = self.cells_tab;

        for x in 0..Self::SIZE {
            for y in 0..Self::SIZE {
                let living_neighbours = self.check_neighbours(x, y);

                next_cells_tab[x][y].living = match (self.cells_tab[x][y].living, living_neighbours)
                {
                    (true, 2) | (true, 3) => true, // Reste en vie si 2 ou 3 voisins vivants
                    (false, 3) => {
                        self.number_of_living_cells += 1;
                        true
                    } // Devient vivant si exactement 3 voisins vivants
                    (true, _) => {
                        self.number_of_living_cells -= 1;
                        false
                    }
                    _ => false, // Sinon, reste ou devient mort
                };
            }
        }
        self.cells_tab = next_cells_tab;
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Update => {
                if self.generation < self.nb_max_generation {
                    Self::update_cells(self);
                    self.generation += 1;
                }
            }
            Message::Stop => self.playing = false,
            Message::PlayPause => self.playing = !self.playing,
            Message::Generationchange(value) => {
                if value >= self.generation && self.generation <= self.nb_max_generation {
                    self.generation = value;
                    Self::update_cells(self);
                }
            }
            Message::InitCellsNumber(value) => {
                self.nb_init_cells = value;
                self.number_of_living_cells = value
            }
            Message::InitDensity(value) => self.living_density = value,
            Message::InitGenNumber(value) => {
                self.nb_max_generation = value;
                self.generation = 1;
            }
            Message::FillingMethodChanged(method) => self.filling_method = method,
            Message::Simulation => {
                match self.filling_method {
                    FillingMethod::Density => Self::build_cells_with_density(self),
                    FillingMethod::NumberOfCells => Self::build_cells_with_number_of_cells(self),
                };
                self.screen = Screen::Simul;
            }
            Message::Settings => self.screen = Screen::Init,
            Message::Réinitialiser => Self::réinitialiser(self),
            Message::ActiverDésactiver(x, y) => {
                if self.cells_tab[x][y].living {
                    self.number_of_living_cells -= 1;
                    self.cells_tab[x][y].living = false;
                } else {
                    self.number_of_living_cells += 1;
                    self.cells_tab[x][y].living = true;
                }
            }
        }
    }
}

impl Default for Conway {
    fn default() -> Self {
        let mut count_cells = 0;
        let density = 25;
        let mut rng = rand::thread_rng();
        let mut cells_tab = [[Cell { living: false }; Self::SIZE]; Self::SIZE];

        for x in 0..Self::SIZE {
            for y in 0..Self::SIZE {
                cells_tab[x][y] = Cell {
                    living: rng.gen_bool(density as f64 / 100.0),
                };
                if cells_tab[x][y].living {
                    count_cells += 1;
                }
            }
        }

        Self {
            cells_tab,
            playing: false,
            generation: 1,
            screen: Screen::Init,
            nb_init_cells: count_cells,
            nb_max_generation: 100,
            living_density: 25,
            filling_method: FillingMethod::Density,
            number_of_living_cells: count_cells,
            initial_tab: cells_tab,
        }
    }
}
