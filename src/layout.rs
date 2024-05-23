use keyberon::action::{k, l, m, Action::*, HoldTapAction, HoldTapConfig};
use keyberon::key_code::KeyCode::*;

type Action = keyberon::action::Action<()>;

const TIMEOUT: u16 = 180;
const CUT: Action = m(&[LShift, Delete].as_slice());
const COPY: Action = m(&[LCtrl, Insert].as_slice());
const PASTE: Action = m(&[LShift, Insert].as_slice());
const L1_SP: Action = HoldTap(&HoldTapAction {
    timeout: TIMEOUT,
    tap_hold_interval: 0,
    config: HoldTapConfig::Default,
    hold: l(1),
    tap: k(Space),
});
const CSP: Action = m(&[LCtrl, Space].as_slice());
const NBSP: Action = m(&[RAlt, LShift, Space].as_slice());
const STAB: Action = m(&[LShift, Tab].as_slice());
const AL_SH: Action = m(&[RAlt, LShift].as_slice());
const CT_ES: Action = HoldTap(&HoldTapAction {
    timeout: TIMEOUT,
    tap_hold_interval: 0,
    config: HoldTapConfig::Default,
    hold: k(LCtrl),
    tap: k(Escape),
});

macro_rules! s {
    ($k:ident) => {
        m(&[LShift, $k].as_slice())
    };
}
macro_rules! a {
    ($k:ident) => {
        m(&[RAlt, $k].as_slice())
    };
}

#[rustfmt::skip]
pub static LAYERS: keyberon::layout::Layers<16, 5, 1,  ()> = keyberon::layout::layout! {
    { //[+* ***+*** ***+*** ***+*** ***+*** ***+*** ******* ******* ******* ***+*** ******* ***+*** ***+*** ***+*** ***+*** ***+],
        [Escape     1       2       3       4       5       n      n        n       n       6       7       8         9       0       Minus ],
        [Tab        Q       W       E       R       T       n      n        n       n       Y       U       I         O       P       Equal ],
        [CapsLock   A       S       D       F       G       n      n        n       n       H       J       K         L     SColon    Quote ],
        [LCtrl      Z       X       C       V       B    LGui     LGui     LGui    LGui     N       M     Comma      Dot    Slash     Bslash],
        [  n        n    LShift    LAlt   Space  LCtrl   LAlt     Tab     Space   BSpace RShift  BSpace  Delete     RAlt    n           n   ], 
    }
};
