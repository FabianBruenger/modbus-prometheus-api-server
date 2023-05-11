// use crate::clients as Clients;
use warp::{
    body::BodyDeserializeError, filters::cors::CorsForbidden, http::StatusCode, Rejection, Reply,
};
pub mod impls;
pub async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(error) = r.find::<CorsForbidden>() {
        log::error!("CorsForbidden: {:?}", error);
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::FORBIDDEN,
        ))
    } else if let Some(error) = r.find::<BodyDeserializeError>() {
        log::error!("BodyDeserializeError: {:?}", error);
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(impls::ErrorRuntime::ClientNotFound(client)) = r.find() {
        // Set return string
        let return_string = format!(
            "Client {} not found. Please check the client name.",
            client.as_ref().unwrap()
        );
        log::error!("{}", return_string);
        Ok(warp::reply::with_status(
            return_string,
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(impls::ErrorRuntime::ClientNotAbleToConnect(client)) = r.find() {
        // Set return string
        let return_string = format!(
            "Client {} could not be connected via modbus. Please check the connection.",
            client.as_ref().unwrap()
        );
        log::error!("{}", return_string);
        Ok(warp::reply::with_status(
            return_string,
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(impls::ErrorRuntime::ClientProtocolNotSupported) = r.find() {
        log::error!("ClientProtocolNotSupported");
        Ok(warp::reply::with_status(
            "The current client protocol is not supported. Supported protocols are: tcp"
                .to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(impls::ErrorRuntime::ClientRegisterDatatypeNotSupported) = r.find() {
        log::error!("ClientRegisterDatatypeNotSupported");
        Ok(warp::reply::with_status(
            "One of the registers does not have a supported datatype. Supported datatypes are: uint16, int16"
                .to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(impls::ErrorRuntime::ClientRegisterObjecttypeNotSupported) = r.find() {
        log::error!("ClientRegisterObjecttypeNotSupported");
        Ok(warp::reply::with_status(
            "One of the registers does not have a supported objecttype. Supported objecttypes are: input, holding"
                .to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(impls::ErrorRuntime::ClientCoilObjecttypeNotSupported) = r.find() {
        log::error!("ClientCoilObjecttypeNotSupported");
        Ok(warp::reply::with_status(
            "One of the coils does not have a supported objecttype. Supported objecttypes are: coil, discrete"
                .to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(impls::ErrorRuntime::ClientExists) = r.find() {
        log::error!("ClientExists");
        Ok(warp::reply::with_status(
                "Trying to create client, but a valid config for this client already exists. Please delete the client first.".to_string(),
                StatusCode::UNPROCESSABLE_ENTITY,
            ))
    } else if let Some(impls::ErrorRuntime::ClientJsonParseError) = r.find() {
        log::error!("ClientJsonParseError");
        Ok(warp::reply::with_status(
            "Trying to create client, but the local config is corrupted. Can not create the client"
                .to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(impls::ErrorRuntime::ClientRegisterNotFound(register)) = r.find() {
        let return_string = format!(
            "Register {} not found in client.",
            register.as_ref().unwrap()
        );
        log::error!("{}", return_string);
        Ok(warp::reply::with_status(
            return_string,
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(impls::ErrorRuntime::ClientRegisterNotInput(register)) = r.find() {
        let return_string = format!(
            "Register {} is not writable. It is an input register.",
            register.as_ref().unwrap()
        );
        log::error!("{}", return_string);
        Ok(warp::reply::with_status(
            return_string,
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(impls::ErrorRuntime::ClientRegisterWriteGenericError) = r.find() {
        log::error!("ClientRegisterWriteGenericError");
        Ok(warp::reply::with_status(
            "Generic Error while writing register".to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(impls::ErrorRuntime::ClientCoilNotFound(coil)) = r.find() {
        let return_string = format!("Coil {} not found in client.", coil.as_ref().unwrap());
        log::error!("{}", return_string);
        Ok(warp::reply::with_status(
            return_string,
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(impls::ErrorRuntime::ClientCoilNotInput(coil)) = r.find() {
        let return_string = format!(
            "Coil {} is not writable. It is an discrete coil.",
            coil.as_ref().unwrap()
        );
        log::error!("{}", return_string);
        Ok(warp::reply::with_status(
            return_string,
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(impls::ErrorRuntime::ClientCoilWriteGenericError) = r.find() {
        log::error!("ClientCoilWriteGenericError");
        Ok(warp::reply::with_status(
            "Generic Error while writing coil".to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(impls::ErrorRuntime::FSReadToStringError) = r.find() {
        log::error!("FSReadToStringError");
        Ok(warp::reply::with_status(
            "Cannot process local config file to string".to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(impls::ErrorRuntime::FSReadDirError) = r.find() {
        log::error!("FSReadDirError");
        Ok(warp::reply::with_status(
            "Cannot read configs from local FS. Please contact developer, this is a major issue!"
                .to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(impls::ErrorRuntime::FSDirEntryError) = r.find() {
        log::error!("FSDirEntryError");
        Ok(warp::reply::with_status(
            "Cannot process dir entry. Please contact developer, this is a major issue!"
                .to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(impls::ErrorRuntime::FSFileDeleteError) = r.find() {
        log::error!("FSFileDeleteError");
        Ok(warp::reply::with_status(
            "Cannot delete local config file.".to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(impls::ErrorRuntime::FSFileCreateError) = r.find() {
        log::error!("FSFileCreateError");
        Ok(warp::reply::with_status(
            "Cannot create local config file".to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(impls::ErrorRuntime::PrometheusErrorRegistry) = r.find() {
        log::error!("PrometheusErrorRegistry");
        Ok(warp::reply::with_status(
            "The Prometheus Registry couldnt be processed".to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(impls::ErrorRuntime::PrometheusErrorGaugeNew) = r.find() {
        log::error!("PrometheusErrorGaugeNew");
        Ok(warp::reply::with_status(
            "Could not create a new Gauge value".to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(impls::ErrorRuntime::PrometheusErrorRegistryRegister) = r.find() {
        log::error!("PrometheusErrorRegistryRegister");
        Ok(warp::reply::with_status(
            "Could not register the new Gauge value to the registry".to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(impls::ErrorRuntime::PrometheusErrorEncoder) = r.find() {
        log::error!("PrometheusErrorEncoder");
        Ok(warp::reply::with_status(
            "The Prometheus Encoder couldnt be processed".to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(impls::ErrorRuntime::PrometheusErrorGaugeRemove) = r.find() {
        log::error!("PrometheusErrorGaugeRemove");
        Ok(warp::reply::with_status(
            "Could not get the target Gauge value to be removed".to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(impls::ErrorRuntime::PrometheusErrorRegistryUnregister) = r.find() {
        log::error!("PrometheusErrorRegistryUnregister");
        Ok(warp::reply::with_status(
            "Cannot unregister the Gauge value from registry".to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(impls::ErrorRuntime::RegexError) = r.find() {
        log::error!("RegexError");
        Ok(warp::reply::with_status(
            "No valid String is provided. Please check the fields name, register.name if you just have: lowercase, number or underscores".to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(impls::ErrorRuntime::JSONSerializeError) = r.find() {
        log::error!("JSONSerializeError");
        Ok(warp::reply::with_status(
            "Cannot serialize the provided JSON for creating local config file".to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(impls::ErrorRuntime::ValueNotParsableToU16(value)) = r.find() {
        let return_string = format!(
            "Value {} is not parsable. Please provide a number between 0 and 65535",
            value.as_ref().unwrap()
        );
        log::error!("{}", return_string);
        Ok(warp::reply::with_status(
            return_string,
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(impls::ErrorRuntime::ValueNotParsableToBool(value)) = r.find() {
        let return_string = format!(
            "Value {} is not parsable. Please provide either: true or false",
            value.as_ref().unwrap()
        );
        log::error!("{}", return_string);
        Ok(warp::reply::with_status(
            return_string,
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(impls::ErrorRuntime::ClientRegisterWriteError(value)) = r.find() {
        let return_string = format!(
            "Register {} is not writable. Please check the error logs for more information",
            value.as_ref().unwrap()
        );
        log::error!("{}", return_string);
        Ok(warp::reply::with_status(
            return_string,
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else {
        log::error!("Unknown Error");
        Ok(warp::reply::with_status(
            "Route not found".to_string(),
            StatusCode::NOT_FOUND,
        ))
    }
}
