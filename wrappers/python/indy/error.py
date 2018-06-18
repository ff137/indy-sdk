from enum import IntEnum


class ErrorCode(IntEnum):
    Success = 0,

    # Common errors

    # Caller passed invalid value as param 1 (null, invalid json and etc..)
    CommonInvalidParam1 = 100,

    # Caller passed invalid value as param 2 (null, invalid json and etc..)
    CommonInvalidParam2 = 101,

    # Caller passed invalid value as param 3 (null, invalid json and etc..)
    CommonInvalidParam3 = 102,

    # Caller passed invalid value as param 4 (null, invalid json and etc..)
    CommonInvalidParam4 = 103,

    # Caller passed invalid value as param 5 (null, invalid json and etc..)
    CommonInvalidParam5 = 104,

    # Caller passed invalid value as param 6 (null, invalid json and etc..)
    CommonInvalidParam6 = 105,

    # Caller passed invalid value as param 7 (null, invalid json and etc..)
    CommonInvalidParam7 = 106,

    # Caller passed invalid value as param 8 (null, invalid json and etc..)
    CommonInvalidParam8 = 107,

    # Caller passed invalid value as param 9 (null, invalid json and etc..)
    CommonInvalidParam9 = 108,

    # Caller passed invalid value as param 10 (null, invalid json and etc..)
    CommonInvalidParam10 = 109,

    # Caller passed invalid value as param 11 (null, invalid json and etc..)
    CommonInvalidParam11 = 110,

    # Caller passed invalid value as param 12 (null, invalid json and etc..)
    CommonInvalidParam12 = 111,

    # Invalid library state was detected in runtime. It signals library bug
    CommonInvalidState = 112,

    # Object (json, config, key, credential and etc...) passed by library caller has invalid structure
    CommonInvalidStructure = 113,

    # IO Error
    CommonIOError = 114,

    # Wallet errors
    # Caller passed invalid wallet handle
    WalletInvalidHandle = 200,

    # Unknown type of wallet was passed on create_wallet
    WalletUnknownTypeError = 201,

    # Attempt to register already existing wallet type
    WalletTypeAlreadyRegisteredError = 202,

    # Attempt to create wallet with name used for another exists wallet
    WalletAlreadyExistsError = 203,

    # Requested entity id isn't present in wallet
    WalletNotFoundError = 204,

    # Trying to use wallet with pool that has different name
    WalletIncompatiblePoolError = 205,

    # Trying to open wallet that was opened already
    WalletAlreadyOpenedError = 206,

    # Input provided to wallet operations is considered not valid
    WalletAccessFailed = 207,

    # Attempt to open encrypted wallet with invalid credentials
    WalletInputError = 208,

    # Decoding of wallet data during input/output failed
    WalletDecodingError = 209,

    # Storage error occurred during wallet operation
    WalletStorageError = 210,

    # Error during encryption-related operations
    WalletEncryptonError = 211,

    # Requested wallet item not found
    WalletItemNotFound = 212,

    # Returned if wallet's add_record operation is used with record name that already exists
    WalletItemAlreadyExists = 213,

    # Returned if provided wallet query is invalid
    WalletQueryError = 214,

    # Ledger errors
    # Trying to open pool ledger that wasn't created before
    PoolLedgerNotCreatedError = 300,

    # Caller passed invalid pool ledger handle
    PoolLedgerInvalidPoolHandle = 301,

    # Pool ledger terminated
    PoolLedgerTerminated = 302,

    # No concensus during ledger operation
    LedgerNoConsensusError = 303,

    # Attempt to parse invalid transaction response
    LedgerInvalidTransaction = 304,

    # Attempt to send transaction without the necessary privileges
    LedgerSecurityError = 305,

    # Attempt to create pool ledger config with name used for another existing pool
    PoolLedgerConfigAlreadyExistsError = 306,

    # Timeout for action
    PoolLedgerTimeout = 307,

    # Attempt to open Pool for witch Genesis Transactions are not compatible with set Protocol version
    PoolIncompatibleProtocolVersion = 308,

    # Revocation registry is full and creation of new registry is necessary
    AnoncredsRevocationRegistryFullError = 400,

    AnoncredsInvalidUserRevocId = 401,

    # Attempt to generate master secret with dupplicated name
    AnoncredsMasterSecretDuplicateNameError = 404,

    AnoncredsProofRejected = 405,

    AnoncredsCredentialRevoked = 406,

    # Attempt to create credential definition with duplicated did schema pair
    AnoncredsCredDefAlreadyExistsError = 407,

    # Crypto errors
    # Unknown format of DID entity keys
    UnknownCryptoTypeError = 500,

    # Attempt to create duplicate did
    DidAlreadyExistsError = 600,

    # Unknown payment method was given
    PaymentUnknownMethodError = 700,

    # No method were scraped from inputs/outputs or more than one were scraped
    PaymentIncompatibleMethodsError = 701,

    # Insufficient funds on inputs
    PaymentInsufficientFundsError = 702


class IndyError(Exception):
    # error_code: ErrorCode

    def __init__(self, error_code: ErrorCode):
        self.error_code = error_code
