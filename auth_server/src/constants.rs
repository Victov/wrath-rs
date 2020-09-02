#[allow(dead_code)]
pub enum AuthResult
{
    Success = 0x00,
    FailBanned = 0x03,
    FailUnknownAccount = 0x04,
    FailIncorrectPassword = 0x05,
    FailAlreadyOnline = 0x06,
    FailNoGameTime = 0x07,
    FailDatabaseBusy = 0x08,
    FailVersionInvalid = 0x09,
    FailVersionUpdateAvailable = 0x0A,
    FailInvalidServer = 0x0B,
    FailSuspended = 0x0C,
    FailNoAccess = 0x0D,
    SuccessSurvey = 0x0E,
    FailParentalControl = 0x0F,
    FailLockedEnforced = 0x10,
    FailTrialEnded = 0x11,
    FailUseBattlenet = 0x12,
    FailAntiIndulgence = 0x13,
    FailExpired = 0x14,
    FailNoGameAccount = 0x15,
    FailChargeBack = 0x16,
    FailInternetGameRoomWithoutBattleNet = 0x17,
    FailGameAccountLocked = 0x18,
    FailUnlockableLock = 0x19,
    FailConversionRequired = 0x20,
    FailDisconnected = 0xFF
}

#[allow(dead_code)]
pub enum RealmFlags
{
    None                              = 0x00,
    Invalid                           = 0x01,
    Offline                           = 0x02,
    SpecificBuild                     = 0x04,
    Unknown1                          = 0x08,
    Unknown2                          = 0x10,
    Recommended                       = 0x20,
    New                               = 0x40,
    Full                              = 0x80
}

