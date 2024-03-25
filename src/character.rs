use std::hash::{Hash, Hasher};

use once_cell::sync::Lazy;
use regex::Regex;

pub static BLANKA: Lazy<CharacterId> = Lazy::new(|| CharacterId::new(0, "Blanka", r"(?i)(blanka)"));
pub static CAMMY: Lazy<CharacterId> = Lazy::new(|| CharacterId::new(1, "Cammy", r"(?i)(camm?y)"));
pub static CHUN_LI: Lazy<CharacterId> = Lazy::new(|| CharacterId::new(2, "Chun-Li", r"(?i)(chun([-\s]?li)?)"));
pub static DEE_JAY: Lazy<CharacterId> = Lazy::new(|| CharacterId::new(3, "Dee Jay", r"(?i)(d(ee?)?\s?j(ay)?)"));
pub static DHALSIM: Lazy<CharacterId> = Lazy::new(|| CharacterId::new(4, "Dhalsim", r"(?i)(sim)"));
pub static HONDA: Lazy<CharacterId> = Lazy::new(|| CharacterId::new(5, "Honda", r"(?i)(honda)"));
pub static GUILE: Lazy<CharacterId> = Lazy::new(|| CharacterId::new(6, "Guile", r"(?i)(guile)"));
pub static JAMIE: Lazy<CharacterId> = Lazy::new(|| CharacterId::new(7, "Jamie", r"(?i)(jamie)"));
pub static JP: Lazy<CharacterId> = Lazy::new(|| CharacterId::new(8, "JP", r"(?i)(jp)"));
pub static JURI: Lazy<CharacterId> = Lazy::new(|| CharacterId::new(9, "Juri", r"(?i)(juri)"));
pub static KEN: Lazy<CharacterId> = Lazy::new(|| CharacterId::new(10, "Ken", r"(?i)(ken)"));
pub static KIMBERLY: Lazy<CharacterId> = Lazy::new(|| CharacterId::new(11, "Kimberly", r"(?i)(kim)"));
pub static LILY: Lazy<CharacterId> = Lazy::new(|| CharacterId::new(12, "Lily", r"(?i)(lily)"));
pub static LUKE: Lazy<CharacterId> = Lazy::new(|| CharacterId::new(13, "Luke", r"(?i)(luke)"));
pub static MANON: Lazy<CharacterId> = Lazy::new(|| CharacterId::new(14, "Manon", r"(?i)(manon)"));
pub static MARISA: Lazy<CharacterId> = Lazy::new(|| CharacterId::new(15, "Marisa", r"(?i)(marisa)"));
pub static RYU: Lazy<CharacterId> = Lazy::new(|| CharacterId::new(16, "Ryu", r"(?i)(ryu)"));
pub static ZANGIEF: Lazy<CharacterId> = Lazy::new(|| CharacterId::new(17, "Zangief", r"(?i)(gief)"));
pub static RASHID: Lazy<CharacterId> = Lazy::new(|| CharacterId::new(18, "Rashid", r"(?i)(rashid)"));
pub static AKI: Lazy<CharacterId> = Lazy::new(|| CharacterId::new(19, "A.K.I.", r"(?i)(a.?k.?i.?)"));
pub static ED: Lazy<CharacterId> = Lazy::new(|| CharacterId::new(20, "Ed", r"(?i)(ed)"));

static CHARACTER_IDS: Lazy<[&CharacterId; 21]> = Lazy::new(|| [
    &BLANKA, &CAMMY, &CHUN_LI, &DEE_JAY, &DHALSIM, &HONDA, &GUILE, &JAMIE, &JP, &JURI, &KEN,
    &KIMBERLY, &LILY, &LUKE, &MANON, &MARISA, &RYU, &ZANGIEF, &RASHID, &AKI, &ED
]);

pub fn get_character_regex<'a>(name: String) -> Option<&'a CharacterId> {
    (*CHARACTER_IDS).into_iter().find(|&character| character.matcher.is_match(&name))
}

#[derive(Clone, Debug)]
pub struct CharacterId {
    pub id: u8,
    pub name: String,
    pub matcher: Regex,
    pub data_url: String,
}

#[derive(Clone, Debug)]
pub struct Character {
    pub character_id: CharacterId,
    pub moves: Vec<Move>
}

impl PartialEq<Self> for CharacterId {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for CharacterId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Eq for CharacterId {}

impl CharacterId {
    fn new(id: u8, name: &str, matcher: &str) -> Self {
        let name_sanitized = name.replace(" ", "_");
        let data_url = format!("https://wiki.supercombo.gg/w/Street_Fighter_6/{}/Data", name_sanitized);
        CharacterId {
            id,
            name: name.to_string(),
            matcher: Regex::new(&format!(r"(?i)^({})$", matcher)).unwrap(),
            data_url
        }
    }
}

#[derive(Clone, Debug)]
pub struct Move {
    pub name: String,
    pub input: String,
    pub input_matcher: Regex,
    pub startup: String,
    pub active: String,
    pub recovery: String,
    pub cancel: String,
    pub damage: String,
    pub guard: String,
    pub invuln: String,
    pub armour: String,
    pub on_hit: String,
    pub on_block: String,
    pub hitbox_image_url: String,
    pub notes: String,
}

pub mod loader {
    use std::error::Error;

    use once_cell::sync::Lazy;
    use regex::Regex;
    use scraper::{Element, ElementRef, Selector};
    use scraper::selectable::Selectable;
    use tokio::task::JoinSet;

    use crate::character::{Character, CHARACTER_IDS, CharacterId, Move};

