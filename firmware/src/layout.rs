//! Here we define the keyboard layout and layers for [keyberon](https://github.com/TeXitoi/keyberon)
//! using the new layout macro.

use keyberon::layout::{layout, Layers};

use crate::{NUM_COLS, NUM_LAYERS, NUM_ROWS};

pub static LAYERS: Layers<NUM_COLS, NUM_ROWS, NUM_LAYERS> = layout! {
    {
        // TODO: adjust default lines
        [ Escape   F1   F2   F3 F4 F5 F6    F7 F8 n F9   F10  F11 F12    ]
        [ '`'      1    2    3  4  5  6     7  8  9 0    -    =   BSpace ]
        [ Tab      Q    W    E  R  T  Y     U  I  O P    '['  ']' '\\'   ]
        [ CapsLock A    S    D  F  G  H     J  K  L ;    '\'' n   Enter  ]
        [ LShift   n    Z    X  C  V  B     N  M  , .    /    n   RShift ]
        [ LCtrl    LGui LAlt n  n  n  Space n  n  n RAlt RGui (1) RCtrl  ]
    }
    {
        [ t t VolDown VolUp Mute MediaPlayPause MediaStop MediaPreviousSong MediaNextSong t t t t t ]
        [ t t t       t     t    t              t         t                 t             t t t t t ]
        [ t t t       t     t    t              t         t                 t             t t t t t ]
        [ t t t       t     t    t              t         t                 t             t t t t t ]
        [ t t t       t     t    t              t         t                 t             t t t t t ]
        [ t t t       t     n    t              t         t                 t             t t t t t ]
    }
};
