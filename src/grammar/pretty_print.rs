use crowbook_text_processing::escape;
use serde::Serialize;

use super::{Grammar, EPSILON};

struct ProductionOutput<'a> {
    left: &'a str,
    rights: Vec<Vec<&'a str>>,
}

impl ProductionOutput<'_> {
    fn to_plaintext(&self, left_width: usize) -> String {
        self.rights
            .iter()
            .map(|right| right.join(" "))
            .enumerate()
            .map(|(i, right)| {
                if i == 0 {
                    format!("{:>width$} -> {}", self.left, right, width = left_width)
                } else {
                    format!("{:>width$}  | {}", "", right, width = left_width)
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
    fn to_latex(&self) -> String {
        if self.rights.len() == 0 {
            return String::new();
        }

        std::iter::once(format!("{} & \\rightarrow &", escape::tex(self.left)))
            .chain(
                self.rights
                    .iter()
                    .map(|right| escape::tex(right.join(" ")).into_owned())
                    .collect::<Vec<_>>(),
            )
            .collect::<Vec<_>>()
            .join(" \\mid ")
    }
}

pub struct ProductionOutputVec<'a> {
    productions: Vec<ProductionOutput<'a>>,
}

impl ProductionOutputVec<'_> {
    pub fn to_plaintext(&self) -> String {
        let left_max_len = self.productions.iter().map(|p| p.left.len()).max().unwrap();
        self.productions
            .iter()
            .map(|s| s.to_plaintext(left_max_len))
            .collect::<Vec<String>>()
            .join("\n")
    }

    pub fn to_latex(&self) -> String {
        std::iter::once("\\begin{array}{cll}".to_string())
            .chain(self.productions.iter().map(|s| s.to_latex()))
            .chain(std::iter::once("\\end{array}".to_string()))
            .collect::<Vec<String>>()
            .join("\n")
    }
}

impl Grammar {
    pub fn to_production_output_vec(&self) -> ProductionOutputVec {
        let mut productions = Vec::new();
        for symbol in self.symbols.iter().skip(1) {
            // skip(1): skip epsilon
            if let Some(non_terminal) = symbol.non_terminal() {
                let mut rights = Vec::new();
                for production in &non_terminal.productions {
                    rights.push(
                        production
                            .iter()
                            .map(|idx| self.get_symbol_name(*idx))
                            .collect(),
                    );
                }
                productions.push(ProductionOutput {
                    left: non_terminal.name.as_str(),
                    rights,
                });
            }
        }
        ProductionOutputVec { productions }
    }
}

#[derive(Serialize)]
struct NonTerminalOutput<'a> {
    name: &'a str,
    nullable: bool,
    first: Vec<&'a str>,
    follow: Vec<&'a str>,
}

impl NonTerminalOutput<'_> {
    fn to_plaintext(&self) -> String {
        format!(
            "{} | {} | {} | {}",
            self.name,
            self.nullable,
            self.first.join(", "),
            self.follow.join(", ")
        )
    }
}

#[derive(Serialize)]
pub struct NonTerminalOutputVec<'a> {
    data: Vec<NonTerminalOutput<'a>>,
}

impl NonTerminalOutputVec<'_> {
    pub fn to_plaintext(&self) -> String {
        self.data
            .iter()
            .map(|s| s.to_plaintext())
            .collect::<Vec<String>>()
            .join("\n")
    }
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

impl Grammar {
    pub fn to_non_terminal_output_vec(&self) -> NonTerminalOutputVec {
        let mut data = Vec::new();
        for symbol in self.symbols.iter().skip(1) {
            // skip(1): skip epsilon
            if let Some(non_terminal) = symbol.non_terminal() {
                let mut t = NonTerminalOutput {
                    name: non_terminal.name.as_str(),
                    nullable: non_terminal.nullable,
                    first: non_terminal
                        .first
                        .iter()
                        .map(|idx| self.get_symbol_name(*idx))
                        .collect(),
                    follow: non_terminal
                        .follow
                        .iter()
                        .map(|idx| self.get_symbol_name(*idx))
                        .collect(),
                };
                if non_terminal.nullable {
                    t.first.push(EPSILON);
                }
                data.push(t);
            }
        }
        NonTerminalOutputVec { data }
    }
}