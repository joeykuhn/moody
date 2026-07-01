mod number_input;

use std::time::Duration;
use std::io::{BufReader, Write};
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::layout::{Rect, Layout, Constraint};
use ratatui::{DefaultTerminal, Frame};
use tracing::info;
use std::time::Instant;
use std::fs::File;
use std::path::{PathBuf};
use serde_json;

use crate::number_input::NumberInput;

fn main() -> color_eyre::Result<()> {
    ratatui::run(app)?;
    Ok(())
}

#[derive(Debug, Default, PartialEq)]
enum Pages {
    #[default]
    Settings,
    Test,
    Results,
}

#[derive(Debug,Default, PartialEq)]
enum TestOptions {
    #[default]
    CharactersPerTarget,
    PercentageOfMaskingCharacters,
    DurationOfExperiment,
    DelayBetweenCharacters,
}

impl TestOptions {

    const ALL: [TestOptions; 4] = [
        TestOptions::CharactersPerTarget,
        TestOptions::PercentageOfMaskingCharacters,
        TestOptions::DurationOfExperiment,
        TestOptions::DelayBetweenCharacters,
    ];

    pub fn as_index(&self) -> usize {
        match self {
            TestOptions::CharactersPerTarget => 0,
            TestOptions::PercentageOfMaskingCharacters => 1,
            TestOptions::DurationOfExperiment => 2,
            TestOptions::DelayBetweenCharacters => 3,
        }
    }

    pub fn prev(&self) -> TestOptions {
        match self {
            TestOptions::CharactersPerTarget => TestOptions::CharactersPerTarget,
            TestOptions::PercentageOfMaskingCharacters => TestOptions::CharactersPerTarget,
            TestOptions::DurationOfExperiment => TestOptions::PercentageOfMaskingCharacters,
            TestOptions::DelayBetweenCharacters => TestOptions::DurationOfExperiment,
        }
    }

    pub fn next(&self) -> TestOptions {
        match self {
            TestOptions::CharactersPerTarget => TestOptions::PercentageOfMaskingCharacters,
            TestOptions::PercentageOfMaskingCharacters => TestOptions::DurationOfExperiment,
            TestOptions::DurationOfExperiment => TestOptions::DelayBetweenCharacters,
            TestOptions::DelayBetweenCharacters => TestOptions::DelayBetweenCharacters,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            TestOptions::CharactersPerTarget           => "Characters per target (3000 .. 30000)",
            TestOptions::PercentageOfMaskingCharacters => "Percentage of masking characters (1 .. 100)",
            TestOptions::DurationOfExperiment          => "Duration of experiment <minutes> (1 .. 60)",
            TestOptions::DelayBetweenCharacters        => "Delay between characters <ms> (0 .. 3000)",
        }
    }
}

#[derive(Debug, Default, serde::Serialize)]
struct OutputRecord {
    time : String,
    event : String,
}

#[derive(Debug, Default)]
struct App {
    test_str : Vec<char>,
    ticks : u64,
    quit_app : bool,
    pause : bool,
    space_presses : Vec<Instant>,
    characters_per_comma : usize,
    characters_mod : usize,
    page : Pages,
    tick_rate : u64,
    term_cols : u16,
    term_rows : u16,
    masking_odds : u16,
    comma_added : Vec<Instant>,
    comma_removed : Vec<Instant>,
    tabulated : bool,
    misses : u16,
    hits : u16,
    false_hits : usize,
    test_duration : u16,
    time_started : Option<Instant>,
    selected_option : TestOptions,
    character_delay : Duration,
    inputs : [number_input::NumberInput; 4], // the number of TestOptions
    config_file : PathBuf,
    output_dir : Option<PathBuf>,
    output_filename : String,
    events : Vec<OutputRecord>
}

