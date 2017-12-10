use klc::*;
use keylayout::*;

pub fn convert(win_kbd: WinKeyLayout, id: i32) -> KeyLayout {
    let WinKeyLayout {
        name,
        layout,
        deadkeys,
        keynames_dead,
        ..
    } = win_kbd;


    let mut key_maps: Vec<_> = (0..9).map(|i| KeyMap {
        index: i,
        key: Vec::new(),
    }).collect();
    let mut actions = vec![];
    let mut terminators = vec![];
    KeyLayout {
        group: 126,
        id,
        name,
        layouts: Layouts {
            layout: vec![
                Layout {
                    first: 0,
                    last: 255,
                    map_set: "a".to_owned(),
                    modifiers: "mods".to_owned(),
                }
            ]
        },
        modifier_map: ModifierMap {
            id: "mods".to_owned(),
            default_index: 0,
            key_map_select: vec![
                KeyMapSelect {
                    map_index: 0,
                    modifier: vec![
                        Modifier{
                            keys: "".to_owned(),
                        },
                        Modifier{
                            keys: "command anyShift? caps?".to_owned(),
                        },
                    ]
                },
                KeyMapSelect {
                    map_index: 1,
                    modifier: vec![
                        Modifier{
                            keys: "caps".to_owned(),
                        },
                    ]
                },
                KeyMapSelect {
                    map_index: 2,
                    modifier: vec![
                        Modifier{
                            keys: "anyShift caps?".to_owned(),
                        },
                    ]
                },
                KeyMapSelect {
                    map_index: 3,
                    modifier: vec![
                        Modifier{
                            keys: "anyOption".to_owned(),
                        },
                    ]
                },
                KeyMapSelect {
                    map_index: 4,
                    modifier: vec![
                        Modifier{
                            keys: "anyOption caps".to_owned(),
                        },
                    ]
                },
                KeyMapSelect {
                    map_index: 5,
                    modifier: vec![
                        Modifier{
                            keys: "anyOption anyShift caps?".to_owned(),
                        },
                    ]
                },
                KeyMapSelect {
                    map_index: 6,
                    modifier: vec![
                        Modifier{
                            keys: "command anyOption caps?".to_owned(),
                        },
                    ]
                },
                KeyMapSelect {
                    map_index: 7,
                    modifier: vec![
                        Modifier{
                            keys: "command anyOption anyShift caps?".to_owned(),
                        },
                    ]
                },
                KeyMapSelect {
                    map_index: 8,
                    modifier: vec![
                        Modifier{
                            keys: "control command? anyOption? anyShift? caps?".to_owned(),
                        },
                    ]
                }
            ]
        },
        key_map_set: KeyMapSet {
            id: "a".to_owned(),
            key_map: key_maps
        },
        actions: Actions {
            action: actions
        },
        terminators: Terminators {
            when: terminators
        },
    }
}
