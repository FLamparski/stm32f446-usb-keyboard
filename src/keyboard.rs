use usbd_hid_device::HidReport;
use arrayvec::ArrayVec;

/**
 * Much of the code here has been, uh, adapted from the Arduino keyboard libraries:
 * https://github.com/arduino-libraries/Keyboard
 * https://github.com/NicoHood/HID
 */

pub struct KeyboardReport {
    //# Layout:
    //# ```
    //# { modifiers: u8, keys: [u8; 6] }
    //# ```
    bytes: [u8; 7]
}

impl KeyboardReport {
    fn new(modifiers: u8, keys: [u8; 6]) -> Self {
        let mut report = Self { bytes: [0; 7] };
        report.bytes[0] = modifiers;
        for (i, k) in keys.iter().enumerate() {
            report.bytes[i + 1] = *k;
        }
        report
    }

    pub fn all_released() -> Self {
        Self::new(0, [0; 6])
    }

    pub fn keys_pressed(modifiers: u8, keys: [u8; 6]) -> Self {
        Self::new(modifiers, keys)
    }
}

pub struct Keyboard {
    modifiers: u8,
    keys: ArrayVec<[u8; 6]>,
}

impl Keyboard {
    pub fn new() -> Self {
        Self {
            modifiers: 0,
            keys: ArrayVec::<[u8; 6]>::new(),
        }
    }

    pub fn press(&mut self, ch: u8) -> Result<(), KeyboardError> {
        if self.keys.len() >= 6 {
            return Err(KeyboardError::TooManyKeys);
        }

        let mut modifiers = self.modifiers;
        let k = if ch >= 136 {
            ch - 136 // non-printing, not a modifier
        } else if ch >= 128 {
            modifiers |= 1 << (ch - 128); // modifier
            0
        } else { // 'normal' character
            let mut k = ASCII_MAP[ch as usize];
            if k & SHIFT != 0 { // handle uppercase characters
                modifiers |= 0x02;
                k &= 0x7F;
            }
            k
        };

        self.modifiers = modifiers;
        if !self.keys.contains(&k) {
            self.keys.push(k);
        }
        Ok(())
    }

    pub fn release_all(&mut self) -> Result<(), KeyboardError> {
        self.modifiers = 0;
        self.keys.clear();
        Ok(())
    }

    pub fn release(&mut self, ch: u8) -> Result<(), KeyboardError> {
        let mut modifiers = self.modifiers;
        let k = if ch >= 136 {
            ch - 136 // non-printing, not a modifier
        } else if ch >= 128 {
            modifiers &= !(1 << (ch - 128)); // modifier
            0
        } else { // 'normal' character
            let mut k = ASCII_MAP[ch as usize];
            if k & SHIFT != 0 { // handle uppercase characters
                modifiers &= !0x02;
                k &= 0x7F;
            }
            k
        };

        if k != 0 && self.keys.contains(&k) {
            self.modifiers = modifiers;
            for (i, pk) in self.keys.iter().enumerate() {
                if pk == &k {
                    self.keys.swap_remove(i);
                    break
                }
            }
        }

        Ok(())
    }

    pub fn get_report(&self) -> KeyboardReport {
        if self.modifiers == 0 && self.keys.len() == 0 {
            KeyboardReport::all_released()
        } else {
            let mut keys = [0; 6];
            for (i, k) in self.keys.iter().enumerate() {
                keys[i] = *k;
            }
            KeyboardReport::keys_pressed(self.modifiers, keys)
        }
    }
}

#[derive(Debug)]
pub enum KeyboardError {
    TooManyKeys,
}

impl AsRef<[u8]> for KeyboardReport {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

impl HidReport for KeyboardReport {
    const DESCRIPTOR: &'static [u8] = &[
        //  Keyboard
        0x05, 0x01,                    // USAGE_PAGE (Generic Desktop)  // 47
        0x09, 0x06,                    // USAGE (Keyboard)
        0xa1, 0x01,                    // COLLECTION (Application)
        
        0x05, 0x07,                    //   USAGE_PAGE (Keyboard)
        0x19, 0xe0,                    //   USAGE_MINIMUM (Keyboard LeftControl)
        0x29, 0xe7,                    //   USAGE_MAXIMUM (Keyboard Right GUI)
        0x15, 0x00,                    //   LOGICAL_MINIMUM (0)
        0x25, 0x01,                    //   LOGICAL_MAXIMUM (1)
        0x75, 0x01,                    //   REPORT_SIZE (1)
        0x95, 0x08,                    //   REPORT_COUNT (8)
        0x81, 0x02,                    //   INPUT (Data,Var,Abs)

