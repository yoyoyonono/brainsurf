use std::path::PathBuf;
use std::thread::current;

use iced::widget::container::bordered_box;
use iced::widget::{Button, Column, Container, Slider};
use iced::widget::{
    button, checkbox, column, container, horizontal_space, image, radio, row, scrollable, slider,
    text, text_input, toggler, vertical_space,
};
use iced::{Center, Color, Element, Fill, Font, Length, Pixels, alignment};

mod logic;
use crate::logic::{ModInfo, download_mod, get_all_mods, install_mod};

pub fn main() -> iced::Result {
    iced::application(Brainsurf::title, Brainsurf::update, Brainsurf::view)
        .window_size(iced::Size::new(1280.0, 720.0))
        .centered()
        .run()
}

pub struct Brainsurf {
    screen: Screen,
    mods: Vec<ModInfo>,
    selected_mod: Option<ModInfo>,
    data_win_path: Option<PathBuf>,
    done_installing: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    ChangeScreen(Screen),
    ChangeSelected(ModInfo),
    InstallMod,
    PickDataWinFile,
    LaunchGame,
    OpenModPage,
}

impl Brainsurf {
    fn title(&self) -> String {
        let screen = match self.screen {
            Screen::Warning => "WARNING",
            Screen::ModPicker => "Mod Picker",
        };

        format!("brainsurf - {}", screen)
    }

    fn update(&mut self, event: Message) {
        match event {
            Message::ChangeScreen(new_screen) => {
                self.screen = new_screen;
            }
            Message::ChangeSelected(new_selected) => {
                self.selected_mod = Some(new_selected);
                self.done_installing = false;
            }
            Message::InstallMod => {
                self.done_installing = false;
                download_mod(self.selected_mod.as_ref().unwrap());
                install_mod(
                    (self.selected_mod.as_ref()).unwrap(),
                    (self.data_win_path.as_ref()).unwrap().to_path_buf(),
                );
                self.done_installing = true;
            }
            Message::PickDataWinFile => {
                self.data_win_path = Some(
                    rfd::FileDialog::new()
                        .set_title("Choose your MINDWAVE data.win file")
                        .add_filter("file", &["win"])
                        .pick_file()
                        .unwrap(),
                );
            }
            Message::LaunchGame => {
                open::that("steam://rungameid/2741670").unwrap();
            }
            Message::OpenModPage => {
                open::that(format!("https://gamebanana.com/mods/{}", self.selected_mod.as_ref().unwrap().id)).unwrap();
            }
        }
    }

    fn view(&self) -> Element<Message> {
        match self.screen {
            Screen::Warning => self.welcome(),
            Screen::ModPicker => self.mod_picker(),
        }
        .into()
    }

    fn welcome(&self) -> Element<Message> {
        let content = Self::container("⚠WARNING⚠")
            .push("You agree that you've made backups blababla")
            .push(button("Continue").on_press(Message::ChangeScreen(Screen::ModPicker)))
            .align_x(alignment::Horizontal::Center);

        container(content).center_x(Fill).center_y(Fill).into()
    }

    fn mod_picker(&self) -> Element<Message> {
        let text_content = text("Choose a mod")
            .size(30)
            .align_x(alignment::Horizontal::Left)
            .align_y(alignment::Vertical::Top);
        let title_container: Element<Message> = container(text_content).padding(20).into();

        let mut mods_list: Column<Message> = column![];

        for x in &self.mods {
            mods_list = mods_list.push(
                radio(&x.name, x, self.selected_mod.as_ref(), |x: &ModInfo| {
                    Message::ChangeSelected(x.clone())
                })
                .size(20),
            );
        }

        let mods_container: Element<Message> = container(mods_list.padding(10).spacing(10)).into();

        let folder_button = button(text("Pick data.win file")).on_press(Message::PickDataWinFile);
        let install_button = button(text("Install")).on_press(Message::InstallMod);
        let launch_button = button(text("Launch")).on_press(Message::LaunchGame);

        let buttons = row![folder_button, install_button, launch_button]
            .spacing(10)
            .padding(10);

        let is_done = text(if self.done_installing {
            "Done Installing"
        } else {
            ""
        })
        .size(20);

        let left_half = column![title_container, mods_container, buttons, is_done];

        let mod_info_name = container(text(if let Some(current_mod) = &self.selected_mod {
            current_mod.name.clone()
        } else {
            "".to_string()
        }).size(40));

        let mod_description = text(if let Some(current_mod) = &self.selected_mod {
            current_mod.description.clone().unwrap_or("".to_string())
        } else {
            "".to_string()
        }).size(30);

        let mod_author = text(format!("Author: {}", if let Some(current_mod) = &self.selected_mod {
            current_mod.submitter.name.clone()
        } else {
            "".to_string()
        })).size(20);

        let mod_text = text(format!("Description: {}", if let Some(current_mod) = &self.selected_mod {
            current_mod.text.clone().unwrap_or("".to_string())
        } else {
            "".to_string()
        })).size(20);

        let open_mod_page = button(text("Open Gamebanana Page")).on_press(Message::OpenModPage);

        let right_half = column![mod_info_name, mod_description, mod_author, mod_text, open_mod_page].padding(20);

        row![left_half.width(Length::Fill), right_half.width(Length::Fill)].into()
    }

    fn container(title: &str) -> Column<'_, Message> {
        column![text(title).size(30)].spacing(20)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Warning,
    ModPicker,
}

impl Default for Brainsurf {
    fn default() -> Self {
        let mods = get_all_mods();
        Self {
            screen: Screen::Warning,
            mods: mods,
            selected_mod: None,
            data_win_path: None,
            done_installing: false,
        }
    }
}
