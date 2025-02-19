use iced::{
    color, time,
    widget::{button, column, container, row, text, text_input, Button, Column, Row, Svg, Text},
    Border, Color, Element, Length, Subscription, Theme,
};
use rand::Rng;

fn main() {
    let _ = iced::application(Conway::title, Conway::update, Conway::view)
        .subscription(Conway::subscription)
        .run();
}

#[derive(Clone, Debug)]
enum Message {
    Update,
    PlayPause,
    Simulation,
    Settings,
    Examples,
    Réinitialiser,
    Grid,
    IncreaseVitesse,
    DecreaseVitesse,
    IncreaseQuickVitesse,
    DecreaseQuickVitesse,
    ActiverDésactiver(usize, usize),
    FillingMethodChanged,
    InitCellsNumber(u32),
    InitDensity(u32),
    ChangeVitesse(u32),
    InputChangeMethod(String),
    InputVitesse(String),
    ConvertVitesse,
    ConvertDensity,
    ConvertCells,
}

#[derive(Clone, Copy, Debug)]
enum Screen {
    Init,
    Simul,
    Example,
    Conway,
}

#[derive(Clone, Copy)]
struct Cell {
    living: bool,
}

struct Conway {
    nb_init_cells: u32,
    cells_tab: [[Cell; Self::SIZE]; Self::SIZE],
    playing: bool,
    generation: u32,
    screen: Screen,
    filling_method: bool,
    living_density: u32,
    number_of_living_cells: u32,
    initial_tab: [[Cell; Self::SIZE]; Self::SIZE],
    vitesse: u32,
    grid_state: bool,
    input_v: String,
    input_c: String,
    erreur_v: bool,
    erreur_c: bool,
}

impl Conway {
    const SIZE: usize = 50;
    fn subscription(&self) -> Subscription<Message> {
        if self.playing {
            time::every(time::Duration::from_millis(self.vitesse as u64).into())
                .map(|_| Message::Update)
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
            Screen::Example => "Jeu de Conway - Exemples".into(),
            Screen::Conway => "Conway".into(),
        }
    }

    fn view(&self) -> Element<Message> {
        let screen = match self.screen {
            Screen::Init => self.init(),
            Screen::Simul => self.simulation(),
            Screen::Example => self.examples(),
            Screen::Conway => self.conway(),
        };
        container(screen).into()
    }

    fn examples(&self) -> Element<Message> {
        column![
            button("Simulation").on_press(Message::Simulation),
            text("En cours...")
        ]
        .into()
    }

    fn conway(&self) -> Element<Message> {
        container(column![
            container(text("Jeu de Conway").size(50)).center(Length::Fill),
            container(
                row![
                    button("Paramètres").on_press(Message::Settings),
                    button("Bac à sable").on_press(Message::Simulation)
                ]
                .spacing(50)
            )
            .center(Length::Fill)
        ])
        .center(Length::Fill)
        .into()
    }

