use crate::http::{HttpRequest, StatusCode};
use std::sync::Arc;

#[cfg(feature = "serde")]
use serde::de::DeserializeOwned;

#[cfg(feature = "serde")]
use serde::Serialize;

/// Used to understand conceptually the flow that grazie uses to manage web requests.
/// Requests to an `HttpServer` are handled like so:
/// `<from client> -> dyn Unpacker -> dyn Seeder[0..n] -> <to dyn Router>`
///
/// An `Unpacker` is an object which reads a byte stream from a TCP socket and transforms it into
/// an HttpRequest object. The body of an `Unpacker` is only passed along as a series of
/// heap-allocated bytes, the `Unpacker` does not parse the request body, it only packages it into a
/// `BoxBody` to be passed between different middlewares.
///
/// A `Seeder` is an object which transforms this data. Many `Seeder`s can exist before a `Router`
/// is hit. A `Seeder` can also be used within a `Router`. `Seeder`s can also be used as Route
/// guards, preventing access to a `Route`/`Router` unless the `Seeder` has verified a set of
/// criteria.
///
/// Management of all transformed data is handled within the `<dyn Router>` handler. The `Router`
/// is responsible for matching the HttpRequest against a series of internal routes, and providing
/// the HttpRequest to the correct route.
pub trait Unpacker {
    /// Unpacks a TCP stream and opens up a request object, parsing the contents of the body into
    /// the request.
    fn unpack(&self, stream: &mut [u8]) -> impl Future<Output = HttpRequest<BoxBody>> + Send;
}

/// Holds the accessibility state of a route, as handled by a `Seeder` acting as a route guard
/// any series of `Route`s/`Router`s.
///
/// `Guard::Accessible(&HttpRequest<BoxBody>)` is used to pass along a successful route check to
/// the `HttpServer`. This makes allows the request to access the requested resource.
///
/// `Guard::Inaccessible { reason, status_code }` is used to pass along a route check that was
/// unsuccessful, containing the request body, reason, and status code to use for the rejected
/// request. A `Seeder` can be used to catch `Inaccessible` requests and create a new response body.
/// This can be used for things like providing error codes, error traces, messages, standardized
/// API responses, and more.
pub enum Guard<'a, T> {
    /// A successful Guard check was met, and the request chain will continue to the requested
    /// accessible route.
    Accessible(&'a T),

    /// An unsuccessful Guard check was met, and the request chain will only continue on to
    /// `Seeder`s which accept inaccessible guards.
    Inaccessible {
        request: &'a T,
        reason: Option<&'static str>,
        status_code: StatusCode,
    },
}

/// An object representing some heap-allocated HTTP request's body.
///
/// This internally holds an atomically reference-counted pointer to the data,
/// allowing for packing/unpacking the data into a desired data type. This also allows for multiple
/// accessors to the inner data.
///
/// By default, the requested data type must implement `TryFrom<&[u8]>`, which makes conversion
/// to/from a string easy, however on other data types this might be more involved.
///
/// Luckily, `grazie` comes with some features enabling `serde` serialization and deserialization
/// from the request body. This can be utilized to open the request body into the desired type a bit
/// easier, and also makes handling of different raw content types easier.
pub struct BoxBody {
    /// The pointer to the heap-allocated HTTP request body.
    inner: Arc<[u8]>,
}

impl BoxBody {
    /// Constructs a new box body.
    pub fn new(inner: Box<[u8]>) -> BoxBody {
        BoxBody {
            inner: Arc::from(inner),
        }
    }

    /// Attempts to open this `BoxBody`.
    ///
    /// This returns `Some(())` if the open is successful, otherwise, this returns
    /// `None.
    pub fn try_open<B>(&self) -> Box<Option<B>>
    where
        B: for<'a> TryFrom<&'a [u8]>,
    {
        Box::new(B::try_from(&self.inner).ok())
    }

    /// Directly opens the box body as the desired type.
    ///
    /// # Panics
    /// Since this does a direct conversion without checking for an error, it's possible that the
    /// `BoxBody` data can be corrupted or undefined. `try_open` should be preferred over this
    /// function. Only use this when confident that this `BoxBody` contains a valid set of data,
    /// and it's crucial that an extra heap allocation for `Option` in `try_open` cannot be spared.
    pub fn open<B>(&self) -> Box<B>
    where
        B: for<'a> From<&'a [u8]>,
    {
        Box::new(B::from(&self.inner))
    }

    /// Attempts to convert a provided object back into a byte stream to pass along across the
    /// request chain.
    ///
    /// This will write the newly formed request body back into the request.
    pub fn close<B>(&mut self, body: B) -> Option<()>
    where
        B: Into<Box<[u8]>>,
    {
        let new_body: Box<[u8]> = body.try_into().ok()?;
        self.inner = Arc::from(new_body);

        Some(())
    }

    /// Gets an immutable pointer to the raw bytes of this `BoxBody`.
    pub fn raw_bytes(&self) -> &[u8] {
        &self.inner
    }

    /// Attempts to open this `BoxBody` as a JSON object.
    ///
    /// Returns `Some(T)` for a successful read, and `None` for an unsuccessful read.
    ///
    /// Part of the `serde_json` feature, this can only be done for types which implement
    /// the `DeserializeOwned` trait.
    #[cfg(feature = "serde_json")]
    pub fn open_json<D>(&self) -> Option<D>
    where
        D: DeserializeOwned,
    {
        serde_json::from_slice::<D>(self.inner.as_ref()).ok()
    }

    /// Attempts to write a JSON object back into this `BoxBody`.
    ///
    /// Returns `Some(())` for a successful write, and `None` for an unsuccessful write.
    ///
    /// Part of the `serde_json` feature, this can only be done for types which implement
    /// the `Serialize` trait.
    #[cfg(feature = "serde_json")]
    pub fn close_json<S>(&mut self, json: S) -> Option<()>
    where
        S: Serialize,
    {
        match serde_json::to_vec(&json).ok() {
            Some(b) => {
                let bx = b.into_boxed_slice();
                self.inner = Arc::from(bx);

                Some(())
            }
            None => None,
        }
    }

    /// Attempts to open this `BoxBody` as an XML object.
    ///
    /// Returns `Some(())` for a successful write, and `None` for an unsuccessful write.
    ///
    /// Part of the `serde_xml` feature, this can only be done for types which implement
    /// the `Serialize` trait.
    #[cfg(feature = "serde_xml")]
    pub fn open_xml<D>(&self) -> Option<D>
    where
        D: DeserializeOwned,
    {
        serde_xml_rs::from_reader(self.inner.as_ref()).ok()
    }

    /// Attempts to write an XML object back into this `BoxBody`.
    ///
    /// Returns `Some(())` for a successful write, and `None` for an unsuccessful write.
    ///
    /// Part of the `serde_xml` feature, this can only be done for types which implement
    /// the `Serialize` trait.
    #[cfg(feature = "serde_xml")]
    pub fn close_xml<S>(&mut self, xml: S) -> Option<()>
    where
        S: Serialize,
    {
        match serde_xml_rs::to_string(&xml).ok() {
            Some(b) => {
                let bx = Box::new(*b.as_bytes());
                self.inner = Arc::from(bx);

                Some(())
            }
            None => None,
        }
    }
}

/// A middleware implementor.
pub trait Seeder {
    fn seed(
        &self,
        input: Guard<&HttpRequest<BoxBody>>,
    ) -> impl Future<Output = Guard<&HttpRequest<BoxBody>>> + Send;
}
