use bincode::{Decode, Encode};

/// Type of slot within a cell
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode)]
pub enum SlotType {
    Interior,
    Exterior,
}

impl SlotType {
    pub fn to_string(&self) -> &'static str {
        match self {
            SlotType::Interior => "interior",
            SlotType::Exterior => "exterior",
        }
    }

    pub fn from_string(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "interior" => Some(SlotType::Interior),
            "exterior" => Some(SlotType::Exterior),
            _ => None,
        }
    }
}

/// Position of a unit slot within a cell
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode)]
pub struct SlotPosition {
    pub slot_type: SlotType,
    pub index: usize,
}

impl SlotPosition {
    pub fn new(slot_type: SlotType, index: usize) -> Self {
        Self { slot_type, index }
    }

    pub fn interior(index: usize) -> Self {
        Self {
            slot_type: SlotType::Interior,
            index,
        }
    }

    pub fn exterior(index: usize) -> Self {
        Self {
            slot_type: SlotType::Exterior,
            index,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slot_type_conversion() {
        assert_eq!(SlotType::Interior.to_string(), "interior");
        assert_eq!(SlotType::Exterior.to_string(), "exterior");
        assert_eq!(SlotType::from_string("interior"), Some(SlotType::Interior));
        assert_eq!(SlotType::from_string("exterior"), Some(SlotType::Exterior));
        assert_eq!(SlotType::from_string("invalid"), None);
    }

    #[test]
    fn test_slot_position() {
        let pos = SlotPosition::interior(5);
        assert_eq!(pos.slot_type, SlotType::Interior);
        assert_eq!(pos.index, 5);

        let pos2 = SlotPosition::exterior(3);
        assert_eq!(pos2.slot_type, SlotType::Exterior);
        assert_eq!(pos2.index, 3);
    }
}
