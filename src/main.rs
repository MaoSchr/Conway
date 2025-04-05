use std::fmt;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use iced::widget::Image;
use iced::{
    color, time,
    widget::{button, column, container, row, text, text_input, Button, Column, Row, Svg},
    Border, Color, Element, Length, Subscription, Theme,
};

use image::{Rgb, RgbImage};
use rand::Rng;
use serde::de::{self, SeqAccess, Visitor};
use serde::Deserializer;
use serde::{ser::SerializeSeq, Deserialize, Serialize, Serializer};
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
    Sauvegarder,
    ChargerEScreen,
    ChargerEFinal(usize),
    ChargerSScreen,
    ChargerSFinal(usize),
    Conway,
    Tick,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
enum Screen {
    Init,
    Simul,
    Example,
    Conway,
    ExamplesC,
    SavesC,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct Cell {
    living: bool,
}
#[derive(Debug, Clone, Copy)]
struct Tab([[Cell; Conway::SIZE]; Conway::SIZE]);

impl Serialize for Tab {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(Conway::SIZE * Conway::SIZE))?;
        for row in &self.0 {
            for cell in row {
                seq.serialize_element(&cell)?;
            }
        }

        seq.end()
    }
}

impl<'de> Deserialize<'de> for Tab {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TabVisitor;

        impl<'de> Visitor<'de> for TabVisitor {
            type Value = Tab;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(
                    formatter,
                    "a 2D array of size {}x{}",
                    Conway::SIZE,
                    Conway::SIZE
                )
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Tab, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut cells = [[Cell { living: false }; Conway::SIZE]; Conway::SIZE];
                for i in 0..Conway::SIZE {
                    for j in 0..Conway::SIZE {
                        let cell: Cell = seq.next_element()?.ok_or_else(|| {
                            de::Error::invalid_length(i * Conway::SIZE + j, &self)
                        })?;
                        cells[i][j] = cell;
                    }
                }
                Ok(Tab(cells))
            }
        }

        deserializer.deserialize_seq(TabVisitor)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Conway {
    nb_init_cells: u32,
    cells_tab: Tab,
    playing: bool,
    generation: u64,
    screen: Screen,
    filling_method: bool,
    living_density: u32,
    number_of_living_cells: u32,
    initial_tab: Tab,
    vitesse: u32,
    grid_state: bool,
    input_v: String,
    input_c: String,
    erreur_v: bool,
    erreur_c: bool,
    nb_sauvegardes: usize,
    current_frame: usize,
}

impl Conway {
    const SIZE: usize = 50;
    fn subscription(&self) -> Subscription<Message> {
        if self.playing {
            time::every(time::Duration::from_millis(self.vitesse as u64).into())
                .map(|_| Message::Update)
        } else {
            time::every(time::Duration::from_millis(100).into()).map(|_| Message::Tick)
        }
    }

