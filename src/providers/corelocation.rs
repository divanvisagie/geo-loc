use crate::location::Location;
use chrono::{TimeZone, Utc};
use objc::declare::ClassDecl;
use objc::rc::Id;
use objc::runtime::{Class, Object, Sel};
use objc::{class, msg_send, sel, sel_impl};
use std::error::Error;
use std::ffi::CStr;
use std::fmt;
use std::os::raw::c_void;
use std::ptr;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum CoreLocationError {
    AuthorizationDenied,
    Timeout,
    Failed(String),
}

impl fmt::Display for CoreLocationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreLocationError::AuthorizationDenied => {
                write!(f, "CoreLocation authorization denied")
            }
            CoreLocationError::Timeout => write!(f, "Timed out waiting for CoreLocation fix"),
            CoreLocationError::Failed(reason) => write!(f, "CoreLocation error: {reason}"),
        }
    }
}

impl Error for CoreLocationError {}

struct Fix {
    latitude: f64,
    longitude: f64,
    accuracy: f64,
    timestamp: f64,
}

struct DelegateState {
    tx: Mutex<Option<oneshot::Sender<Result<Fix, CoreLocationError>>>>,
}

impl DelegateState {
    fn new(tx: oneshot::Sender<Result<Fix, CoreLocationError>>) -> Self {
        Self {
            tx: Mutex::new(Some(tx)),
        }
    }

    fn send(&self, value: Result<Fix, CoreLocationError>) {
        if let Some(sender) = self.tx.lock().ok().and_then(|mut guard| guard.take()) {
            let _ = sender.send(value);
        }
    }
}

#[repr(C)]
struct CLLocationCoordinate2D {
    latitude: f64,
    longitude: f64,
}

fn delegate_class() -> &'static Class {
    static CLASS: OnceLock<&'static Class> = OnceLock::new();
    CLASS.get_or_init(|| unsafe {
        let superclass = class!(NSObject);
        let mut decl = ClassDecl::new("GeoLocCLDelegate", superclass)
            .expect("failed to declare GeoLocCLDelegate class");
        decl.add_ivar::<*mut c_void>("state");
        decl.add_method(
            sel!(locationManager:didUpdateLocations:),
            update as extern "C" fn(&mut Object, Sel, *mut Object, *mut Object),
        );
        decl.add_method(
            sel!(locationManager:didFailWithError:),
            fail as extern "C" fn(&mut Object, Sel, *mut Object, *mut Object),
        );
        decl.add_method(sel!(dealloc), dealloc as extern "C" fn(&mut Object, Sel));
        decl.register()
    })
}

unsafe fn take_state(this: &Object) -> Option<&'static DelegateState> {
    let ptr: *mut c_void = *this.get_ivar("state");
    if ptr.is_null() {
        None
    } else {
        Some(&*(ptr as *mut DelegateState))
    }
}

unsafe extern "C" fn update(
    this: &mut Object,
    _: Sel,
    manager: *mut Object,
    locations: *mut Object,
) {
    if let Some(state) = take_state(this) {
        let count: usize = msg_send![locations, count];
        if count == 0 {
            return;
        }
        let location_obj: *mut Object = msg_send![locations, lastObject];
        if location_obj.is_null() {
            return;
        }
        let coordinate: CLLocationCoordinate2D = msg_send![location_obj, coordinate];
        let accuracy: f64 = msg_send![location_obj, horizontalAccuracy];
        let timestamp_obj: *mut Object = msg_send![location_obj, timestamp];
        let timestamp: f64 = msg_send![timestamp_obj, timeIntervalSince1970];
        state.send(Ok(Fix {
            latitude: coordinate.latitude,
            longitude: coordinate.longitude,
            accuracy,
            timestamp,
        }));
    }
    let _: () = msg_send![manager, stopUpdatingLocation];
}

unsafe extern "C" fn fail(this: &mut Object, _: Sel, manager: *mut Object, error: *mut Object) {
    if let Some(state) = take_state(this) {
        let description: *mut Object = msg_send![error, localizedDescription];
        let c_string: *const std::os::raw::c_char = msg_send![description, UTF8String];
        let reason = if c_string.is_null() {
            "unknown".to_string()
        } else {
            CStr::from_ptr(c_string).to_string_lossy().into_owned()
        };
        state.send(Err(CoreLocationError::Failed(reason)));
    }
    let _: () = msg_send![manager, stopUpdatingLocation];
}

