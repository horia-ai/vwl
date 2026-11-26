use ratatui::text::Line;

fn main() {
    let line = Line::from("VWL Terminal User Interface (Ratatui Shell - Gate 3)");
    println!("{}", line);
    println!("Supported Views: [Home] [Market] [Projects] [Node] [Inbox]");
}
