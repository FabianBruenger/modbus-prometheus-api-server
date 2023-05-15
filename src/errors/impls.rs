use warp::reject::Reject;

#[derive(Debug)]
// Enums for non warp errors which end up in a rejection
pub enum ErrorRuntime {
    ClientNotFound(Option<String>),
    ClientNotAbleToConnect(Option<String>),
    ClientProtocolNotSupported,
    ClientRegisterDatatypeNotSupported,
    ClientRegisterObjecttypeNotSupported,
    ClientCoilObjecttypeNotSupported,
    ClientExists,
    ClientJsonParseError, // used when clients are created on init
    ClientRegisterNotFound(Option<String>),
    ClientRegisterNotWritable(Option<String>),
    ClientRegisterWriteGenericError,
    ClientCoilNotFound(Option<String>),
    ClientCoilNotInput(Option<String>),
    ClientCoilWriteGenericError,
    FSReadToStringError,
    FSReadDirError,
    FSDirEntryError,
    FSFileDeleteError,
    FSFileCreateError,
    PrometheusErrorRegistry,
    PrometheusErrorGaugeNew,
    PrometheusErrorRegistryRegister,
    PrometheusErrorEncoder,
    PrometheusErrorGaugeRemove,
    PrometheusErrorRegistryUnregister,
    RegexError,
    JSONSerializeError,
    ValueNotParsableToU16(Option<String>),
    ValueNotParsableToBool(Option<String>),
    ClientRegisterWriteError(Option<String>),
    NoParametersProvided,
}
impl Reject for ErrorRuntime {}
#[derive(Debug)]
pub enum ErrorRuntimeNoRejection {
    InvalidIpAddress,
    CouldNotConnect,
}