    fn init(&self) -> Element<Message> {
        let mut init = Column::new();
        let mut density_button = button("Density Method").on_press(Message::FillingMethodChanged);
        let mut nb_cells_button =
            button("Number of cells Method").on_press(Message::FillingMethodChanged);
        if self.filling_method {
            density_button = density_button.style(button::primary);
            nb_cells_button = nb_cells_button.style(button::secondary);
        } else {
            density_button = density_button.style(button::secondary);
            nb_cells_button = nb_cells_button.style(button::primary)
        }
        let fillingmethod_row = column![
            text("Construction du tableau initial").size(35),
            row![density_button, nb_cells_button].spacing(50)
        ];

        init = init.push(fillingmethod_row);

        init = init.push(Row::new());

        match self.filling_method {
            true => {
                let vitesse_row = row![
                    text("1<").size(20),
                    text_input("Choose the vitesse!", &self.input_v.as_str())
                        .on_input(Message::InputVitesse)
                        .size(20),
                    Button::new("OK").on_press(Message::ConvertVitesse),
                    text("<500").size(20),
                ];

                let fillingmethod_choice_row = row![
                    text("1%<").size(20),
                    text_input("Choose the density of cells!", &self.input_c.as_str())
                        .on_input(Message::InputChangeMethod),
                    Button::new("OK").on_press(Message::ConvertDensity),
                    text("<100%").size(20)
                ];
                init = init.push(vitesse_row);
                init = init.push(text("Don\'t worry, you can change it later.").size(15));
                init = init.push(fillingmethod_choice_row);
                if self.erreur_v == false {
                    init = init.push(row![text(format!("Vitesse validée: {0}", self.vitesse))]);
                }
                if self.erreur_c == false {
                    init = init.push(row![text(format!(
                        "Densité de cellules initiales: {0}",
                        self.living_density
                    ))])
                };
                init = init.push(button("Simulation").on_press(Message::Simulation));
                return init.into();
            }
            false => {
                let vitesse_row = row![
                    text("1<").size(20),
                    text_input("Choose the vitesse!", &self.input_v.as_str())
                        .on_input(Message::InputVitesse)
                        .size(20),
                    Button::new("OK").on_press(Message::ConvertVitesse),
                    text("<500").size(20),
                ];

                let fillingmethod_choice_row = row![
                    text("1<").size(20),
                    text_input("Choose the number of cells!", &self.input_c.as_str())
                        .on_input(Message::InputChangeMethod),
                    Button::new("OK").on_press(Message::ConvertCells),
                    text("<5000").size(20)
                ];
                init = init.push(vitesse_row);
                init = init.push(text("Don\'t worry, you can change it later.").size(15));
                init = init.push(fillingmethod_choice_row);
                if self.erreur_v == false {
                    init = init.push(row![text(format!("Vitesse validée: {0}", self.vitesse))]);
                } else {
                    init = init.push(row![text("Rentrez un nombre valide!")]);
                }
                if self.erreur_c == false {
                    init = init.push(row![text(format!(
                        "Nombres de cellules initiales: {0}",
                        self.number_of_living_cells
                    ))]);
                } else {
                    init = init.push(row![text("Rentrez un nombre valide!")]);
                }
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
                let grid_state = self.grid_state;
                row = row.push(
                    Self::default_button(x, y).style(move |_theme: &Theme, _status| {
                        button::Style {
                            background: Some(if living {
                                Color::BLACK.into()
                            } else {
                                Color::WHITE.into()
                            }),
                            border: Border {
                                color: Some(if grid_state {
                                    color!(0xBFBFBF)
                                } else {
                                    Color::WHITE.into()
                                })
                                .unwrap(),
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
        let lecture_buttons = row![
            button("Update").on_press(Message::Update),
            if self.playing {
                button(Svg::from_path("images/pause.svg"))
                    .height(35)
                    .width(35)
                    .on_press(Message::PlayPause)
                    .style(button::secondary)
            } else {
                button(Svg::from_path("images/play.svg"))
                    .height(35)
                    .width(35)
                    .on_press(Message::PlayPause)
                    .style(button::secondary)
            },
            button(Svg::from_path("images/stop.svg"))
                .height(35)
                .width(35)
                .on_press(Message::Réinitialiser)
                .style(button::secondary),
        ];

        let settings_buttons = row![
            button("Grille")
                .on_press(Message::Grid)
                .style(button::secondary),
            button("Paramètres").on_press(Message::Settings),
            button("Exemples").on_press(Message::Examples),
        ];

        let vitesse_buttons = row![
            button(">>")
                .on_press(Message::IncreaseQuickVitesse)
                .style(button::secondary),
            button(">")
                .on_press(Message::IncreaseVitesse)
                .style(button::secondary),
            button("<")
                .on_press(Message::DecreaseVitesse)
                .style(button::secondary),
            button("<<")
                .on_press(Message::DecreaseQuickVitesse)
                .style(button::secondary),
        ];

        let info_row = row![
            text("Génération:").size(20),
            text(self.generation.to_string()).size(21),
            text("\t"),
            text("Cellules vivantes:").size(20),
            text(self.number_of_living_cells.to_string()).size(21),
            text("\t"),
            text("Vitesse:").size(21),
            text(self.vitesse.to_string()).size(20),
        ];

        let control_row = row![lecture_buttons, vitesse_buttons, settings_buttons].spacing(242);
        column![column_conway, control_row, info_row,].into()
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
                Self::update_cells(self);
                self.generation += 1;
            }
            Message::PlayPause => self.playing = !self.playing,
            Message::InitCellsNumber(value) => {
                self.nb_init_cells = value;
                self.number_of_living_cells = value
            }
            Message::InitDensity(value) => self.living_density = value,
            Message::FillingMethodChanged => {
                self.filling_method = !self.filling_method;
            }
            Message::Simulation => match self.screen {
                Screen::Conway => {
                    let cells_tab = [[Cell { living: false }; Self::SIZE]; Self::SIZE];
                    *self = Conway {
                        cells_tab,
                        playing: false,
                        generation: 1,
                        screen: Screen::Simul,
                        nb_init_cells: 0,
                        living_density: 0,
                        filling_method: true,
                        number_of_living_cells: 0,
                        initial_tab: cells_tab,
                        vitesse: 100,
                        grid_state: true,
                        input_c: "".to_string(),
                        input_v: "".to_string(),
                        erreur_c: true,
                        erreur_v: true,
                    }
                }
                Screen::Init => {
                    match self.filling_method {
                        true => Self::build_cells_with_density(self),
                        false => Self::build_cells_with_number_of_cells(self),
                    };
                    self.screen = Screen::Simul;
                }
                _ => (),
            },
            Message::Settings => self.screen = Screen::Init,
            Message::Réinitialiser => {
                Self::réinitialiser(self);
                self.playing = false;
                self.generation = 1;
            }

            Message::ActiverDésactiver(x, y) => {
                if self.cells_tab[x][y].living {
                    self.number_of_living_cells -= 1;
                    self.cells_tab[x][y].living = false;
                } else {
                    self.number_of_living_cells += 1;
                    self.cells_tab[x][y].living = true;
                }
            }
            Message::ChangeVitesse(valeur) => self.vitesse = valeur,
            Message::Grid => self.grid_state = !self.grid_state,
            Message::IncreaseVitesse => {
                if self.vitesse >= 5 {
                    self.vitesse = self.vitesse + 5;
                } else {
                    self.vitesse = 0;
                }
            }
            Message::DecreaseVitesse => {
                if self.vitesse <= 150 {
                    self.vitesse = self.vitesse - 5;
                } else {
                    self.vitesse = 150;
                }
            }
            Message::IncreaseQuickVitesse => {
                if self.vitesse >= 25 {
                    self.vitesse = self.vitesse + 25;
                } else {
                    self.vitesse = 0;
                }
            }
            Message::DecreaseQuickVitesse => {
                if self.vitesse <= 1500 {
                    self.vitesse = self.vitesse - 25;
                } else {
                    self.vitesse = 150;
                }
            }
            Message::Examples => self.screen = Screen::Example,
            Message::InputVitesse(n) => {
                if n.chars().all(|c| c.is_ascii_digit()) {
                    self.input_v = n;
                }
            }

            Message::ConvertVitesse => {
                let n = self.input_v.parse().unwrap();
                if n >= 1 && n <= 500 {
                    self.vitesse = n;
                    self.erreur_v = false;
                } else {
                    self.input_v = "".to_string();
                    self.erreur_v = true;
                }
            }
            Message::InputChangeMethod(n) => {
                if n.chars().all(|c| c.is_ascii_digit()) {
                    self.input_c = n;
                }
            }
            Message::ConvertDensity => {
                let n = self.input_c.parse().unwrap();
                if n >= 1 && n <= 100 {
                    self.living_density = n;
                    self.erreur_c = false;
                } else {
                    self.input_c = "".to_string();
                    self.erreur_c = true;
                }
            }

            Message::ConvertCells => {
                let n = self.input_c.parse().unwrap();
                if n >= 1 && n <= 5000 {
                    self.number_of_living_cells = n;
                    self.erreur_c = false;
                } else {
                    self.input_c = "".to_string();
                    self.erreur_c = true;
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
            screen: Screen::Conway,
            nb_init_cells: count_cells,
            living_density: 25,
            filling_method: true,
            number_of_living_cells: count_cells,
            initial_tab: cells_tab,
            vitesse: 100,
            grid_state: true,
            input_c: "".to_string(),
            input_v: "".to_string(),
            erreur_c: true,
            erreur_v: true,
        }
    }
}