    pub(crate) async fn load_characters() -> Vec<Character> {
        let mut characters = vec![];
        let mut join_set = JoinSet::new();
        for character_id in (*CHARACTER_IDS).into_iter() {
            join_set.spawn(load_character(character_id));
        }
        while let Some(res) = join_set.join_next().await {
            let Ok(character_opt) = res else {
                println!("Error handling character creation future: {}", res.unwrap_err());
                continue;
            };
            let Some(character) = character_opt else {
                println!("Error loading character");
                continue;
            };
            characters.push(character);
        }
        characters
    }

    async fn load_character(character_id: &CharacterId) -> Option<Character> {
        let Ok(moves) = load_moves(character_id).await else {
            return None;
        };
        let character = Character {
            character_id: character_id.clone(),
            moves,
        };
        Some(character)
    }

    pub(crate) async fn load_moves(character: &CharacterId) -> Result<Vec<Move>, Box<dyn Error>> {
        let mut moves = vec![];
        let body = request_body(character).await?;
        let document = scraper::Html::parse_document(&body);
        let block_select = document.select(&BLOCK_SELECTOR);
        for ele in block_select {
            match parse_block(ele) {
                Ok(move_loaded) => {
                    moves.push(move_loaded);
                }
                Err(err) => {
                    println!("skipping move for: {:?}: {:?}", character.name, err);
                }
            }
            // if let Some(move_loaded) = parse_block(ele) {
            //     moves.push(move_loaded);
            // } else {
            //     let input = ele.select(&INPUT_SELECTOR).next().map(|e| e.inner_html());
            //     println!("skipping move for: {:?}: {:?}", character.name, input);
            // }
        }
        Ok(moves)
    }

    static BLOCK_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse("div > div > section.section-collapsible > table.wikitable > tbody").unwrap());

    async fn request_body(character: &CharacterId) -> Result<String, Box<dyn Error>> {
        let body = reqwest::get(&character.data_url).await?.text().await?;
        Ok(body)
    }

    static INPUT_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse("tr > th > div > p > span").unwrap());
    static NAME_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse("tr > th > div > div").unwrap());
    static HITBOX_IMAGE_ELEMENT_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse("tr > th > a").unwrap());
    static HITBOX_IMAGE_URL_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(/images/thumb\S+) 2x").unwrap());
    static DATA_ROW_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse("tr > td").unwrap());
    const DEFAULT_IMAGE: &str = "https://wiki.supercombo.gg/images/thumb/4/42/SF6_Logo.png/300px-SF6_Logo.png";

    pub(crate) fn parse_block(block: ElementRef) -> Result<Move, &str> {
        let Some(input) = block.select(&INPUT_SELECTOR)
            .next()
            .map(|e| e.inner_html()) else {
            return Err("no input");
        };
        let regex = {
            let r = input.replace(".", ".?");
            let r = r.replace(" ", "");
            let mut r = r.replace("~", "\\~");
            r = format!("(?i)^({})$", r);
            r
        };
        let Ok(regex) = Regex::new(&regex) else {
            return Err("regex load err");
        };
        let Some(name) = block.select(&NAME_SELECTOR)
            .next()
            .map(|e| e.inner_html()) else {
            return Err("no name");
        };
        // need to initialize this as its own variable first since 'e' is consumed
        let mut select = block.select(&HITBOX_IMAGE_ELEMENT_SELECTOR).map(|e| e.html());
        let hitbox_image_url = {
            let image = select.next().and_then(|s| hitbox_image_matcher(s));
            let hitbox = select.next().and_then(|s| hitbox_image_matcher(s));
            hitbox.or(image).unwrap_or_else(|| {
                println!("no image, using default");
                DEFAULT_IMAGE.to_string()
            })
        };
        let mut data = block.select(&DATA_ROW_SELECTOR)
            .map(|e| get_lowest_child(e))
            .map(|e| e.inner_html())
            .collect::<Vec<String>>()
            .into_iter();
        let damage = data.next().unwrap_or_else(|| String::from("-"));
        let guard = data.nth(2).unwrap_or_else(|| String::from("-"));
        let cancel = data.next().unwrap_or_else(|| String::from("-"));
        let startup = data.nth(1).unwrap_or_else(|| String::from("-"));
        let active = data.next().unwrap_or_else(|| String::from("-"));
        let recovery = data.next().unwrap_or_else(|| String::from("-"));
        let invuln = data.nth(9).unwrap_or_else(|| String::from("-"));
        let armour = data.next().unwrap_or_else(|| String::from("-"));
        let on_hit = data.nth(10).unwrap_or_else(|| String::from("-"));
        let on_block = data.next().unwrap_or_else(|| String::from("-"));
        let notes = data.next().unwrap_or_else(|| String::from("-"));

        let move_constructed = Move {
            name,
            input,
            input_matcher: regex,
            startup,
            active,
            recovery,
            cancel,
            damage,
            guard,
            invuln,
            armour,
            on_hit,
            on_block,
            hitbox_image_url,
            notes,
        };
        Ok(move_constructed)
    }

    fn get_lowest_child(parent: ElementRef) -> ElementRef {
        match parent.first_element_child() {
            None => parent,
            Some(child) => get_lowest_child(child)
        }
    }

    fn hitbox_image_matcher(element: String) -> Option<String> {
        HITBOX_IMAGE_URL_REGEX.captures(element.as_str())
            .and_then(|caps| caps.get(1))// skip first match: is whole match
            .map(|m| m.as_str().to_string())
            .map(|s| format!("https://wiki.supercombo.gg/{}", s))
    }
}