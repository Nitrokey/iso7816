use crate::Data;

impl Default for Status {
    fn default() -> Self {
        Status::Success
    }
}

// I0x6985SO/IEC 7816-4, 5.1.3 "Status bytes"
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Status {
    //////////////////////////////
    // Normal processing (90, 61)
    //////////////////////////////
    /// 9000
    Success,

    /// 61XX
    MoreAvailable(u8),

    ///////////////////////////////
    // Warning processing (62, 63)
    ///////////////////////////////

    // 62XX: state of non-volatile memory unchanged (cf. SW2)

    // 63XX: state of non-volatile memory changed (cf. SW2)
    VerificationFailed,
    RemainingRetries(u8),

    ////////////////////////////////
    // Execution error (64, 65, 66)
    ////////////////////////////////

    // 64XX: persistent memory unchanged (cf. SW2)
    UnspecifiedNonpersistentExecutionError,

    // 65XX: persistent memory changed (cf. SW2)
    UnspecifiedPersistentExecutionError,

    // 66XX: security related issues

    ///////////////////////////////
    // Checking error (67 - 6F)
    ///////////////////////////////

    // 6700: wrong length, no further indication
    WrongLength,

    // 68XX: functions in CLA not supported (cf. SW2)
    LogicalChannelNotSupported,
    SecureMessagingNotSupported,
    CommandChainingNotSupported,

    // 69xx: command not allowed (cf. SW2)
    SecurityStatusNotSatisfied,
    ConditionsOfUseNotSatisfied,
    OperationBlocked,

    // 6Axx: wrong parameters P1-P2 (cf. SW2)
    IncorrectDataParameter,
    FunctionNotSupported,
    NotFound,
    NotEnoughMemory,
    IncorrectP1OrP2Parameter,
    KeyReferenceNotFound,

    // 6BXX: wrong parameters P1-P2

    // 6CXX: wrong Le field, SW2 encodes available bytes

    // 6D00: instruction code not supported or invalid
    InstructionNotSupportedOrInvalid,

    // 6E00: class not supported
    ClassNotSupported,

    // 6F00: no precise diagnosis
    UnspecifiedCheckingError,
}

impl TryFrom<(u8, u8)> for Status {
    type Error = u16;
    #[inline]
    fn try_from(sw: (u8, u8)) -> Result<Self, Self::Error> {
        let (sw1, sw2) = sw;
        Ok(match u16::from_be_bytes([sw1, sw2]) {
            0x6300 => Self::VerificationFailed,
            sw @ 0x63c0..=0x63cf => Self::RemainingRetries((sw as u8) & 0xf),

            0x6400 => Self::UnspecifiedNonpersistentExecutionError,
            0x6500 => Self::UnspecifiedPersistentExecutionError,

            0x6700 => Self::WrongLength,

            0x6881 => Self::LogicalChannelNotSupported,
            0x6882 => Self::SecureMessagingNotSupported,
            0x6884 => Self::CommandChainingNotSupported,

            0x6982 => Self::SecurityStatusNotSatisfied,
            0x6985 => Self::ConditionsOfUseNotSatisfied,
            0x6983 => Self::OperationBlocked,

            0x6a80 => Self::IncorrectDataParameter,
            0x6a81 => Self::FunctionNotSupported,
            0x6a82 => Self::NotFound,
            0x6a84 => Self::NotEnoughMemory,
            0x6a86 => Self::IncorrectP1OrP2Parameter,
            0x6a88 => Self::KeyReferenceNotFound,

            0x6d00 => Self::InstructionNotSupportedOrInvalid,
            0x6e00 => Self::ClassNotSupported,
            0x6f00 => Self::UnspecifiedCheckingError,

            0x9000 => Self::Success,
            sw @ 0x6100..=0x61FF => Self::MoreAvailable(sw as u8),
            other => return Err(other),
        })
    }
}

impl From<Status> for u16 {
    #[inline]
    fn from(status: Status) -> u16 {
        use Status::*;
        match status {
            VerificationFailed => 0x6300,
            RemainingRetries(x) => {
                assert!(x < 16);
                u16::from_be_bytes([0x63, 0xc0 + x])
            }

            UnspecifiedNonpersistentExecutionError => 0x6400,
            UnspecifiedPersistentExecutionError => 0x6500,

            WrongLength => 0x6700,

            LogicalChannelNotSupported => 0x6881,
            SecureMessagingNotSupported => 0x6882,
            CommandChainingNotSupported => 0x6884,

            SecurityStatusNotSatisfied => 0x6982,
            ConditionsOfUseNotSatisfied => 0x6985,
            OperationBlocked => 0x6983,

            IncorrectDataParameter => 0x6a80,
            FunctionNotSupported => 0x6a81,
            NotFound => 0x6a82,
            NotEnoughMemory => 0x6a84,
            IncorrectP1OrP2Parameter => 0x6a86,
            KeyReferenceNotFound => 0x6a88,

            InstructionNotSupportedOrInvalid => 0x6d00,
            ClassNotSupported => 0x6e00,
            UnspecifiedCheckingError => 0x6f00,

            Success => 0x9000,
            MoreAvailable(x) => u16::from_be_bytes([0x61, x]),
        }
    }
}

impl From<Status> for [u8; 2] {
    #[inline]
    fn from(status: Status) -> [u8; 2] {
        let sw: u16 = status.into();
        sw.to_be_bytes()
    }
}

impl<const S: usize> From<Status> for Data<S> {
    #[inline]
    fn from(status: Status) -> Data<S> {
        let arr: [u8; 2] = status.into();
        Data::from_slice(&arr).unwrap()
    }
}
