use std::{
    io::{BufRead, Write},
    marker::PhantomData,
    sync::mpsc::{channel, Receiver, Sender},
    thread::{self, JoinHandle},
};

pub use vampirc_uci::*;

pub trait Engine {
    fn id() -> (Option<String>, Option<String>) {
        (None, None)
    }

    fn go(
        &mut self,
        reciver: &Receiver<()>,
        sender: &mut Sender<RecivedMessage>,
        time_control: Option<UciTimeControl>,
        search_control: Option<UciSearchControl>,
    );

    fn options() -> Vec<UciOptionConfig>;

    fn set_option(&mut self, name: &str, value: Option<&str>);

    fn new_game(&mut self);

    fn set_pos(&mut self, fen: &str, moves: Vec<UciMove>);
}

pub enum EngineCommand {
    NewGame,
    IsReady,
    SetPos {
        fen: String,
        moves: Vec<UciMove>,
    },
    Go {
        time_control: Option<vampirc_uci::UciTimeControl>,
        search_control: Option<vampirc_uci::UciSearchControl>,
    },
    SetOption {
        name: String,
        value: Option<String>,
    },
}

pub enum RecivedMessage {
    BestMove(UciMove),
    Info(Vec<UciInfoAttribute>),
    ReadyOk,
    Uci(UciMessage),
}

pub struct UciConnection<W: Write, E: 'static + Engine + Send> {
    writer: W,

    message_reciver: Receiver<RecivedMessage>,

    engine_thread: JoinHandle<()>,
    stop_sender: Sender<()>,
    engine_sender: Sender<EngineCommand>,

    input_thread: JoinHandle<()>,

    e: PhantomData<E>,
}

impl<W: Write, E: 'static + Engine + Send> UciConnection<W, E> {
    pub fn new<R: 'static + BufRead + Send>(reader: R, writer: W, mut engine: E) -> Self {
        let (engine_command_sender, engine_command_recv) = channel();

        let (stop_sender, stop_recv) = channel();
        let (mut message_sender, message_recv) = channel();

        let input_sender = message_sender.clone();

        let engine_thread = thread::spawn(move || {
            while let Ok(message) = engine_command_recv.recv() {
                match message {
                    EngineCommand::Go { time_control, search_control } => {
                        engine.go(&stop_recv, &mut message_sender, time_control, search_control);
                    }
                    EngineCommand::SetPos { fen, moves } => {
                        engine.set_pos(&fen, moves);
                    }
                    EngineCommand::NewGame => {
                        engine.new_game();
                    }
                    EngineCommand::IsReady => {
                        _ = message_sender.send(RecivedMessage::ReadyOk);
                    }
                    EngineCommand::SetOption { name, value } => {
                        engine.set_option(&name, value.as_deref());
                    }
                }
            }
        });

        let input_thread = thread::spawn(move || {
            for line in reader.lines() {
                if let Err(_) = input_sender.send(RecivedMessage::Uci(parse_one(&line.unwrap()))) {
                    break;
                }
            }
        });

        Self {
            writer,
            engine_thread,
            message_reciver: message_recv,
            engine_sender: engine_command_sender,
            stop_sender,
            input_thread,
            e: Default::default(),
        }
    }

    pub fn run(mut self) {
        while let Ok(message) = self.message_reciver.recv() {
            match message {
                RecivedMessage::Uci(message) => match message {
                    UciMessage::Uci => {
                        self.writer.write_fmt(format_args!("{}\n", UciMessage::UciOk)).unwrap();

                        for option in E::options() {
                            _ = self
                                .writer
                                .write_fmt(format_args!("{}\n", UciMessage::Option(option)))
                        }
                    }
                    UciMessage::IsReady => {
                        _ = self.engine_sender.send(EngineCommand::IsReady);
                    }
                    UciMessage::Go { time_control, search_control } => {
                        _ = self
                            .engine_sender
                            .send(EngineCommand::Go { time_control, search_control });
                    }
                    UciMessage::Position { startpos, fen, moves } => {
                        let fen = if startpos || fen.is_none() {
                            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
                        } else {
                            fen.as_ref().unwrap().0.as_str()
                        }
                        .to_string();

                        _ = self.engine_sender.send(EngineCommand::SetPos { fen, moves });
                    }
                    UciMessage::UciNewGame => {
                        _ = self.engine_sender.send(EngineCommand::NewGame);
                        _ = self
                            .writer
                            .write_fmt(format_args!("{}\n", UciMessage::ReadyOk))
                            .unwrap();
                    }
                    UciMessage::SetOption { name, value } => {
                        _ = self.engine_sender.send(EngineCommand::SetOption { name, value })
                    }
                    UciMessage::Stop => {
                        let _ = self.stop_sender.send(());
                    }
                    UciMessage::Quit => {
                        return;
                    }
                    UciMessage::Unknown(s, err) => {
                        _ = self
                            .writer
                            .write_fmt(format_args!("Uknown uci command: {}\n", s))
                            .unwrap();

                        if let Some(err) = err {
                            _ = self.writer.write_fmt(format_args!("{}\n", err)).unwrap();
                        }
                    }
                    _ => {}
                },
                RecivedMessage::BestMove(uci_move) => {
                    self.writer
                        .write_fmt(format_args!(
                            "{}\n",
                            UciMessage::BestMove { best_move: uci_move, ponder: None }
                        ))
                        .unwrap();
                }
                RecivedMessage::Info(info) => {
                    self.writer.write_fmt(format_args!("{}\n", UciMessage::Info(info))).unwrap();
                }
                RecivedMessage::ReadyOk => {
                    self.writer.write_fmt(format_args!("{}\n", UciMessage::ReadyOk)).unwrap();
                }
            }
        }

        self.input_thread.join().unwrap();
        self.engine_thread.join().unwrap();
    }
}