        0x05, 0x07,                    //   USAGE_PAGE (Keyboard)
        0x19, 0x00,                    //   USAGE_MINIMUM (Reserved (no event indicated))
        0x29, 0xE7,                    //   USAGE_MAXIMUM (Keyboard Right GUI)
        0x15, 0x00,                    //   LOGICAL_MINIMUM (0)
        0x26, 0xE7, 0x00,              //   LOGICAL_MAXIMUM (231)
        0x95, 0x06,                    //   REPORT COUNT (6)
        0x75, 0x08,                    //   REPORT SIZE (8)
        0x81, 0x00,                    //   INPUT (Data,Ary,Abs)

        0xc0,                          // END_COLLECTION
    ];
}

const SHIFT: u8 = 0x80;
static ASCII_MAP: &[u8] = &[
    0x00,             // NUL
    0x00,             // SOH
    0x00,             // STX
    0x00,             // ETX
    0x00,             // EOT
    0x00,             // ENQ
    0x00,             // ACK
    0x00,             // BEL
    0x2a,             // BS Backspace
    0x2b,             // TAB Tab
    0x28,             // LF Enter
    0x00,             // VT 
    0x00,             // FF 
    0x00,             // CR 
    0x00,             // SO 
    0x00,             // SI 
    0x00,             // DEL
    0x00,             // DC1
    0x00,             // DC2
    0x00,             // DC3
    0x00,             // DC4
    0x00,             // NAK
    0x00,             // SYN
    0x00,             // ETB
    0x00,             // CAN
    0x00,             // EM
    0x00,             // SUB
    0x00,             // ESC
    0x00,             // FS
    0x00,             // GS
    0x00,             // RS
    0x00,             // US

    0x2c,          // ' '
    0x1e|SHIFT,    // !
    0x34|SHIFT,    // "
    0x20|SHIFT,    // #
    0x21|SHIFT,    // $
    0x22|SHIFT,    // %
    0x24|SHIFT,    // &
    0x34,          // '
    0x26|SHIFT,    // (
    0x27|SHIFT,    // )
    0x25|SHIFT,    // *
    0x2e|SHIFT,    // +
    0x36,          // ,
    0x2d,          // -
    0x37,          // .
    0x38,          // /
    0x27,          // 0
    0x1e,          // 1
    0x1f,          // 2
    0x20,          // 3
    0x21,          // 4
    0x22,          // 5
    0x23,          // 6
    0x24,          // 7
    0x25,          // 8
    0x26,          // 9
    0x33|SHIFT,    // :
    0x33,          // ;
    0x36|SHIFT,    // <
    0x2e,          // =
    0x37|SHIFT,    // >
    0x38|SHIFT,    // ?
    0x1f|SHIFT,    // @

    0x04|SHIFT,      // A
    0x05|SHIFT,      // B
    0x06|SHIFT,      // C
    0x07|SHIFT,      // D
    0x08|SHIFT,      // E
    0x09|SHIFT,      // F
    0x0a|SHIFT,      // G
    0x0b|SHIFT,      // H
    0x0c|SHIFT,      // I
    0x0d|SHIFT,      // J
    0x0e|SHIFT,      // K
    0x0f|SHIFT,      // L
    0x10|SHIFT,      // M
    0x11|SHIFT,      // N
    0x12|SHIFT,      // O
    0x13|SHIFT,      // P
    0x14|SHIFT,      // Q
    0x15|SHIFT,      // R
    0x16|SHIFT,      // S
    0x17|SHIFT,      // T
    0x18|SHIFT,      // U
    0x19|SHIFT,      // V
    0x1a|SHIFT,      // W
    0x1b|SHIFT,      // X
    0x1c|SHIFT,      // Y
    0x1d|SHIFT,      // Z

    0x2f,          // [
    0x31,          // bslash
    0x30,          // ]
    0x23|SHIFT,    // ^
    0x2d|SHIFT,    // _
    0x35,          // `
    0x04,          // a
    0x05,          // b
    0x06,          // c
    0x07,          // d
    0x08,          // e
    0x09,          // f
    0x0a,          // g
    0x0b,          // h
    0x0c,          // i
    0x0d,          // j
    0x0e,          // k
    0x0f,          // l
    0x10,          // m
    0x11,          // n
    0x12,          // o
    0x13,          // p
    0x14,          // q
    0x15,          // r
    0x16,          // s
    0x17,          // t
    0x18,          // u
    0x19,          // v
    0x1a,          // w
    0x1b,          // x
    0x1c,          // y
    0x1d,          // z
    0x2f|SHIFT,    // {
    0x31|SHIFT,    // |
    0x30|SHIFT,    // }
    0x35|SHIFT,    // ~
    0              // DEL
];