impl App {
    pub fn handle_events(&mut self, timeout: &Duration) -> std::io::Result<()> {
        if crossterm::event::poll(*timeout)? {
            match crossterm::event::read() {
                Ok(evt) =>
                    match evt {
                        crossterm::event::Event::Key(key) => {
                            match self.page {
                                Pages::Settings => {
                                    if key.kind == crossterm::event::KeyEventKind::Press {
                                        match key.code {
                                            crossterm::event::KeyCode::Char('q') => self.quit_app = true,
                                            crossterm::event::KeyCode::Up => self.selected_option = self.selected_option.prev(),
                                            crossterm::event::KeyCode::Down => self.selected_option = self.selected_option.next(),
                                            crossterm::event::KeyCode::Enter => {
                                                if self.inputs.iter().all(|x| x.in_range()) {
                                                    self.characters_per_comma = self.inputs[0].value.parse::<usize>().unwrap();
                                                    self.masking_odds = self.inputs[1].value.parse::<u16>().unwrap();
                                                    self.test_duration = self.inputs[2].value.parse::<u16>().unwrap();
                                                    self.time_started = Some(Instant::now());
                                                    self.character_delay = Duration::from_millis(self.inputs[3].value.parse::<u64>().unwrap());

                                                    match File::create(&self.config_file) {
                                                        Ok(mut cf) => {cf.write(&serde_json::to_vec(&self.inputs).unwrap())?;}
                                                        Err(e) => {info!("Could not create config file: {:?}", e);}
                                                    }
                                                    self.page = Pages::Test;
                                                }
                                            }
                                            _ => {
                                                self.inputs[self.selected_option.as_index()].handle_key(&key);
                                            }
                                        }
                                    }
                                }
                                Pages::Test => {

                                    if key.kind == crossterm::event::KeyEventKind::Press {
                                        match key.code {
                                            crossterm::event::KeyCode::Char('q') => self.quit_app = true,
                                            crossterm::event::KeyCode::Char(' ') => {
                                                let inst = Instant::now();
                                                self.space_presses.push(inst);
                                                let dur_secs = (inst - self.time_started.unwrap()).as_secs();
                                                let dur_millis = (inst - self.time_started.unwrap()).subsec_millis();
                                                self.events.push(OutputRecord {
                                                    time  : format!("{dur_secs}.{dur_millis}"),
                                                    event : "input pressed".to_string()});
                                            },
                                            //crossterm::event::KeyCode::Char('p') => self.pause = !self.pause,
                                            _ => ()
                                        }
                                    }
                                }
                                Pages::Results => {
                                    match key.code {
                                        crossterm::event::KeyCode::Char('q') => self.quit_app = true,
                                        _ => ()
                                    }
                                }
                            }
                        },
                        crossterm::event::Event::Resize(x, y ) => {
                            self.term_cols = x;
                            self.term_rows = y;
                        }
                        _ => {}
                    }
                Err(e) => return Err(e)
            }
        }
        Ok(())
    }

}

