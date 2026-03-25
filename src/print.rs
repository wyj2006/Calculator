use crate::expr::Expr;
use std::fmt::Display;
use unicode_width::UnicodeWidthStr;

pub struct Canvas(Vec<Vec<char>>);

impl Canvas {
    pub fn new(width: usize, height: usize) -> Canvas {
        Canvas(vec![vec![' '; width]; height])
    }

    pub fn paint_at(&mut self, canvas: &Canvas, x: usize, y: usize) {
        for (i, v) in canvas.0.iter().enumerate() {
            for (j, v) in v.iter().enumerate() {
                self.0[i + y][j + x] = *v;
            }
        }
    }
}

pub enum Layout {
    Text(String),
    Column(Vec<Layout>),
    SuperScript(Box<Layout>, Box<Layout>),
    Parentheses(Box<Layout>),
}

impl Layout {
    //布局显示后的宽度
    pub fn width(&self) -> usize {
        match self {
            Layout::Text(t) => t.width(),
            Layout::Column(t) => t.iter().map(|x| x.width()).sum(),
            Layout::SuperScript(a, b) => a.width() + b.width(),
            Layout::Parentheses(t) => t.width() + 2,
        }
    }

    //布局显示后的高度
    pub fn height(&self) -> usize {
        match self {
            Layout::Text(_) => 1,
            Layout::Column(t) => t.iter().map(|x| x.height()).max().unwrap_or(0),
            Layout::SuperScript(a, b) => a.height() + b.height(),
            Layout::Parentheses(t) => t.height(),
        }
    }

    pub fn paint(&self) -> Canvas {
        let width = self.width();
        let height = self.height();
        let mut canvas = Canvas::new(width, height);
        match self {
            Layout::Text(t) => {
                let chars: Vec<char> = t.chars().collect();
                let mut i = 0;
                while i < width {
                    match chars.get(i) {
                        Some(t) => {
                            canvas.0[0][i] = *t;
                            i += 1;
                        }
                        None => {
                            //这个时候 i 一定大于chars.len
                            canvas.0.remove(i);
                        }
                    }
                }
            }
            Layout::Column(t) => {
                let mut x = 0;
                for v in t.iter() {
                    canvas.paint_at(&v.paint(), x, height - v.height());
                    x += v.width();
                }
            }
            Layout::SuperScript(a, b) => {
                canvas.paint_at(&a.paint(), 0, height - a.height());
                canvas.paint_at(&b.paint(), a.width(), 0);
            }
            Layout::Parentheses(t) => {
                match height {
                    1 => {
                        canvas.0[0][0] = '(';
                        canvas.0[0][width - 1] = ')';
                    }
                    _ => {
                        canvas.0[0][0] = '⎛';
                        canvas.0[height - 1][0] = '⎝';
                        canvas.0[0][width - 1] = '⎞';
                        canvas.0[height - 1][width - 1] = '⎠';

                        for i in 1..height - 1 {
                            canvas.0[i][0] = '|';
                            canvas.0[i][width - 1] = '|';
                        }
                    }
                }
                canvas.paint_at(&t.paint(), 1, 0);
            }
        }
        canvas
    }
}

pub trait Print {
    fn to_layout(&self) -> Layout;

    fn print(&self) {
        let layout = self.to_layout();
        let canvas = layout.paint();
        for i in canvas.0 {
            println!("{}", i.iter().collect::<String>());
        }
    }
}

impl<C> Print for Expr<C>
where
    C: Display,
{
    fn to_layout(&self) -> Layout {
        match self {
            Expr::Const(t) => Layout::Text(t.to_string()),
            Expr::Symbol(t) => Layout::Text(t.to_string()),
            Expr::Add(t) => {
                let mut a = vec![];
                for (i, v) in t.iter().enumerate() {
                    if i > 0 && !v.to_string().starts_with("-") {
                        a.push(Layout::Text("+".to_string()));
                    }
                    a.push(v.to_layout());
                }
                Layout::Column(a)
            }
            Expr::Mul(t) => {
                let mut has_neg = false;
                let mut a = vec![];
                for v in t.iter() {
                    match v {
                        Expr::Const(t) if !has_neg && t.to_string() == "-1" => has_neg = true,
                        Expr::Add(_) => a.push(Layout::Parentheses(Box::new(v.to_layout()))),
                        _ => a.push(v.to_layout()),
                    }
                }
                for i in 0..a.len() - 1 {
                    a.insert(i * 2 + 1, Layout::Text("⋅".to_string()));
                }
                if has_neg {
                    a.insert(0, Layout::Text("-".to_string()));
                }
                Layout::Column(a)
            }
            Expr::Pow(a, b) => match &**a {
                Expr::Add(_) | Expr::Mul(_) => Layout::SuperScript(
                    Box::new(Layout::Parentheses(Box::new(a.to_layout()))),
                    Box::new(b.to_layout()),
                ),
                _ => Layout::SuperScript(Box::new(a.to_layout()), Box::new(b.to_layout())),
            },
        }
    }
}
