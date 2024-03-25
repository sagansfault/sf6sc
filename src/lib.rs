use crate::character::{Character, CharacterId, Move};

pub mod character;

pub async fn load_supercombo_data() -> SuperComboData {
    let characters = character::loader::load_characters().await;
    let data = SuperComboData {
        characters,
    };
    data
}

#[derive(Debug)]
pub struct SuperComboData {
    characters: Vec<Character>
}

impl SuperComboData {
    pub fn get_character_search(&self, query: &str) -> Option<&Character> {
        for character in &self.characters {
            if character.character_id.matcher.is_match(query) {
                return Some(&character);
            }
        }
        None
    }

    pub fn get_character(&self, character_id: &CharacterId) -> Option<&Character> {
        for character in &self.characters {
            if character.character_id == *character_id {
                return Some(&character);
            }
        }
        None
    }

    pub fn get_moves(&self, character_id: &CharacterId) -> Option<&Vec<Move>> {
        for character in &self.characters {
            if character.character_id == *character_id {
                return Some(&character.moves);
            }
        }
        None
    }

    pub fn get_move_by_input(&self, character_id: &CharacterId, input_query: &str) -> Option<&Move> {
        let moves = self.get_moves(character_id)?;
        for move_i in moves {
            if move_i.input_matcher.is_match(input_query) {
                return Some(move_i);
            }
        }
        None
    }

    pub fn get_character_move_by_input(&self, character_query: &str, input_query: &str) -> Option<&Move> {
        let character = self.get_character_search(character_query)?;
        self.get_move_by_input(&character.character_id, input_query)
    }
}

#[tokio::test]
async fn test() {
    let data = load_supercombo_data().await;
    let _cammy = data.get_moves(&character::CAMMY).unwrap();
    // for m in cammy.into_iter() {
    //     println!("{:?} :: {:?}", m.input, m.input_matcher);
    // }
    // println!("{:?}", data.get_character_move_by_input("ryu", "214HP"));
}
