use instant::{Duration, Instant};
use seed::{prelude::*, *};

use sudoku::*;

#[wasm_bindgen(start)]
pub fn start() {
    App::start("app", init, update, view);
}

#[derive(Debug)]
struct Model {
    sudoku: Sudoku,
    solution: Board,
    sq_selected: Option<Square>,
    sq_missed: Option<Square>,
    miss_count: u32,
    state: State,
}

impl Default for Model {
    fn default() -> Self {
        let sudoku = Sudoku::new(Board::empty());

        Self {
            sudoku,
            solution: Board::empty(),
            sq_selected: None,
            sq_missed: None,
            miss_count: 0,
            state: State::Startup,
        }
    }
}

impl Model {
    fn solution_at(&self, sq: Square) -> Number {
        self.solution[sq].unwrap()
    }
}

#[derive(Debug)]
enum State {
    Startup,
    Playing { now: Instant },
    Completed { dur: Duration },
}

#[derive(Debug)]
enum Msg {
    TimerTick,
    Reset,
    SelectSquare(Square),
    PutNumber { sq: Square, num: Number },
}

fn init(_url: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.send_msg(Msg::Reset);
    orders.stream(streams::interval(100, || Msg::TimerTick));

    Model::default()
}

fn update(msg: Msg, model: &mut Model, _orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::TimerTick => {}
        Msg::Reset => {
            let (sudoku, solution) = Sudoku::generate_unique(35);
            model.sudoku = sudoku;
            model.solution = solution.board().clone();
            model.sq_selected = None;
            model.sq_missed = None;
            model.miss_count = 0;
            model.state = State::Playing {
                now: Instant::now(),
            };
        }
        Msg::SelectSquare(sq) => {
            model.sq_selected = Some(sq);
        }
        Msg::PutNumber { sq, num } => {
            if num != model.solution_at(sq) {
                model.sq_missed = Some(sq);
                model.miss_count += 1;
                return;
            }
            if !model.sudoku.put(sq, num) {
                log!("internal error: sudoku.put() should succeed");
            }
            model.sq_missed = None;
            if model.sudoku.is_solved() {
                let State::Playing{now} = model.state else {
                    return;
                };
                let dur = now.elapsed();
                model.state = State::Completed { dur };
            }
        }
    }
}

const SQUARE_WIDTH: u32 = 100;
const SQUARE_HEIGHT: u32 = 100;

const NUMBER_WIDTH: u32 = 100;
const NUMBER_HEIGHT: u32 = 100;

fn view(model: &Model) -> Node<Msg> {
    div![
        id!("app-container"),
        view_sudoku(model),
        view_control(model)
    ]
}

fn view_control(model: &Model) -> Node<Msg> {
    div![
        id!("control-container"),
        view_control_timer(model),
        view_control_miss_count(model),
        view_control_reset(model),
        view_control_complete(model),
    ]
}

fn view_control_reset(_model: &Model) -> Node<Msg> {
    div![button![
        C!["button-reset"],
        attrs! {
            At::Type => "button",
        },
        "リセット",
        ev(Ev::Click, |_| Msg::Reset)
    ]]
}

fn view_control_timer(model: &Model) -> Node<Msg> {
    fn format_duration(dur: Duration) -> String {
        let secs = dur.as_secs();
        let mins = secs / 60;
        let secs = secs % 60;
        format!("{mins:02}:{secs:02}")
    }

    let text_dur = match model.state {
        State::Playing { now } => format_duration(now.elapsed()),
        State::Completed { dur } => format_duration(dur),
        _ => "".to_owned(),
    };
    let text = format!("Time: {text_dur}");

    div![C!["output-time"], text]
}

fn view_control_miss_count(model: &Model) -> Node<Msg> {
    let text = format!("Miss: {}", model.miss_count);

    div![C!["output-miss"], text]
}

fn view_control_complete(model: &Model) -> Node<Msg> {
    let text = if matches!(model.state, State::Completed { .. }) {
        "Complete!!"
    } else {
        ""
    };

    div![C!["output-complete"], text]
}