    fn create_frames(&self) -> Vec<String> {
        let mut paths: Vec<String> = Vec::new();
        for i in 0..30 {
            paths.push(format!("Frames/frame_{:02}_delay-0.1s.png", i).to_string());
        }
        paths
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

                if self.cells_tab.0[nx as usize][ny as usize].living {
                    living_neighbours += 1;
                }
            }
        }
        living_neighbours
    }

    fn build_cells_with_density(&mut self) {
        let mut count_cells = 0;
        let mut rng = rand::thread_rng();
        let mut cells_tab = Tab::default();

        for x in 0..Self::SIZE {
            for y in 0..Self::SIZE {
                cells_tab.0[x][y] = Cell {
                    living: rng.gen_bool(self.living_density as f64 / 100.0),
                };
                if cells_tab.0[x][y].living {
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
        self.cells_tab = Tab::default();
        while count_cells < self.nb_init_cells {
            let x = rng.gen_range(0..Self::SIZE);
            let y = rng.gen_range(0..Self::SIZE);
            if !self.cells_tab.0[x][y].living {
                self.cells_tab.0[x][y].living = true;
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
            Screen::ExamplesC => "Charger un exemple".into(),
            Screen::SavesC => "Charger une sauvegarde".into(),
        }
    }

    fn view(&self) -> Element<Message> {
        let screen = match self.screen {
            Screen::Init => self.init(),
            Screen::Simul => self.simulation(),
            Screen::Example => self.examples(),
            Screen::Conway => self.conway(),
            Screen::ExamplesC => self.charge_examples(),
            Screen::SavesC => self.charger_saves(),
        };
        container(screen).into()
    }

    fn charge_examples(&self) -> Element<Message> {
        let i_max = 1;
        let mut row_s = Row::new();
        for i in 0..i_max {
            let mut column_s = Column::new();
            for _j in 0..1 {
                let image = Image::new(format!("examples/miniatures/miniature{}.png", i));
                column_s = column_s.push(button(image).on_press(Message::ChargerSFinal(i)));
                column_s = column_s.push(text("Gosper Glider Gun"));
            }
            row_s = row_s.push(column_s);
        }

        row_s = row_s.push(button("Menu_principal").on_press(Message::Conway));
        row_s.into()
    }

    fn charger_saves(&self) -> Element<Message> {
        let i_max = self.nb_sauvegardes;
        let mut row_s = Row::new();
        for i in 0..i_max {
            let mut column_s = Column::new();
            for _j in 0..1 {
                let image = Image::new(format!("saves/miniatures/miniature{}.png", i));
                column_s = column_s.push(button(image).on_press(Message::ChargerSFinal(i)));
            }
            row_s = row_s.push(column_s);
        }

        row_s = row_s.push(button("Menu_principal").on_press(Message::Conway));
        row_s.into()
    }

    fn examples(&self) -> Element<Message> {
        column![
            button("Charger un exemple").on_press(Message::ChargerEScreen),
            button("Charger une sauvegarde").on_press(Message::ChargerSScreen),
            button("Simulation").on_press(Message::Simulation),
            text("En cours...")
        ]
        .into()
    }

    fn conway(&self) -> Element<Message> {
        let images = self.create_frames();
        let path = PathBuf::from(&images[self.current_frame]);
        let handle = iced::widget::image::Handle::from(path);
        let conway_image = Image::new(handle).width(Length::Fill).height(Length::Fill);

        container(column![
            container(text("Jeu de Conway").size(50)).center(Length::Fill),
            container(conway_image).center(Length::Fill),
            container(
                row![
                    button("Paramètres").on_press(Message::Settings),
                    button("Bac à sable").on_press(Message::Simulation),
                    button("Charger une sauvegarde").on_press(Message::Examples)
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

    fn create_miniature(&self) {
        let mut img = RgbImage::new((Self::SIZE * 2) as u32, (Self::SIZE * 2) as u32);
        for i in 0..Self::SIZE {
            for j in 0..Self::SIZE {
                if self.cells_tab.0[i][j].living {
                    for di in 0..2 {
                        for dj in 0..2 {
                            img.put_pixel((2 * i + di) as u32, (2 * j + dj) as u32, Rgb([0, 0, 0]));
                        }
                    }
                } else {
                    for di in 0..2 {
                        for dj in 0..2 {
                            img.put_pixel(
                                (2 * i + di) as u32,
                                (2 * j + dj) as u32,
                                Rgb([255, 255, 255]),
                            );
                        }
                    }
                }
            }
        }
        img.save(format!(
            "./saves/miniatures/miniature{}.png",
            self.nb_sauvegardes
        ))
        .expect("Erreur lors de la sauvegarde de l'image");
    }

    fn compter_documents(dossier: &Path) -> Result<usize, std::io::Error> {
        let mut nombre_documents = 0;

        for entrée in fs::read_dir(dossier)? {
            let entrée = entrée?;
            if entrée.file_type()?.is_file() {
                nombre_documents += 1;
            }
        }

        Ok(nombre_documents)
    }

    fn simulation(&self) -> Element<Message> {
        let mut column_conway = Column::new();
        for y in 0..Self::SIZE {
            let mut row = Row::new();
            for x in 0..Self::SIZE {
                let living = self.cells_tab.0[x][y].living;
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
            button("Paramètres")
                .on_press(Message::Settings)
                .style(button::secondary),
            button("Exemples")
                .on_press(Message::Examples)
                .style(button::secondary),
            button("Sauvegarder")
                .on_press(Message::Sauvegarder)
                .style(button::secondary),
            button("Menu principal")
                .on_press(Message::Conway)
                .style(button::secondary)
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

        let control_row = row![lecture_buttons, vitesse_buttons, settings_buttons].spacing(120);
        column![column_conway, control_row, info_row,].into()
    }

    fn update_cells(&mut self) {
        let mut next_cells_tab = self.cells_tab;
        for x in 0..Self::SIZE {
            for y in 0..Self::SIZE {
                let living_neighbours = self.check_neighbours(x, y);

                next_cells_tab.0[x][y].living =
                    match (self.cells_tab.0[x][y].living, living_neighbours) {
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
                    let cells_tab = Tab::default();
                    *self = Conway {
                        cells_tab,
                        playing: false,
                        generation: 1,
                        screen: Screen::Simul,
                        nb_init_cells: 0,
                        living_density: 0,
                        filling_method: true,
                        number_of_living_cells: 0,
                        initial_tab: (cells_tab),
                        vitesse: 100,
                        grid_state: true,
                        input_c: "".to_string(),
                        input_v: "".to_string(),
                        erreur_c: true,
                        erreur_v: true,
                        nb_sauvegardes: Self::compter_documents(Path::new("./saves/main")).unwrap(),
                        current_frame: 0,
                    }
                }
                Screen::Init => {
                    match self.filling_method {
                        true => Self::build_cells_with_density(self),
                        false => Self::build_cells_with_number_of_cells(self),
                    };
                    self.screen = Screen::Simul;
                }
                Screen::Example => self.screen = Screen::Simul,
                Screen::Simul => (),
                Screen::SavesC => (),
                Screen::ExamplesC => (),
            },
            Message::Settings => self.screen = Screen::Init,
            Message::Réinitialiser => {
                Self::réinitialiser(self);
                self.playing = false;
                self.generation = 1;
            }

            Message::ActiverDésactiver(x, y) => {
                if self.cells_tab.0[x][y].living {
                    self.number_of_living_cells -= 1;
                    self.cells_tab.0[x][y].living = false;
                } else {
                    self.number_of_living_cells += 1;
                    self.cells_tab.0[x][y].living = true;
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
            Message::Conway => self.screen = Screen::Conway,
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
            Message::Sauvegarder => {
                let serialized = match serde_json::to_string(&self) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Erreur lors de la sérialisation : {}", e);
                        return;
                    }
                };
                Self::create_miniature(self);
                let file_name = format!("saves/main/Sauvegarde{}.txt", self.nb_sauvegardes);

                self.nb_sauvegardes += 1;
                let mut file = match File::create(&file_name) {
                    Ok(f) => f,
                    Err(e) => {
                        eprintln!("Erreur lors de la création du fichier {}: {}", file_name, e);
                        return;
                    }
                };

                if let Err(e) = file.write_all(serialized.as_bytes()) {
                    eprintln!(
                        "Erreur lors de l'écriture dans le fichier {}: {}",
                        file_name, e
                    );
                };
            }
            Message::ChargerSScreen => self.screen = Screen::SavesC,
            Message::ChargerSFinal(i) => {
                let mut file = File::open(format!("./saves/main/Sauvegarde{}.txt", i))
                    .expect("Unable to open file");
                let mut contents = String::new();
                file.read_to_string(&mut contents)
                    .expect("Unable to read file");
                let deserialized: Conway = serde_json::from_str(&contents).unwrap();
                *self = Conway {
                    cells_tab: deserialized.cells_tab,
                    playing: false,
                    generation: 1,
                    screen: Screen::Simul,
                    nb_init_cells: 0,
                    living_density: 0,
                    filling_method: true,
                    number_of_living_cells: 0,
                    initial_tab: (deserialized.initial_tab),
                    vitesse: 100,
                    grid_state: true,
                    input_c: "".to_string(),
                    input_v: "".to_string(),
                    erreur_c: true,
                    erreur_v: true,
                    nb_sauvegardes: Self::compter_documents(Path::new("./saves/main")).unwrap(),
                    current_frame: 0,
                }
            }
            Message::ChargerEScreen => self.screen = Screen::ExamplesC,
            Message::ChargerEFinal(i) => {
                let mut file = File::open(format!("./saves/main/Sauvegarde{}.txt", i))
                    .expect("Unable to open file");
                let mut contents = String::new();
                file.read_to_string(&mut contents)
                    .expect("Unable to read file");
                let deserialized: Conway = serde_json::from_str(&contents).unwrap();
                *self = Conway {
                    cells_tab: deserialized.cells_tab,
                    playing: false,
                    generation: 1,
                    screen: Screen::Simul,
                    nb_init_cells: 0,
                    living_density: 0,
                    filling_method: true,
                    number_of_living_cells: 0,
                    initial_tab: (deserialized.initial_tab),
                    vitesse: 100,
                    grid_state: true,
                    input_c: "".to_string(),
                    input_v: "".to_string(),
                    erreur_c: true,
                    erreur_v: true,
                    nb_sauvegardes: Self::compter_documents(Path::new("./saves/main")).unwrap(),
                    current_frame: 0,
                }
            }
            Message::Tick => {
                self.current_frame = (self.current_frame + 1) % (Self::create_frames(self).len())
            }
        }
    }
}
impl Default for Tab {
    fn default() -> Self {
        Tab([[Cell { living: false }; Conway::SIZE]; Conway::SIZE])
    }
}

impl Default for Conway {
    fn default() -> Self {
        let mut count_cells = 0;
        let density = 25;
        let mut rng = rand::thread_rng();
        let mut cells_tab = Tab::default();
        for x in 0..Self::SIZE {
            for y in 0..Self::SIZE {
                cells_tab.0[x][y] = Cell {
                    living: rng.gen_bool(density as f64 / 100.0),
                };
                if cells_tab.0[x][y].living {
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
            nb_sauvegardes: Self::compter_documents(Path::new("./saves/main")).unwrap(),
            current_frame: 0,
        }
    }
}
