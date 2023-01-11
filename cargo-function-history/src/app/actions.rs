use std::{
    collections::HashMap,
    fmt::{self, Display},
    slice::Iter,
};

use crate::keys::Key;

/// We define all available action
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Action {
    Quit,
    TextEdit,
    ScrollUp,
    ScrollDown,
    BackCommit,
    ForwardCommit,
    BackFile,
    ForwardFile,
}

impl Action {
    /// All available actions
    pub fn iterator() -> Iter<'static, Action> {
        static ACTIONS: [Action; 8] = [
            Action::Quit,
            Action::TextEdit,
            Action::ScrollUp,
            Action::ScrollDown,
            Action::BackCommit,
            Action::ForwardCommit,
            Action::BackFile,
            Action::ForwardFile,
        ];
        ACTIONS.iter()
    }

    /// List of key associated to action
    pub fn keys(&self) -> &[Key] {
        match self {
            Action::Quit => &[Key::Ctrl('c'), Key::Char('q')],
            Action::TextEdit => &[Key::Char(':'), Key::Shift(':')],
            Action::ScrollUp => &[Key::Up, Key::Char('k')],
            Action::ScrollDown => &[Key::Down, Key::Char('j')],
            Action::BackCommit => &[Key::Left, Key::Char('h')],
            Action::ForwardCommit => &[Key::Right, Key::Char('l')],
            Action::BackFile => &[Key::Shiftleft, Key::Char('H'), Key::Shift('h')],
            Action::ForwardFile => &[Key::Shiftright, Key::Char('L'), Key::Shift('l')],
        }
    }
}

/// Could display a user friendly short description of action
impl Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Action::Quit => "Quit",
                Action::TextEdit => "TextEdit",
                Action::ScrollUp => "ScrollUp",
                Action::ScrollDown => "ScrollDown",
                Action::BackCommit => "BackCommit",
                Action::ForwardCommit => "ForwardCommit",
                Action::BackFile => "BackFile",
                Action::ForwardFile => "ForwardFile",
            }
        )
    }
}

/// The application should have some contextual actions.
#[derive(Default, Debug, Clone)]
pub struct Actions(Vec<Action>);

impl Actions {
    /// Given a key, find the corresponding action
    pub fn find(&self, key: Key) -> Option<&Action> {
        log::debug!("{}", key);
        Action::iterator()
            .filter(|action| self.0.contains(action))
            .find(|action| action.keys().contains(&key))
    }

    /// Get contextual actions.
    /// (just for building a help view)
    pub fn actions(&self) -> &[Action] {
        self.0.as_slice()
    }
}

impl From<Vec<Action>> for Actions {
    /// Build contextual action
    ///
    /// # Panics
    ///
    /// If two actions have same key
    fn from(actions: Vec<Action>) -> Self {
        // Check key unicity
        let mut map: HashMap<Key, Vec<Action>> = HashMap::new();
        for action in actions.iter() {
            for key in action.keys().iter() {
                match map.get_mut(key) {
                    Some(vec) => vec.push(*action),
                    None => {
                        map.insert(*key, vec![*action]);
                    }
                }
            }
        }
        let errors = map
            .iter()
            .filter(|(_, actions)| actions.len() > 1) // at least two actions share same shortcut
            .map(|(key, actions)| {
                let actions = actions
                    .iter()
                    .map(Action::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("Conflict key {key} with actions {actions}")
            })
            .collect::<Vec<_>>();
        if !errors.is_empty() {
            panic!("{}", errors.join("; "))
        }

        // Ok, we can create contextual actions
        Self(actions)
    }
}