fn update_model(model: &mut App){
    if !model.pause && model.page == Pages::Test {
        model.ticks += 1;

        if model.time_started.unwrap().elapsed() > Duration::from_mins(u64::from(model.test_duration)) {
            model.page = Pages::Results;
        } else if model.ticks % model.tick_rate == 0 {

            if model.characters_mod == model.characters_per_comma - 1 {
                model.test_str.push(',');
                let inst = Instant::now();
                model.comma_added.push(inst);
                let dur_secs = (inst - model.time_started.unwrap()).as_secs();
                let dur_millis = (inst - model.time_started.unwrap()).subsec_millis();
                model.events.push(OutputRecord {
                                    time  : format!("{dur_secs}.{dur_millis}"),
                                    event : "target added".to_string()});
            } else {
                let chance = rand::random_range(1..100);
                if chance <= model.masking_odds {
                    model.test_str.push(';');
                } else {
                    // using space-looking non-space so ratatui doesn't wrap "words"
                    model.test_str.push('\u{00A0}');
                }
            }

            model.characters_mod = (model.characters_mod + 1) % model.characters_per_comma;

            if u16::try_from(model.test_str.len()).unwrap() == (model.term_cols * model.term_rows) {
                let vec_to_check = model.test_str[ .. usize::from(model.term_cols)].to_vec();
                if vec_to_check.contains(&',') {
                    let inst = Instant::now();
                    model.comma_removed.push(inst);
                    let dur_secs = (inst - model.time_started.unwrap()).as_secs();
                    let dur_millis = (inst - model.time_started.unwrap()).subsec_millis();
                    model.events.push(OutputRecord {
                                        time  : format!("{dur_secs}.{dur_millis}"),
                                        event : "target removed".to_string()});
                }
                // info!("shifted at {:?}", u16::try_from(model.test_str.len()).unwrap());
                model.test_str = model.test_str[usize::from(model.term_cols) .. ].to_vec();

            }
        }
    }

    if model.page == Pages::Results && !model.tabulated {
        model.tabulated = true;

        // if the program ended with commas still on the screen, the two lists will not have the same length.
        while model.comma_added.len() > model.comma_removed.len() {
            let inst = Instant::now();
            model.comma_removed.push(inst);
            let dur_secs = (inst - model.time_started.unwrap()).as_secs();
            let dur_millis = (inst - model.time_started.unwrap()).subsec_millis();
            model.events.push(OutputRecord {
                                time  : format!("{dur_secs}.{dur_millis}"),
                                event : "target removed".to_string()});
        }

        for (comma_start, comma_end) in model.comma_added.iter().zip(model.comma_removed.iter()) {
            // If there is a space between the two comma instants...
            if let Some(index) = model.space_presses.iter().position(|x| comma_start < x && comma_end > x) {
                // ... remove the space so it doesn't count for any others
                model.space_presses.remove(index);
                model.hits += 1;
            } else {
                model.misses += 1;
            }
        }

        model.false_hits = model.space_presses.len();

        if let Some(of) = model.output_dir.as_mut() {
            let of_with_results = of.join(format!("{}_{}_{}_{}.csv", model.output_filename, model.hits, model.misses, model.false_hits));
            if let Ok(of) = File::create(of_with_results) {
                info!("writing to {:?}", of);
                let mut writer = csv::Writer::from_writer(of);
                for e in model.events.iter() {
                    writer.serialize(e).unwrap();
                }
                writer.flush().unwrap();
            } else {
                info!("failed to create output file - does the output folder exist?");
            }
        }

    }
}

fn create_config(invec : &[NumberInput; 4], config_path : &PathBuf) -> Result<(), std::io::Error> {

    match File::create(config_path) {
        Ok(mut cfile) => {
            if let Ok(inputs_json) = serde_json::to_string(invec) {
                cfile.write(inputs_json.as_bytes())?;
                Ok(())
            } else {
                info!("Inputs vector could not be serialized!");
                Ok(())
            }
        }
        Err(e) => Err(e)
    }
}

