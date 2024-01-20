use ratatui::buffer::Buffer;
use ratatui::widgets::Widget;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, List};

#[derive(Debug, Clone)]
pub enum Ast {
    Block(String, Box<Ast>),
    List(Vec<String>),
}

impl Widget for &Ast {
    fn render(self, area: Rect, buf: &mut Buffer){
        match self {
            Ast::Block(str, cont) => {
                let block = Block::default()
                    .title(str.as_str())
                    .borders(Borders::ALL);
                let inner_area = block.inner(area);
                block.render(area, buf);
                cont.render(inner_area, buf);
            },
            Ast::List(items) => {
                List::new(
                    items.iter().map(|x| x.as_str())
                ).render(area, buf);
            }
        }
    }
}
