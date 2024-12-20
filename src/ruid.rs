use rand::RngCore;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use chrono::Utc;

use crate::prefix::prefix;

use std::fmt;

#[derive(Debug, Clone)]
pub struct Ruid {
    id: u128, // IDを保持する
}

impl Ruid {
    pub fn to_string(&self) -> String {
        format!("{:032x}", self.id) // u128を16進数文字列に変換
    }

    pub fn to_u128(&self) -> u128 {
        self.id
    }

    pub fn edit_prefix(&self, prefix: u16) -> Ruid {
        let id = (self.id & !(0xFFFF << RuidGenerator::PREFIX_SHIFT))
            | ((prefix as u128) << RuidGenerator::PREFIX_SHIFT);
        Ruid { id }
    }

    pub fn from_u128(id: u128) -> Self {
        Ruid { id }
    }

    pub fn from_str(s: &str) -> Result<Self, ParseRuidError> {
        if s.len() != 32 {
            return Err(ParseRuidError::InvalidLength);
        }

        let id = u128::from_str_radix(s, 16).map_err(|_| ParseRuidError::InvalidFormat)?;
        Ok(Ruid { id })
    }
}

// `FromStr` 実装用のエラー型
#[derive(Debug)]
pub enum ParseRuidError {
    InvalidLength,
    InvalidFormat,
}

impl fmt::Display for ParseRuidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseRuidError::InvalidLength => write!(f, "Ruid文字列の長さが無効です"),
            ParseRuidError::InvalidFormat => write!(f, "Ruid文字列の形式が無効です"),
        }
    }
}

impl std::error::Error for ParseRuidError {}


pub struct RuidGenerator {
    default_device_id: u16,
    prefix: u16,
    device_id: u16,
    rng: ChaCha20Rng, // ChaCha20Rng
}

impl RuidGenerator {
    const PREFIX_SHIFT: u8 = 112;
    const VERSION_CODE_SHIFT: u8 = 108; // 4 bits
    const DEVICE_ID_SHIFT: u8 = 92;
    const TIMESTAMP_SHIFT: u8 = 44;
    const VERSION_CODE: u8 = 0x1; // バージョンコードv1

    pub fn new() -> Self {
        let rng = ChaCha20Rng::from_entropy();
        Self {
            default_device_id: 0x0000,
            prefix: prefix::UNCATEGORIZED_DATA,
            device_id: 0x0000,
            rng,
        }
    }

    pub fn set_seed(mut self, seed: [u8; 32]) -> Self {
        let rng = ChaCha20Rng::from_seed(seed);
        self.rng = rng;
        self
    }

    pub fn set_default_device_id(mut self, device_id: u16) -> Self {
        self.default_device_id = device_id;
        self
    }

    pub fn set_prefix(mut self, prefix: u16) -> Self {
        self.prefix = prefix;
        self
    }

    pub fn set_device_id(mut self, device_id: u16) -> Self {
        self.device_id = device_id;
        self
    }

    pub fn generate(&mut self) -> Ruid {
        let prefix = self.prefix;
        let device_id = self.device_id;

        let timestamp = (Utc::now().timestamp_micros() as u64) & ((1u64 << 48) - 1);
        let rand = self.rng.next_u64() & ((1u64 << 44) - 1);

        let id = Self::generator(prefix, device_id, timestamp, rand);

        // オプションの値をリセット
        self.prefix = prefix::UNCATEGORIZED_DATA;
        self.device_id = self.default_device_id;

        Ruid { id }
    }

    pub fn generator(prefix: u16, device_id: u16, timestamp: u64, rand: u64) -> u128 {
        let id = (prefix as u128) << Self::PREFIX_SHIFT
            | (Self::VERSION_CODE as u128) << Self::VERSION_CODE_SHIFT
            | (device_id as u128) << Self::DEVICE_ID_SHIFT
            | (timestamp as u128) << Self::TIMESTAMP_SHIFT
            | (rand as u128);

        id
    }
}
