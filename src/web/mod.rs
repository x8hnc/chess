use std::{
    fs,
    io::{self, BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
};

use crate::{
    board::movement::{Move, MoveResult},
    game::chess::Chess,
};

pub struct WebUI {
    listener: TcpListener,
    chess: Chess,
    play_white: bool,
    white_on_bottom: bool,
}

enum RequestResponse {
    ResourceOk(String),
    ResourceErr,
    UciOk(Move),
    UciErr,
    SyncOK,
    BotMoveOk,
    Reset,
    UnknownRequest,
}

impl From<io::Result<String>> for RequestResponse {
    fn from(value: io::Result<String>) -> Self {
        match value {
            Ok(content) => Self::ResourceOk(content),
            Err(_) => Self::ResourceErr,
        }
    }
}

impl From<Result<Move, String>> for RequestResponse {
    fn from(value: Result<Move, String>) -> Self {
        match value {
            Ok(m) => Self::UciOk(m),
            Err(_) => Self::UciErr,
        }
    }
}

impl WebUI {
    pub fn new(
        ip: &str,
        depth: usize,
        threads: usize,
        white_on_bottom: bool,
        play_white: bool,
    ) -> io::Result<Self> {
        Ok(Self {
            listener: TcpListener::bind(ip)?,
            chess: Chess::new(depth, threads),
            play_white,
            white_on_bottom,
        })
    }

    fn handle_request(stream: &TcpStream) -> RequestResponse {
        let mut buf_reader = BufReader::new(stream);
        let mut request_line = String::new();
        let _ = buf_reader.read_line(&mut request_line);

        match request_line.trim() {
            "GET / HTTP/1.1" => fs::read_to_string("src/web/index.html").into(),
            "GET /style.css HTTP/1.1" => fs::read_to_string("src/web/style.css").into(),
            "GET /script.js HTTP/1.1" => fs::read_to_string("src/web/script.js").into(),
            "GET /board HTTP/1.1" => RequestResponse::SyncOK,
            "POST /bot_move HTTP/1.1" => RequestResponse::BotMoveOk,
            "POST /reset HTTP/1.1" => RequestResponse::Reset,
            "POST /move HTTP/1.1" => {
                let mut content_length = 0;

                loop {
                    let mut line = String::new();
                    buf_reader.read_line(&mut line).unwrap();

                    let line = line.trim_end();

                    if line.is_empty() {
                        break;
                    }

                    if let Some(value) = line.strip_prefix("Content-Length: ") {
                        content_length = value.parse::<usize>().unwrap();
                    }
                }

                let mut body = vec![0u8; content_length];
                buf_reader.read_exact(&mut body).unwrap();
                let body = String::from_utf8_lossy(&body[..]).to_string();
                Move::from_uci(&body).into()
            }
            _ => RequestResponse::UnknownRequest,
        }
    }

    pub fn start(mut self) {
        let mut should_reset = false;
        for stream in self.listener.incoming() {
            let mut stream = stream.unwrap();

            let (status_line, contents) = match Self::handle_request(&stream) {
                RequestResponse::ResourceOk(content) => ("HTTP/1.1 200 OK", content),
                RequestResponse::ResourceErr => {
                    ("HTTP/1.1 500 Internal Server Error", String::new())
                }
                RequestResponse::UciOk(movement) => {
                    let move_result = self.chess.make_move(movement);
                    match move_result {
                        MoveResult::CheckMate | MoveResult::Draw => {
                            should_reset = true;
                        }
                        _ => (),
                    }
                    ("HTTP/1.1 200 OK", move_result.to_string())
                }
                RequestResponse::UciErr => ("HTTP/1.1 400 Bad UCI format", String::new()),
                RequestResponse::UnknownRequest => ("HTTP/1.1 404 Not Found", String::new()),
                RequestResponse::SyncOK => {
                    let res = ("HTTP/1.1 200 OK", self.chess.board().to_net());
                    if should_reset {
                        self.chess.reset();
                        should_reset = false;
                    }
                    res
                }
                RequestResponse::BotMoveOk => {
                    let bot_move = self.chess.search();

                    let move_result = self.chess.make_move(bot_move.0);
                    match move_result {
                        MoveResult::CheckMate | MoveResult::Draw => {
                            should_reset = true;
                        }
                        _ => (),
                    }
                    ("HTTP/1.1 200 OK", move_result.to_string())
                },
                RequestResponse::Reset => {
                    self.chess.reset();
                    ("HTTP/1.1 200 OK", String::new())
                },
            };

            let length = contents.len();
            let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

            stream.write_all(response.as_bytes()).unwrap();
        }
    }
}
