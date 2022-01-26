use iced::{
    button, executor, Align, Application, Button, Clipboard, Column, Command, Element, Row,
    Settings, Text, VerticalAlignment,
};

use serde::Deserialize;

#[derive(Debug, Clone)]
enum Message {
    A,
    Home,
}

#[derive(Deserialize, Debug)]
enum ExtensionKind {
    Image,
    Video,
    Text,
}

#[derive(Deserialize, Debug)]
struct File {
    name: String,
    path: String,
    kind: String,
    extension_kind: ExtensionKind,
}

impl File {
    fn view(&self) -> Element<Message> {
        Column::new()
            .push(Text::new(format!("name: {}", self.name)).size(12))
            .push(Text::new(format!("path: {}", self.path)).size(12))
            .push(Text::new(format!("{:?}", self.extension_kind)).size(12))
            .into()
    }
}

#[derive(Clone, Debug)]
enum Route {
    List,
}

struct App {
    list_button: button::State,
    route: Route,
    files: Option<Vec<File>>,
    path: String,
}

impl App {
    fn render_files(files: &mut Vec<File>) -> Element<Message> {
        let c = Column::new();
        let posts: Element<_> = files 
            .iter_mut()
            .fold(Column::new().spacing(10), |col, f| {
                col.push(f.view())
            })
            .into();
        c.push(posts).into()
    }
}

impl Application for App {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;

    fn new(_flags: ()) -> (App, Command<Message>) {
        (
            App {
                list_button: button::State::new(),
                route: Route::List,
                files: None,
                path: String::from("/"),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("App - Iced")
    }

    fn update(&mut self, message: Message, _c: &mut Clipboard) -> Command<Message> {
        println!("Got {:?}", message);
        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        let col = Column::new()
            .max_width(600)
            .spacing(10)
            .padding(10)
            .align_items(Align::Center)
            .push(Button::new(&mut self.list_button, Text::new("Home")).on_press(Message::Home));
        match self.route {
            Route::List => {
                let posts: Element<_> = match self.files {
                    None => Column::new()
                        .push(Text::new("loading...".to_owned()).size(15))
                        .into(),
                    Some(ref mut p) => App::render_files(p),
                };
                col.push(Text::new("Home".to_owned()).size(20))
                    .push(posts)
                    .into()
            }
        }
    }
}

pub fn main() -> iced::Result {
    App::run(Settings::default())
}
