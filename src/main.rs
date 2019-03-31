use std::collections::{HashMap, HashSet};
use std::env;
use std::error;
use std::fmt;
use std::fs;

type Result<T> = std::result::Result<T, Box<error::Error>>;

#[derive(Debug)]
enum Error {
    Usage,
    Format,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl error::Error for Error {
    fn cause(&self) -> Option<&error::Error> {
        Some(self)
    }
}

fn usage() {
    println!("usage: vcfdedup <vcf>");
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct VcardEntry {
    lines: Vec<String>,
}

impl VcardEntry {
    fn new() -> Self {
        VcardEntry { lines: Vec::new() }
    }

    fn push(&mut self, s: &str) {
        assert!(s.starts_with(' ') || self.lines.is_empty());
        self.lines.push(s.to_owned());
    }

    fn print(&self) {
        for l in &self.lines {
            println!("{}", l);
        }
    }
}

struct Vcard {
    version: String,
    content: HashSet<VcardEntry>,
}

impl Vcard {
    fn new() -> Self {
        Vcard {
            version: String::new(),
            content: HashSet::new(),
        }
    }

    fn set_version(&mut self, s: &str) {
        self.version = s.to_owned();
    }

    fn insert(&mut self, e: VcardEntry) {
        self.content.insert(e);
    }

    fn extend(&mut self, other: &Vcard) {
        for e in &other.content {
            self.content.insert(e.clone());
        }
    }

    fn get(&self, key: &str) -> Option<&VcardEntry> {
        self.content.iter().find(|e| e.lines[0].starts_with(key))
    }

    fn print(&self) {
        for e in &self.content {
            e.print();
        }
    }
}

struct Parser {
    cards: Vec<Vcard>,
    cur_card: Option<Vcard>,

    lines: Vec<String>,
    cur_idx: usize,
}

impl Parser {
    fn new(stream: String) -> Self {
        Parser {
            cards: Vec::new(),
            cur_card: None,

            lines: stream.lines().map(|s| s.to_string()).collect(),
            cur_idx: 0,
        }
    }

    fn parse(mut self) -> Result<Vec<Vcard>> {
        while self.cur_idx < self.lines.len() {
            self.vcard()?;
        }
        Ok(self.cards)
    }

    fn vcard(&mut self) -> Result<()> {
        self.begin()?;
        while !self.end()? {
            self.entry()?;
        }
        Ok(())
    }

    fn begin(&mut self) -> Result<()> {
        match self.lines[self.cur_idx].as_str() {
            "BEGIN:VCARD" => {
                if self.cur_card.is_some() {
                    return Err(Box::new(Error::Format));
                }

                eprintln!("New card at line {}", self.cur_idx + 1);
                self.cur_card = Some(Vcard::new());
                self.cur_idx += 1;
                self.version()?;
                Ok(())
            }
            _ => Err(Box::new(Error::Format)),
        }
    }

    fn version(&mut self) -> Result<()> {
        if self.lines[self.cur_idx].starts_with("VERSION:") {
            self.cur_card
                .as_mut()
                .unwrap()
                .set_version(self.lines[self.cur_idx].as_str());
            self.cur_idx += 1;
            Ok(())
        } else {
            Err(Box::new(Error::Format))
        }
    }

    fn end(&mut self) -> Result<bool> {
        match self.lines[self.cur_idx].as_str() {
            "END:VCARD" => {
                self.cards.push(self.cur_card.take().unwrap());
                self.cur_idx += 1;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn entry(&mut self) -> Result<()> {
        let mut entry = VcardEntry::new();
        entry.push(self.lines[self.cur_idx].as_str());
        self.cur_idx += 1;
        while self.lines[self.cur_idx].starts_with(' ') {
            entry.push(self.lines[self.cur_idx].as_str());
            self.cur_idx += 1;
        }
        self.cur_card.as_mut().unwrap().insert(entry);
        Ok(())
    }
}

fn main() -> Result<()> {
    let infile = env::args().nth(1);
    if infile.is_none() {
        usage();
        return Err(Box::new(Error::Usage));
    }

    let cards = {
        let input = fs::read_to_string(infile.unwrap())?;
        let parser = Parser::new(input);
        parser.parse()?
    };

    let mut collection: HashMap<VcardEntry, Vcard> = HashMap::new();
    for c in cards {
        if let Some(name) = c.get("N") {
            collection
                .entry(name.clone())
                .and_modify(|e| e.extend(&c))
                .or_insert(c);
        }
    }

    for c in collection.values() {
        println!("BEGIN:VCARD");
        println!("{}", c.version);
        c.print();
        println!("END:VCARD");
    }

    Ok(())
}
