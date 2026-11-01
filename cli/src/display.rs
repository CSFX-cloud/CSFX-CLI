use colored::{Color, Colorize};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

fn is_light_background() -> bool {
    if let Ok(val) = std::env::var("COLORFGBG") {
        if let Some(bg) = val.split(';').last() {
            if let Ok(n) = bg.trim().parse::<u8>() {
                return n >= 8;
            }
        }
    }
    false
}

pub fn banner() {
    use std::io::Write;
    use std::thread::sleep;

    let lines = [
        "   ██████╗███████╗███████╗██╗  ██╗",
        "  ██╔════╝██╔════╝██╔════╝╚██╗██╔╝",
        "  ██║     ███████╗█████╗   ╚███╔╝ ",
        "  ██║     ╚════██║██╔══╝   ██╔██╗ ",
        "  ╚██████╗███████║██║     ██╔╝╚██╗",
        "   ╚═════╝╚══════╝╚═╝     ╚═╝  ╚═╝",
    ];

    let colors: [(u8, u8, u8); 6] = if is_light_background() {
        [
            (30, 30, 30),
            (60, 60, 60),
            (90, 90, 90),
            (120, 120, 120),
            (150, 150, 150),
            (180, 180, 180),
        ]
    } else {
        [
            (255, 255, 255),
            (220, 220, 220),
            (185, 185, 185),
            (150, 150, 150),
            (115, 115, 115),
            (80, 80, 80),
        ]
    };

    println!();
    for (line, (r, g, b)) in lines.iter().zip(colors.iter()) {
        println!(
            "{}",
            line.color(Color::TrueColor {
                r: *r,
                g: *g,
                b: *b
            })
            .bold()
        );
        std::io::stdout().flush().ok();
        sleep(Duration::from_millis(55));
    }

    println!(
        "\n  {}  {}\n",
        "Cloud Systems Fabric Xchange".bold(),
        format!("v{}", env!("CARGO_PKG_VERSION")).dimmed()
    );
}

pub fn spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

pub fn success(message: &str) {
    println!("{} {}", "✔".green().bold(), message);
}

pub fn error(message: &str) {
    eprintln!("{} {}", "✖".red().bold(), message);
}

pub fn warn(message: &str) {
    println!("{} {}", "⚠".yellow().bold(), message);
}

pub fn info(message: &str) {
    println!("{} {}", "●".cyan(), message);
}

pub fn section(title: &str) {
    println!("\n{}", title.bold().underline());
}

pub fn kv(key: &str, value: &str) {
    println!("  {:<20} {}", key.dimmed(), value);
}

pub fn kv_colored(key: &str, value: &str, color: Color) {
    println!("  {:<20} {}", key.dimmed(), value.color(color).bold());
}

pub struct Table {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    color_fn: Option<Box<dyn Fn(usize, &str) -> Color>>,
}

impl Table {
    pub fn new(headers: Vec<&str>) -> Self {
        Self {
            headers: headers.iter().map(|s| s.to_string()).collect(),
            rows: Vec::new(),
            color_fn: None,
        }
    }

    pub fn with_color<F>(mut self, f: F) -> Self
    where
        F: Fn(usize, &str) -> Color + 'static,
    {
        self.color_fn = Some(Box::new(f));
        self
    }

    pub fn add_row(&mut self, row: Vec<String>) {
        self.rows.push(row);
    }

    fn col_widths(&self) -> Vec<usize> {
        let mut widths: Vec<usize> = self.headers.iter().map(|h| h.len()).collect();
        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                if i < widths.len() {
                    widths[i] = widths[i].max(cell.len());
                }
            }
        }
        widths
    }

    pub fn print(&self) {
        let widths = self.col_widths();
        let total_width: usize = widths.iter().sum::<usize>() + widths.len() * 3;

        let header_line: String = self
            .headers
            .iter()
            .enumerate()
            .map(|(i, h)| format!("{:<w$}", h, w = widths[i]).bold().to_string())
            .collect::<Vec<_>>()
            .join("  ");
        println!("  {}", header_line);

        let sep = "─".repeat(total_width.min(120));
        println!("  {}", sep.dimmed());

        for row in &self.rows {
            let row_line: String = row
                .iter()
                .enumerate()
                .map(|(i, cell)| {
                    let w = widths.get(i).copied().unwrap_or(cell.len());
                    let padded = format!("{:<w$}", cell, w = w);
                    if let Some(ref f) = self.color_fn {
                        padded.color(f(i, cell)).to_string()
                    } else {
                        padded
                    }
                })
                .collect::<Vec<_>>()
                .join("  ");
            println!("  {}", row_line);
        }
    }
}

pub fn status_color(status: &str) -> Color {
    match status.to_lowercase().as_str() {
        "available" | "online" | "active" | "healthy" | "leader" => Color::Green,
        "creating" | "migrating" | "degraded" | "follower" => Color::Yellow,
        "error" | "offline" | "deleting" | "failed" => Color::Red,
        "inuse" | "in_use" => Color::Cyan,
        _ => Color::White,
    }
}