fn app(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let log_file = File::create("debug.log")?;

    tracing_subscriber::fmt().with_writer(log_file).init();

    let mut model = App::default();
    model.term_cols = terminal.size()?.width;
    model.term_rows = terminal.size()?.height;
    model.characters_per_comma = 100;
    model.tick_rate = 10;
    model.masking_odds = 50;
    model.test_duration = 1;
    model.time_started = Some(Instant::now());
    model.inputs[0].value = "3000".to_string();
    model.inputs[0].with_range(3000 ..=30000);
    model.inputs[1].value = "1".to_string();
    model.inputs[1].with_range(1 ..=100);
    model.inputs[2].value = "1".to_string();
    model.inputs[2].with_range(1 ..=60);
    model.inputs[3].value = "0".to_string();
    model.inputs[3].with_range(0 ..=3000);
    model.character_delay = Duration::from_millis(10);
    model.config_file = PathBuf::from("config.txt");

    let mut args = std::env::args();

    info!("{:?}", args);

    while let Some(arg) = args.next() {
        if arg == "--output-dir" {
            let output_arg = match args.next() {
                Some(a) => a,
                None => "./output/".to_string()
            };
            model.output_filename = chrono::Local::now().format("%Y-%m-%d_%H-%M").to_string();
            model.output_dir = Some(PathBuf::from(output_arg));
        } else if arg == "--config-file" {
            model.config_file = match args.next() {
                Some(c) => PathBuf::from(c),
                None => PathBuf::from("config.txt"),
            };
        }
    }

    info!("output directory: {:?}", model.output_dir);
    info!("output filename: {:?}", model.output_filename);
    info!("config file: {:?}", model.config_file);

    let mut create_config_file = false;
    if let Ok(cfile) = File::open(&model.config_file) {
        let reader = BufReader::new(&cfile);
        match serde_json::from_reader(reader) {
            Ok(mi) => {
                info!("reading from config file");
                model.inputs = mi;
            },
            Err(e) => {
                info!("malformed config file, overwriting because {:?}", e);
                create_config_file = true;
            }
        }
    } else { //could not open config file (likely due to nonexistence)
        info!("no config file found, creating one");
        create_config_file = true;
    }

    if create_config_file {
        create_config(&model.inputs, &model.config_file)?;
    }

    let mut last_tick = Instant::now();
    loop {
        if last_tick.elapsed() >= model.character_delay {
            last_tick = Instant::now();

            update_model(&mut model);
            terminal.draw(|frame| {render(frame, &model)})?;

            if model.quit_app {
                if model.time_started.is_some() {
                    info!("Average tick time: {} microseconds", model.time_started.unwrap().elapsed().as_micros() / model.ticks as u128);
                    info!("length of test: {:?} microseconds", model.time_started.unwrap().elapsed().as_micros());
                    info!("number of ticks: {}", model.ticks);
                }
                return Ok(())
            }

        }
        let timeout = model.character_delay
                                .checked_sub(last_tick.elapsed())
                                .unwrap_or(Duration::ZERO);
        model.handle_events(&timeout)?;
    }
}

fn render(frame: &mut Frame, model: &App) {

    match &model.page {
        Pages::Settings => {
            let mut working_row = 1;
            for (i, s) in TestOptions::ALL.iter().enumerate() {
                let setting_area = Rect::new(0, working_row, frame.area().width, 1);
                let columns: [Rect; 4] = Layout::horizontal([
                    Constraint::Length(45), // description of setting
                    Constraint::Length(10), // buffer space
                    Constraint::Length(4),  // input
                    Constraint::Fill(1),    // buffer after input
                ]).areas(setting_area);
                let setting_desc = Paragraph::new(s.as_str());
                frame.render_widget(setting_desc, columns[0]);
                model.inputs[i].render(frame, columns[2], model.selected_option == *s);

                working_row += 2;
            }

            let explanation = Paragraph::new("q: quit\nEnter: begin testing.");

            let explanation_rect = Rect::new(0, working_row, 21, 2);

            frame.render_widget(explanation, explanation_rect);

        },
        Pages::Test => {
            let debug_str = model.test_str.clone();
            let paragraph = Paragraph::new(debug_str.iter().collect::<String>())
                                                         .wrap(Wrap {trim: false});
            frame.render_widget(paragraph, frame.area());
        },
        Pages::Results => {
            // There is almost certainly a better way to build these up...
            let hits_rect = Rect::new(0,0, frame.area().width, 1);
            let misses_rect = Rect::new(0,1, frame.area().width, 1);
            let false_hits_rect = Rect::new(0,2, frame.area().width, 1);
            let hits_str = format!("Hits: {}", model.hits);
            let misses_str = format!("Misses: {}", model.misses);
            let false_hits_str = format!("False Hits: {}", model.false_hits);
            let hits_paragraph = Paragraph::new(hits_str);
            let misses_paragraph = Paragraph::new(misses_str);
            let false_hits_paragraph = Paragraph::new(false_hits_str);
            frame.render_widget(hits_paragraph, hits_rect);
            frame.render_widget(misses_paragraph, misses_rect);
            frame.render_widget(false_hits_paragraph, false_hits_rect);

        }
    }

}