fn view_sudoku(model: &Model) -> Node<Msg> {
    div![
        id!("sudoku-container"),
        view_board(model),
        view_numbers(model),
    ]
}

fn view_board(model: &Model) -> Node<Msg> {
    let rows = Row::all().into_iter().map(|row| view_board_row(model, row));

    table![id!("board-container"), rows]
}

fn view_board_row(model: &Model, row: Row) -> Node<Msg> {
    let squares = Col::all()
        .into_iter()
        .map(|col| view_board_square(model, col, row));

    tr![squares]
}

fn view_board_square(model: &Model, col: Col, row: Row) -> Node<Msg> {
    let sq = Square::from_col_row(col, row);
    let board = model.sudoku.board();
    let sq_sel = model.sq_selected;

    let is_selected = sq_sel == Some(sq);
    let is_neighbor = !is_selected
        && sq_sel.map_or(false, |sq_sel| {
            col == sq_sel.col() || row == sq_sel.row() || sq.block() == sq_sel.block()
        });
    let is_selected_number = sq_sel.map_or(false, |sq_sel| {
        board[sq].is_some() && board[sq] == board[sq_sel]
    });
    let is_missed = model.sq_missed == Some(sq);

    let text = if is_missed {
        "☓".to_owned()
    } else {
        board[sq].map_or("".to_owned(), |num| num.get().to_string())
    };
    let borders = view_square_borders(sq);

    td![
        style! {
            St::BorderStyle => "solid",
            St::BorderColor => "black",
            St::BorderTopWidth => borders[0],
            St::BorderRightWidth => borders[1],
            St::BorderBottomWidth => borders[2],
            St::BorderLeftWidth => borders[3],
        },
        div![
            C![
                "square",
                IF!(is_selected => "square-selected"),
                IF!(is_neighbor => "square-neighbor"),
                IF!(is_selected_number => "square-selected-number"),
                IF!(is_missed => "square-missed"),
            ],
            style! {
                St::Width => px(SQUARE_WIDTH),
                St::Height => px(SQUARE_HEIGHT),
                St::FontSize => px(f64::from(SQUARE_HEIGHT) * 0.8),
            },
            text,
            ev(Ev::Click, move |_| Msg::SelectSquare(sq))
        ]
    ]
}

fn view_square_borders(sq: Square) -> [String; 4] {
    const THICK: u32 = 8;
    const THIN: u32 = 2;

    let top = px(if sq.row().get() % 3 == 0 { THICK } else { THIN });
    let right = px(if sq.col().get() % 3 == 2 { THICK } else { THIN });
    let bottom = px(if sq.row().get() % 3 == 2 { THICK } else { THIN });
    let left = px(if sq.col().get() % 3 == 0 { THICK } else { THIN });

    [top, right, bottom, left]
}

fn view_numbers(model: &Model) -> Node<Msg> {
    let numbers = Number::all().into_iter().map(|num| view_number(model, num));

    div![
        id!("numbers-container"),
        style! {
            St::Width => px(f64::from(NUMBER_WIDTH) * 9.5),
            St::Height => px(NUMBER_HEIGHT),
        },
        numbers,
    ]
}

fn view_number(model: &Model, num: Number) -> Node<Msg> {
    let sq_sel = model.sq_selected;
    let is_completed = Square::all()
        .into_iter()
        .filter(|&sq| model.sudoku.board()[sq] == Some(num))
        .count()
        == 9;

    let text = num.get().to_string();

    div![
        C!["number", IF!(is_completed => "number-completed")],
        style! {
            St::Width => px(NUMBER_WIDTH),
            St::Height => px(NUMBER_HEIGHT),
            St::FontSize => px(f64::from(NUMBER_HEIGHT) * 0.8),
        },
        text,
        ev(Ev::Click, move |_| {
            if is_completed {
                return None;
            }
            sq_sel.map(|sq| Msg::PutNumber { sq, num })
        })
    ]
}