unsafe extern "C" fn dealloc(this: &mut Object, _: Sel) {
    let ptr: *mut c_void = *this.get_ivar("state");
    if !ptr.is_null() {
        drop(Box::from_raw(ptr as *mut DelegateState));
        this.set_ivar("state", ptr::null_mut());
    }
    let superclass = (*this).class().superclass().unwrap();
    let _: () = msg_send![super(this, superclass), dealloc];
}

pub async fn get_current_location(
    timeout: Duration,
    verbose: bool,
) -> Result<Location, Box<dyn Error>> {
    let timeout = if timeout.is_zero() {
        Duration::from_secs(5)
    } else {
        timeout
    };

    let (tx, rx) = oneshot::channel();

    unsafe {
        if verbose {
            eprintln!("geo-loc: requesting CoreLocation fix");
        }

        let services_enabled: bool = msg_send![class!(CLLocationManager), locationServicesEnabled];
        if !services_enabled {
            return Err(Box::new(CoreLocationError::Failed(
                "Location services disabled".into(),
            )));
        }

        let status: i32 = msg_send![class!(CLLocationManager), authorizationStatus];
        if status == 2 || status == 1 {
            return Err(Box::new(CoreLocationError::AuthorizationDenied));
        }

        let manager_ptr: *mut Object = msg_send![class!(CLLocationManager), alloc];
        let manager_ptr: *mut Object = msg_send![manager_ptr, init];
        if manager_ptr.is_null() {
            return Err(Box::new(CoreLocationError::Failed(
                "Failed to create CLLocationManager".into(),
            )));
        }
        let delegate_class = delegate_class();
        let delegate_ptr: *mut Object = msg_send![delegate_class, alloc];
        let delegate_ptr: *mut Object = msg_send![delegate_ptr, init];
        if delegate_ptr.is_null() {
            return Err(Box::new(CoreLocationError::Failed(
                "Failed to allocate delegate".into(),
            )));
        }

        let state = Box::new(DelegateState::new(tx));
        let state_ptr = Box::into_raw(state) as *mut c_void;
        (*delegate_ptr).set_ivar("state", state_ptr);

        let manager: Id<Object> = Id::from_ptr(manager_ptr);
        let delegate: Id<Object> = Id::from_ptr(delegate_ptr);
        let _: () = msg_send![&*manager, setDelegate: &*delegate];
        let _: () = msg_send![&*manager, requestWhenInUseAuthorization];
        let _: () = msg_send![&*manager, startUpdatingLocation];

        let result = tokio::time::timeout(timeout, rx).await;
        match result {
            Ok(Ok(Ok(fix))) => {
                let secs = fix.timestamp.floor();
                let mut nanos = ((fix.timestamp - secs) * 1_000_000_000.0) as i64;
                let mut secs = secs as i64;
                if nanos < 0 {
                    nanos += 1_000_000_000;
                    secs -= 1;
                }
                let timestamp = Utc
                    .timestamp_opt(secs, nanos as u32)
                    .single()
                    .unwrap_or_else(|| Utc::now());
                let accuracy = if fix.accuracy.is_sign_positive() {
                    Some(fix.accuracy)
                } else {
                    None
                };
                Ok(Location::new(
                    fix.latitude,
                    fix.longitude,
                    accuracy,
                    "corelocation",
                    timestamp,
                ))
            }
            Ok(Ok(Err(err))) => Err(Box::new(err)),
            Ok(Err(_canceled)) => Err(Box::new(CoreLocationError::Failed(
                "CoreLocation channel closed".into(),
            ))),
            Err(_) => {
                let _: () = msg_send![&*manager, stopUpdatingLocation];
                Err(Box::new(CoreLocationError::Timeout))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delegate_class_is_singleton() {
        let first = delegate_class() as *const Class as usize;
        let second = delegate_class() as *const Class as usize;
        assert_eq!(first, second);
    }
}
