use gtk::{ButtonsType, DialogFlags, Inhibit, MessageDialog, MessageType, Window};
use gtk::Orientation::{Horizontal, Vertical};
use gtk::prelude::*;
use rand::Rng;
use relm_derive::{Msg, widget};
use relm::{Channel, Component, Relm, Sender, Widget, init};

use self::WinMsg::*;

pub struct HeaderModel {
    tx: Sender<WinMsg>,
}

#[widget]
impl Widget for Header {
    fn model(_: &Relm<Self>, tx: Sender<WinMsg>) -> HeaderModel {
        HeaderModel {
            tx,
        }
    }

    fn update(&mut self, event: WinMsg) {
        self.model.tx.send(event).expect("Event to be handled by parent.")
    }

    view! {
        #[name="titlebar"]
        gtk::HeaderBar {
            title: Some("Title"),
            show_close_button: true,

            gtk::Button {
                clicked => Spin,
                label: "Spin",
            },

            gtk::Button {
                clicked => NewGame,
                label: "New Game",
            },
        }
    }
}

pub struct Model {
    header: Component<Header>,
    is_game_over: bool,
    is_new: bool,
    tokens: u32,
    wheel1: u32,
    wheel2: u32,
    wheel3: u32,
    payout: u32,
}

impl Model {
    fn payout_alert(&self) -> String {
        format!("You got {} tokens!", self.payout)
    }

    fn payout_text(&self) -> String {
        format!("Payout: {}", self.payout)
    }

    fn token_text(&self) -> String {
        format!("Tokens: {}", self.tokens)
    }
}

#[derive(Msg)]
pub enum WinMsg {
    NewGame,
    Quit,
    Spin,
}

#[widget]
impl Widget for Win {
    fn model(relm: &Relm<Self>, _:()) -> Model {
        let stream = relm.stream().clone();

        let (_, tx) = Channel::new(move |num| {
            stream.emit(num);
        });

        let header = init::<Header>(tx).expect("Header");

        Model {
            header,
            is_new: true,
            is_game_over: false,
            tokens: 100,
            wheel1: 0,
            wheel2: 0,
            wheel3: 0,
            payout: 0,
        }
    }

    fn init_view(&mut self) {
        self.window.resize(640, 480);
    }

    fn update(&mut self, event: WinMsg) {
        match event {
            NewGame => {
                self.model.is_new = true;
                self.model.is_game_over = false;
                self.model.tokens = 100;
                self.model.wheel1 = 0;
                self.model.wheel2 = 0;
                self.model.wheel3 = 0;
                self.model.payout = 0;

                self.payout.set_text(&self.model.payout_text());
                self.tokens.set_text(&self.model.token_text());
            },
            Quit => gtk::main_quit(),
            Spin => {
                if self.model.is_new {
                    self.model.is_new = false;
                }

                if self.model.tokens > 0 {
                    let mut rng = rand::thread_rng();

                    self.model.tokens -= 1;
                    self.model.wheel1 = rng.gen_range(1, 3);
                    self.model.wheel2 = rng.gen_range(1, 3);
                    self.model.wheel3 = rng.gen_range(1, 3);

                    // 1 - 1 - 1 (3) += 4
                    // 2 - 2 - 2 (6) += 8
                    // 3 - 3 - 3 (9) += 12

                    if self.model.wheel1 == self.model.wheel2 && self.model.wheel2 == self.model.wheel3 {
                        let value = self.model.wheel1 + self.model.wheel2 + self.model.wheel3;
                        self.model.payout = (value / 3) * 4;

                        self.model.tokens += self.model.payout;
                    } else {
                        self.model.payout = 0;
                    }

                    self.payout.set_text(&self.model.payout_text());
                    self.tokens.set_text(&self.model.token_text());

                    if self.model.payout > 0 {
                        let dialog = MessageDialog::new(None::<&Window>,
                                DialogFlags::empty(),
                                MessageType::Info,
                                ButtonsType::Ok,
                                &self.model.payout_alert());

                        dialog.run();
                        dialog.destroy();
                    }

                    if self.model.tokens <= 0 {
                        self.model.is_game_over = true;
                    }
                } else {
                    self.model.is_game_over = true;
                }
            },
        }
    }

    view! {
        #[name="window"]
        gtk::Window {
            titlebar: Some(self.model.header.widget()),

            #[name="app"]
            gtk::Box {
                orientation: Vertical,
                homogeneous: true,

                #[name="state"]
                gtk::Label {
                    text: "Game Over",
                    visible: self.model.is_game_over,
                },

                #[name="placeholder"]
                gtk::Label {
                    text: "Press Spin!",
                    visible: self.model.is_new,
                },

                #[name="wheels"]
                gtk::Box {
                    orientation: Horizontal,
                    margin_top: 64,
                    margin_bottom: 64,
                    homogeneous: true,
                    visible: !self.model.is_new,

                    gtk::Label {
                        text: &self.model.wheel1.to_string(),
                    },

                    gtk::Label {
                        text: &self.model.wheel2.to_string(),
                    },

                    gtk::Label {
                        text: &self.model.wheel3.to_string(),
                    },
                },

                #[name="tokens"]
                gtk::Label {
                    text: &self.model.token_text(),
                },

                #[name="payout"]
                gtk::Label {
                    text: &self.model.payout_text(),
                    visible: !self.model.is_new,
                },
            },

            delete_event(_, _) => (Quit, Inhibit(false)),
        }
    }
}

fn main() {
    Win::run(()).expect("Window::run");
}
