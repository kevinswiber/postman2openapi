use std::borrow::Cow;

use serde::{Deserialize, Deserializer, Serialize};

use crate::core::JsonValue;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct Spec<'a, T: JsonValue> {
    #[serde(borrow, rename = "auth")]
    pub auth: Option<Auth<'a, T>>,

    #[serde(borrow, rename = "event")]
    pub event: Option<Vec<Event<'a, T>>>,

    #[serde(borrow, rename = "info")]
    pub info: Information<'a, T>,

    /// Items are the basic unit for a Postman collection. You can think of them as corresponding
    /// to a single API endpoint. Each Item has one request and may have multiple API responses
    /// associated with it.
    #[serde(borrow, rename = "item")]
    pub item: Vec<Items<'a, T>>,

    #[serde(borrow, rename = "variable")]
    pub variable: Option<Vec<Variable<'a, T>>>,
}

/// Represents authentication helpers provided by Postman
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Auth<'a, T: JsonValue> {
    /// The attributes for [AWS
    /// Auth](http://docs.aws.amazon.com/AmazonS3/latest/dev/RESTAuthentication.html).
    #[serde(borrow, rename = "awsv4")]
    pub awsv4: Option<AuthAttributeUnion<'a, T>>,

    /// The attributes for [Basic
    /// Authentication](https://en.wikipedia.org/wiki/Basic_access_authentication).
    #[serde(borrow, rename = "basic")]
    pub basic: Option<AuthAttributeUnion<'a, T>>,

    /// The helper attributes for [Bearer Token
    /// Authentication](https://tools.ietf.org/html/rfc6750)
    #[serde(borrow, rename = "bearer")]
    pub bearer: Option<AuthAttributeUnion<'a, T>>,

    #[serde(borrow, rename = "jwt")]
    pub jwt: Option<AuthAttributeUnion<'a, T>>,

    /// The attributes for [Digest
    /// Authentication](https://en.wikipedia.org/wiki/Digest_access_authentication).
    #[serde(borrow, rename = "digest")]
    pub digest: Option<AuthAttributeUnion<'a, T>>,

    /// The attributes for [Hawk Authentication](https://github.com/hueniverse/hawk)
    #[serde(borrow, rename = "hawk")]
    pub hawk: Option<AuthAttributeUnion<'a, T>>,

    #[serde(rename = "noauth")]
    pub noauth: Option<T>,

    /// The attributes for [NTLM
    /// Authentication](https://msdn.microsoft.com/en-us/library/cc237488.aspx)
    #[serde(borrow, rename = "ntlm")]
    pub ntlm: Option<AuthAttributeUnion<'a, T>>,

    /// The attributes for [Oauth2](https://oauth.net/1/)
    #[serde(borrow, rename = "oauth1")]
    pub oauth1: Option<AuthAttributeUnion<'a, T>>,

    /// Helper attributes for [Oauth2](https://oauth.net/2/)
    #[serde(rename = "oauth2")]
    pub oauth2: Option<Oauth2AttributeUnion<'a, T>>,

    #[serde(borrow, rename = "apikey")]
    pub apikey: Option<ApiKeyAttributeUnion<'a, T>>,

    #[serde(rename = "type")]
    pub auth_type: AuthType,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ApiKeyAttributeUnion<'a, T: JsonValue> {
    ApiKeyAttributes21(#[serde(borrow)] Vec<ApiKeyAttributes<'a>>),
    ApiKeyAttributes20(Option<T>),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "key", content = "value")]
pub enum ApiKeyAttributes<'a> {
    #[serde(borrow, rename = "key")]
    Key(Cow<'a, str>),
    #[serde(rename = "in")]
    Location(ApiKeyLocation),
    #[serde(borrow, rename = "value")]
    Value(Cow<'a, str>),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum ApiKeyLocation {
    #[serde(rename = "header")]
    Header,
    #[serde(rename = "query")]
    Query,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Oauth2AttributeUnion<'a, T: JsonValue> {
    Oauth2Attributes21(#[serde(borrow)] Vec<Oauth2Attributes<'a>>),
    Oauth2Attributes20(Option<T>),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "key", content = "value")]
pub enum Oauth2Attributes<'a> {
    #[serde(rename = "grantType")]
    GrantType(Oauth2GrantType),
    #[serde(borrow, rename = "accessTokenUrl")]
    AccessTokenUrl(Cow<'a, str>),
    #[serde(borrow, rename = "addTokenTo")]
    AddTokenTo(Cow<'a, str>),
    #[serde(borrow, rename = "authUrl")]
    AuthUrl(Cow<'a, str>),
    #[serde(borrow, rename = "clientId")]
    ClientId(Cow<'a, str>),
    #[serde(borrow, rename = "clientSecret")]
    ClientSecret(Cow<'a, str>),
    #[serde(borrow, rename = "refreshTokenUrl")]
    RefreshTokenUrl(Cow<'a, str>),
    #[serde(borrow, rename = "scope", deserialize_with = "deserialize_scopes")]
    Scope(Vec<Cow<'a, str>>),
    #[serde(borrow, rename = "state")]
    State(Cow<'a, str>),
    #[serde(borrow, rename = "tokenName")]
    TokenName(Cow<'a, str>),
}

fn deserialize_scopes<'de, D>(deserializer: D) -> Result<Vec<Cow<'de, str>>, D::Error>
where
    D: Deserializer<'de>,
{
    let deserialized = Cow::<str>::deserialize(deserializer)?;
    let scopes: Vec<_> = deserialized
        .split(' ')
        .map(|s| Cow::Owned(s.to_string()))
        .collect();
    Ok(scopes)
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum Oauth2GrantType {
    AuthorizationCodeWithPkce,
    ClientCredentials,
    Implicit,
    PasswordCredentials,
    AuthorizationCode,
}

/// Represents an attribute for any authorization method provided by Postman. For example
/// `username` and `password` are set as auth attributes for Basic Authentication method.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AuthAttribute<'a, T: JsonValue> {
    #[serde(borrow, rename = "key")]
    pub key: Cow<'a, str>,

    #[serde(borrow, rename = "type")]
    pub auth_type: Option<Cow<'a, str>>,

    #[serde(rename = "value")]
    pub value: Option<T>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum AuthAttributeUnion<'a, T: JsonValue> {
    AuthAttribute21(#[serde(borrow)] Vec<AuthAttribute<'a, T>>),
    AuthAttribute20(Option<T>),
}

/// Postman allows you to configure scripts to run when specific events occur. These scripts
/// are stored here, and can be referenced in the collection by their ID.
///
/// Defines a script associated with an associated event name
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Event<'a, T: JsonValue> {
    /// Indicates whether the event is disabled. If absent, the event is assumed to be enabled.
    #[serde(rename = "disabled")]
    pub disabled: Option<bool>,

    /// A unique identifier for the enclosing event.
    #[serde(borrow, rename = "id")]
    pub id: Option<Cow<'a, str>>,

    /// Can be set to `test` or `prerequest` for test scripts or pre-request scripts respectively.
    #[serde(rename = "listen")]
    pub listen: Cow<'a, str>,

    #[serde(borrow, rename = "script")]
    pub script: Option<Script<'a, T>>,
}

/// A script is a snippet of Javascript code that can be used to to perform setup or teardown
/// operations on a particular response.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Script<'a, T: JsonValue> {
    #[serde(borrow, rename = "exec")]
    pub exec: Option<ScriptExec<'a>>,

    /// A unique, user defined identifier that can  be used to refer to this script from requests.
    #[serde(borrow, rename = "id")]
    pub id: Option<Cow<'a, str>>,

    /// Script name
    #[serde(borrow, rename = "name")]
    pub name: Option<Cow<'a, str>>,

    #[serde(borrow, rename = "src")]
    pub src: Option<Url<'a, T>>,

    /// Type of the script. E.g: 'text/javascript'
    #[serde(borrow, rename = "type")]
    pub script_type: Option<Cow<'a, str>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct UrlClass<'a, T: JsonValue> {
    /// Contains the URL fragment (if any). Usually this is not transmitted over the network, but
    /// it could be useful to store this in some cases.
    #[serde(borrow, rename = "hash")]
    pub hash: Option<Cow<'a, str>>,

    /// The host for the URL, E.g: api.yourdomain.com. Can be stored as a string or as an array
    /// of strings.
    #[serde(borrow, rename = "host")]
    pub host: Option<Host<'a>>,

    #[serde(borrow, rename = "path")]
    pub path: Option<UrlPath<'a>>,

    /// The port number present in this URL. An empty value implies 80/443 depending on whether
    /// the protocol field contains http/https.
    #[serde(borrow, rename = "port")]
    pub port: Option<Cow<'a, str>>,

    /// The protocol associated with the request, E.g: 'http'
    #[serde(borrow, rename = "protocol")]
    pub protocol: Option<Cow<'a, str>>,

    /// An array of QueryParams, which is basically the query string part of the URL, parsed into
    /// separate variables
    #[serde(borrow, rename = "query")]
    pub query: Option<Vec<QueryParam<'a, T>>>,

    /// The string representation of the request URL, including the protocol, host, path, hash,
    /// query parameter(s) and path variable(s).
    #[serde(borrow, rename = "raw")]
    pub raw: Option<Cow<'a, str>>,

    /// Postman supports path variables with the syntax `/path/:variableName/to/somewhere`. These
    /// variables are stored in this field.
    #[serde(borrow, rename = "variable")]
    pub variable: Option<Vec<Variable<'a, T>>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PathClass<'a> {
    #[serde(borrow, rename = "type")]
    pub path_type: Option<Cow<'a, str>>,

    #[serde(borrow, rename = "value")]
    pub value: Option<Cow<'a, str>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct GraphQlBodyClass<'a> {
    #[serde(borrow, rename = "query")]
    pub query: Option<Cow<'a, str>>,

    #[serde(borrow, rename = "variables")]
    pub variables: Option<Cow<'a, str>>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct QueryParam<'a, T: JsonValue> {
    #[serde(borrow, rename = "description")]
    pub description: Option<DescriptionUnion<'a, T>>,

    /// If set to true, the current query parameter will not be sent with the request.
    #[serde(rename = "disabled")]
    pub disabled: Option<bool>,

    #[serde(borrow, rename = "key")]
    pub key: Option<Cow<'a, str>>,

    #[serde(borrow, rename = "value")]
    pub value: Option<Cow<'a, str>>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct Description<'a, T: JsonValue> {
    /// The content of the description goes here, as a raw string.
    #[serde(borrow, rename = "content")]
    pub content: Option<Cow<'a, str>>,

    /// Holds the mime type of the raw description content. E.g: 'text/markdown' or 'text/html'.
    /// The type is used to correctly render the description when generating documentation, or in
    /// the Postman app.
    #[serde(borrow, rename = "type")]
    pub description_type: Option<Cow<'a, str>>,

    /// Description can have versions associated with it, which should be put in this property.
    #[serde(rename = "version")]
    pub version: Option<T>,
}

/// Collection variables allow you to define a set of variables, that are a *part of the
/// collection*, as opposed to environments, which are separate entities.
/// *Note: Collection variables must not contain any sensitive information.*
///
/// Using variables in your Postman requests eliminates the need to duplicate requests, which
/// can save a lot of time. Variables can be defined, and referenced to from any part of a
/// request.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct Variable<'a, T: JsonValue> {
    #[serde(borrow, rename = "description")]
    pub description: Option<DescriptionUnion<'a, T>>,

    #[serde(rename = "disabled")]
    pub disabled: Option<bool>,

    /// A variable ID is a unique user-defined value that identifies the variable within a
    /// collection. In traditional terms, this would be a variable name.
    #[serde(borrow, rename = "id")]
    pub id: Option<Cow<'a, str>>,

    /// A variable key is a human friendly value that identifies the variable within a
    /// collection. In traditional terms, this would be a variable name.
    #[serde(borrow, rename = "key")]
    pub key: Option<Cow<'a, str>>,

    /// Variable name
    #[serde(borrow, rename = "name")]
    pub name: Option<Cow<'a, str>>,

    /// When set to true, indicates that this variable has been set by Postman
    #[serde(rename = "system")]
    pub system: Option<bool>,

    /// A variable may have multiple types. This field specifies the type of the variable.
    #[serde(rename = "type")]
    pub variable_type: Option<VariableType>,

    /// The value that a variable holds in this collection. Ultimately, the variables will be
    /// replaced by this value, when say running a set of requests from a collection
    #[serde(rename = "value")]
    pub value: Option<T>,
}

/// Detailed description of the info block
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct Information<'a, T: JsonValue> {
    /// Every collection is identified by the unique value of this field. The value of this field
    /// is usually easiest to generate using a UID generator function. If you already have a
    /// collection, it is recommended that you maintain the same id since changing the id usually
    /// implies that is a different collection than it was originally.
    /// *Note: This field exists for compatibility reasons with Collection Format V1.*
    #[serde(borrow, rename = "_postman_id")]
    pub postman_id: Option<Cow<'a, str>>,

    #[serde(borrow, rename = "_exporter_id")]
    pub exporter_id: Option<Cow<'a, str>>,

    #[serde(borrow, rename = "description")]
    pub description: Option<DescriptionUnion<'a, T>>,

    /// A collection's friendly name is defined by this field. You would want to set this field
    /// to a value that would allow you to easily identify this collection among a bunch of other
    /// collections, as such outlining its usage or content.
    #[serde(rename = "name")]
    pub name: Cow<'a, str>,

    /// This should ideally hold a link to the Postman schema that is used to validate this
    /// collection. E.g: https://schema.getpostman.com/collection/v1
    #[serde(rename = "schema")]
    pub schema: Cow<'a, str>,

    #[serde(borrow, rename = "version")]
    pub version: Option<CollectionVersion<'a, T>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CollectionVersionClass<'a, T: JsonValue> {
    /// A human friendly identifier to make sense of the version numbers. E.g: 'beta-3'
    #[serde(borrow, rename = "identifier")]
    pub identifier: Option<Cow<'a, str>>,

    /// Increment this number if you make changes to the collection that changes its behaviour.
    /// E.g: Removing or adding new test scripts. (partly or completely).
    #[serde(rename = "major")]
    pub major: i64,

    #[serde(rename = "meta")]
    pub meta: Option<T>,

    /// You should increment this number if you make changes that will not break anything that
    /// uses the collection. E.g: removing a folder.
    #[serde(rename = "minor")]
    pub minor: i64,

    /// Ideally, minor changes to a collection should result in the increment of this number.
    #[serde(rename = "patch")]
    pub patch: i64,
}

/// Items are entities which contain an actual HTTP request, and sample responses attached to
/// it.
///
/// One of the primary goals of Postman is to organize the development of APIs. To this end,
/// it is necessary to be able to group requests together. This can be achived using
/// 'Folders'. A folder just is an ordered set of requests.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Items<'a, T: JsonValue> {
    #[serde(borrow, rename = "description")]
    pub description: Option<DescriptionUnion<'a, T>>,

    #[serde(borrow, rename = "event")]
    pub event: Option<Vec<Event<'a, T>>>,

    /// A unique ID that is used to identify collections internally
    #[serde(borrow, rename = "id")]
    pub id: Option<Cow<'a, str>>,

    /// A human readable identifier for the current item.
    ///
    /// A folder's friendly name is defined by this field. You would want to set this field to a
    /// value that would allow you to easily identify this folder.
    #[serde(borrow, rename = "name")]
    pub name: Option<Cow<'a, str>>,

    /// Set of configurations used to alter the usual behavior of sending the request
    #[serde(rename = "protocolProfileBehavior")]
    pub protocol_profile_behavior: Option<ProtocolProfileBehavior>,

    #[serde(borrow, rename = "request")]
    pub request: Option<RequestUnion<'a, T>>,

    #[serde(borrow, rename = "response")]
    pub response: Option<Vec<Option<ResponseClass<'a, T>>>>,

    #[serde(borrow, rename = "variable")]
    pub variable: Option<Vec<Variable<'a, T>>>,

    #[serde(borrow, rename = "auth")]
    pub auth: Option<Auth<'a, T>>,

    /// Items are entities which contain an actual HTTP request, and sample responses attached to
    /// it. Folders may contain many items.
    #[serde(borrow, rename = "item")]
    pub item: Option<Vec<Items<'a, T>>>,
}

/// Set of configurations used to alter the usual behavior of sending the request
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ProtocolProfileBehavior {
    /// Disable body pruning for GET, COPY, HEAD, PURGE and UNLOCK request methods.
    #[serde(rename = "disableBodyPruning")]
    pub disable_body_pruning: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct RequestClass<'a, T: JsonValue> {
    #[serde(borrow, rename = "auth")]
    pub auth: Option<Auth<'a, T>>,

    #[serde(borrow, rename = "body")]
    pub body: Option<Body<'a, T>>,

    #[serde(borrow, rename = "certificate")]
    pub certificate: Option<Certificate<'a, T>>,

    #[serde(borrow, rename = "description")]
    pub description: Option<DescriptionUnion<'a, T>>,

    #[serde(borrow, rename = "header")]
    pub header: Option<HeaderUnion<'a, T>>,

    #[serde(borrow, rename = "method")]
    pub method: Option<Cow<'a, str>>,

    #[serde(borrow, rename = "proxy")]
    pub proxy: Option<ProxyConfig<'a>>,

    #[serde(borrow, rename = "url")]
    pub url: Option<Url<'a, T>>,
}

/// This field contains the data usually contained in the request body.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Body<'a, T: JsonValue> {
    /// When set to true, prevents request body from being sent.
    #[serde(rename = "disabled")]
    pub disabled: Option<bool>,

    #[serde(borrow, rename = "file")]
    pub file: Option<File<'a>>,

    #[serde(borrow, rename = "formdata")]
    pub formdata: Option<Vec<FormParameter<'a, T>>>,

    /// Postman stores the type of data associated with this request in this field.
    #[serde(rename = "mode")]
    pub mode: Option<Mode>,

    #[serde(borrow, rename = "raw")]
    pub raw: Option<Cow<'a, str>>,

    #[serde(borrow, rename = "options")]
    pub options: Option<BodyOptions<'a>>,

    #[serde(borrow, rename = "urlencoded")]
    pub urlencoded: Option<Vec<UrlEncodedParameter<'a, T>>>,

    #[serde(borrow, rename = "graphql")]
    pub graphql: Option<GraphQlBody<'a>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct BodyOptions<'a> {
    #[serde(borrow, rename = "raw")]
    pub raw: Option<RawOptions<'a>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct RawOptions<'a> {
    #[serde(borrow, rename = "language")]
    pub language: Option<Cow<'a, str>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct File<'a> {
    #[serde(borrow, rename = "content")]
    pub content: Option<Cow<'a, str>>,

    #[serde(borrow, rename = "src")]
    pub src: Option<Cow<'a, str>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct FormParameter<'a, T: JsonValue> {
    /// Override Content-Type header of this form data entity.
    #[serde(borrow, rename = "contentType")]
    pub content_type: Option<Cow<'a, str>>,

    #[serde(borrow, rename = "description")]
    pub description: Option<DescriptionUnion<'a, T>>,

    /// When set to true, prevents this form data entity from being sent.
    #[serde(rename = "disabled")]
    pub disabled: Option<bool>,

    #[serde(rename = "key")]
    pub key: Cow<'a, str>,

    #[serde(borrow, rename = "type")]
    pub form_parameter_type: Option<Cow<'a, str>>,

    #[serde(borrow, rename = "value")]
    pub value: Option<Cow<'a, str>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct UrlEncodedParameter<'a, T: JsonValue> {
    #[serde(borrow, rename = "description")]
    pub description: Option<DescriptionUnion<'a, T>>,

    #[serde(rename = "disabled")]
    pub disabled: Option<bool>,

    #[serde(rename = "key")]
    pub key: Cow<'a, str>,

    #[serde(borrow, rename = "value")]
    pub value: Option<Cow<'a, str>>,
}

/// A representation of an ssl certificate
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Certificate<'a, T: JsonValue> {
    /// An object containing path to file certificate, on the file system
    #[serde(rename = "cert")]
    pub cert: Option<Cert<T>>,

    /// An object containing path to file containing private key, on the file system
    #[serde(rename = "key")]
    pub key: Option<Key<T>>,

    /// A list of Url match pattern strings, to identify Urls this certificate can be used for.
    #[serde(rename = "matches")]
    pub matches: Option<Vec<Option<T>>>,

    /// A name for the certificate for user reference
    #[serde(borrow, rename = "name")]
    pub name: Option<Cow<'a, str>>,

    /// The passphrase for the certificate
    #[serde(borrow, rename = "passphrase")]
    pub passphrase: Option<Cow<'a, str>>,
}

/// An object containing path to file certificate, on the file system
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Cert<T: JsonValue> {
    /// The path to file containing key for certificate, on the file system
    #[serde(rename = "src")]
    pub src: Option<T>,
}

/// An object containing path to file containing private key, on the file system
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Key<T: JsonValue> {
    /// The path to file containing key for certificate, on the file system
    #[serde(rename = "src")]
    pub src: Option<T>,
}

/// A representation for a list of headers
///
/// Represents a single HTTP Header
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Header<'a, T: JsonValue> {
    #[serde(borrow, rename = "description")]
    pub description: Option<DescriptionUnion<'a, T>>,

    /// If set to true, the current header will not be sent with requests.
    #[serde(rename = "disabled")]
    pub disabled: Option<bool>,

    /// This holds the LHS of the HTTP Header, e.g ``Content-Type`` or ``X-Custom-Header``
    #[serde(rename = "key")]
    pub key: Cow<'a, str>,

    /// The value (or the RHS) of the Header is stored in this field.
    #[serde(rename = "value", deserialize_with = "deserialize_as_str")]
    pub value: String,
}

struct DeserializeAnyAsStr;
impl<'de> serde::de::Visitor<'de> for DeserializeAnyAsStr {
    type Value = String;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an integer or a string")
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.to_string())
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> {
        Ok(v.to_string())
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E> {
        Ok(v)
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.to_string())
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.to_string())
    }
}

fn deserialize_as_str<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(DeserializeAnyAsStr)
}

/// Using the Proxy, you can configure your custom proxy into the postman for particular url
/// match
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ProxyConfig<'a> {
    /// When set to true, ignores this proxy configuration entity
    #[serde(rename = "disabled")]
    pub disabled: Option<bool>,

    /// The proxy server host
    #[serde(borrow, rename = "host")]
    pub host: Option<Cow<'a, str>>,

    /// The Url match for which the proxy config is defined
    #[serde(borrow, rename = "match")]
    pub proxy_config_match: Option<Cow<'a, str>>,

    /// The proxy server port
    #[serde(rename = "port")]
    pub port: Option<i64>,

    /// The tunneling details for the proxy config
    #[serde(rename = "tunnel")]
    pub tunnel: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ResponseClass<'a, T: JsonValue> {
    /// The name of the response.
    #[serde(borrow, rename = "name")]
    pub name: Option<Cow<'a, str>>,

    /// The raw text of the response.
    #[serde(borrow, rename = "body")]
    pub body: Option<Cow<'a, str>>,

    /// The numerical response code, example: 200, 201, 404, etc.
    #[serde(rename = "code")]
    pub code: Option<i64>,

    #[serde(borrow, rename = "cookie")]
    pub cookie: Option<Vec<Cookie<'a, T>>>,

    #[serde(borrow, rename = "header")]
    pub header: Option<Headers<'a, T>>,

    /// A unique, user defined identifier that can  be used to refer to this response from
    /// requests.
    #[serde(borrow, rename = "id")]
    pub id: Option<Cow<'a, str>>,

    #[serde(borrow, rename = "originalRequest")]
    pub original_request: Option<RequestClass<'a, T>>,

    /// The time taken by the request to complete. If a number, the unit is milliseconds. If the
    /// response is manually created, this can be set to `null`.
    #[serde(borrow, rename = "responseTime")]
    pub response_time: Option<ResponseTime<'a>>,

    /// The response status, e.g: '200 OK'
    #[serde(borrow, rename = "status")]
    pub status: Option<Cow<'a, str>>,
}

/// A Cookie, that follows the [Google Chrome
/// format](https://developer.chrome.com/extensions/cookies)
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Cookie<'a, T: JsonValue> {
    /// The domain for which this cookie is valid.
    #[serde(borrow, rename = "domain")]
    pub domain: Option<Cow<'a, str>>,

    /// When the cookie expires.
    #[serde(borrow, rename = "expires")]
    pub expires: Option<Cow<'a, str>>,

    /// Custom attributes for a cookie go here, such as the [Priority
    /// Field](https://code.google.com/p/chromium/issues/detail?id=232693)
    #[serde(rename = "extensions")]
    pub extensions: Option<Vec<Option<T>>>,

    /// True if the cookie is a host-only cookie. (i.e. a request's URL domain must exactly match
    /// the domain of the cookie).
    #[serde(rename = "hostOnly")]
    pub host_only: Option<bool>,

    /// Indicates if this cookie is HTTP Only. (if True, the cookie is inaccessible to
    /// client-side scripts)
    #[serde(rename = "httpOnly")]
    pub http_only: Option<bool>,

    #[serde(borrow, rename = "maxAge")]
    pub max_age: Option<Cow<'a, str>>,

    /// This is the name of the Cookie.
    #[serde(borrow, rename = "name")]
    pub name: Option<Cow<'a, str>>,

    /// The path associated with the Cookie.
    #[serde(borrow, rename = "path")]
    pub path: Option<Cow<'a, str>>,

    /// Indicates if the 'secure' flag is set on the Cookie, meaning that it is transmitted over
    /// secure connections only. (typically HTTPS)
    #[serde(rename = "secure")]
    pub secure: Option<bool>,

    /// True if the cookie is a session cookie.
    #[serde(rename = "session")]
    pub session: Option<bool>,

    /// The value of the Cookie.
    #[serde(borrow, rename = "value")]
    pub value: Option<Cow<'a, str>>,
}

/// The host for the URL, E.g: api.yourdomain.com. Can be stored as a string or as an array
/// of strings.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Host<'a> {
    String(#[serde(borrow)] Cow<'a, str>),

    StringArray(#[serde(borrow)] Vec<Cow<'a, str>>),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ScriptExec<'a> {
    String(#[serde(borrow)] Cow<'a, str>),

    StringArray(#[serde(borrow)] Vec<Cow<'a, str>>),
}

/// If object, contains the complete broken-down URL for this request. If string, contains
/// the literal request URL.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Url<'a, T: JsonValue> {
    String(#[serde(borrow)] Cow<'a, str>),

    UrlClass(#[serde(borrow)] Box<UrlClass<'a, T>>),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum UrlPath<'a> {
    String(#[serde(borrow)] Cow<'a, str>),

    UnionArray(#[serde(borrow)] Vec<PathElement<'a>>),
}

/// The complete path of the current url, broken down into segments. A segment could be a
/// string, or a path variable.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum PathElement<'a> {
    PathClass(#[serde(borrow)] PathClass<'a>),

    String(#[serde(borrow)] Cow<'a, str>),
}

/// A Description can be a raw text, or be an object, which holds the description along with
/// its format.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum DescriptionUnion<'a, T: JsonValue> {
    String(#[serde(borrow)] Cow<'a, str>),
    Description(#[serde(borrow)] Description<'a, T>),
}

impl<'a, T: JsonValue> From<&'a DescriptionUnion<'a, T>> for Cow<'a, str> {
    fn from(description: &'a DescriptionUnion<'a, T>) -> Self {
        match description {
            DescriptionUnion::Description(desc) => {
                desc.content.as_ref().unwrap_or(&Cow::Borrowed("")).clone()
            }
            DescriptionUnion::String(str) => Cow::Borrowed(str),
        }
    }
}

impl<'a, T: JsonValue> From<&'a DescriptionUnion<'a, T>> for String {
    fn from(description: &'a DescriptionUnion<'a, T>) -> Self {
        match description {
            DescriptionUnion::Description(desc) => desc
                .content
                .as_ref()
                .map(|s| s.to_string())
                .unwrap_or_default(),
            DescriptionUnion::String(str) => str.to_string(),
        }
    }
}

/// Postman allows you to version your collections as they grow, and this field holds the
/// version number. While optional, it is recommended that you use this field to its fullest
/// extent!
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum CollectionVersion<'a, T: JsonValue> {
    CollectionVersionClass(#[serde(borrow)] CollectionVersionClass<'a, T>),

    String(#[serde(borrow)] Cow<'a, str>),
}

/// A request represents an HTTP request. If a string, the string is assumed to be the
/// request URL and the method is assumed to be 'GET'.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum RequestUnion<'a, T: JsonValue> {
    RequestClass(#[serde(borrow)] Box<RequestClass<'a, T>>),

    String(#[serde(borrow)] Cow<'a, str>),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum HeaderUnion<'a, T: JsonValue> {
    HeaderArray(#[serde(borrow)] Vec<Header<'a, T>>),

    String(#[serde(borrow)] Cow<'a, str>),
}

/// A response represents an HTTP response.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Response<'a, T: JsonValue> {
    //AnythingArray(Vec<Option<serde_json::Value>>),

    //Bool(bool),

    //Double(f64),

    //Integer(i64),
    ResponseClass(#[serde(borrow)] Box<ResponseClass<'a, T>>),

    String(#[serde(borrow)] Cow<'a, str>),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Headers<'a, T: JsonValue> {
    String(#[serde(borrow)] Cow<'a, str>),

    UnionArray(#[serde(borrow)] Vec<HeaderElement<'a, T>>),
}

/// No HTTP request is complete without its headers, and the same is true for a Postman
/// request. This field is an array containing all the headers.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum HeaderElement<'a, T: JsonValue> {
    Header(#[serde(borrow)] Header<'a, T>),

    String(#[serde(borrow)] Cow<'a, str>),
}

/// The time taken by the request to complete. If a number, the unit is milliseconds. If the
/// response is manually created, this can be set to `null`.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ResponseTime<'a> {
    Number(u64),

    String(#[serde(borrow)] Cow<'a, str>),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum GraphQlBody<'a> {
    String(#[serde(borrow)] Cow<'a, str>),

    GraphQlBodyClass(#[serde(borrow)] GraphQlBodyClass<'a>),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum AuthType {
    #[serde(rename = "awsv4")]
    Awsv4,

    #[serde(rename = "basic")]
    Basic,

    #[serde(rename = "bearer")]
    Bearer,

    #[serde(rename = "digest")]
    Digest,

    #[serde(rename = "jwt")]
    Jwt,

    #[serde(rename = "hawk")]
    Hawk,

    #[serde(rename = "noauth")]
    Noauth,

    #[serde(rename = "ntlm")]
    Ntlm,

    #[serde(rename = "oauth1")]
    Oauth1,

    #[serde(rename = "oauth2")]
    Oauth2,

    #[serde(rename = "apikey")]
    Apikey,
}

/// Returns `Noauth` for AuthType by default
impl Default for AuthType {
    fn default() -> AuthType {
        AuthType::Noauth
    }
}

/// A variable may have multiple types. This field specifies the type of the variable.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum VariableType {
    #[serde(rename = "any")]
    Any,

    #[serde(rename = "boolean")]
    Boolean,

    #[serde(rename = "number")]
    Number,

    #[serde(rename = "string")]
    String,
}

/// Postman stores the type of data associated with this request in this field.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum Mode {
    #[serde(rename = "file")]
    File,

    #[serde(rename = "formdata")]
    Formdata,

    #[serde(rename = "raw")]
    Raw,

    #[serde(rename = "urlencoded")]
    Urlencoded,

    #[serde(rename = "graphql")]
    GraphQl,
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use crate::core::WrappedJson;

    use super::*;

    #[test]
    fn deserializes_oauth2() {
        let fixture = get_fixture("oauth2-code.postman.json");
        let spec: Spec<WrappedJson> = serde_json::from_str(&fixture).unwrap();
        let oauth2 = match spec.auth.unwrap().oauth2.unwrap() {
            Oauth2AttributeUnion::Oauth2Attributes21(oauth2) => oauth2,
            _ => panic!("Expected Oauth2AttributeUnion21"),
        };

        let mut grant_type = None;
        let mut auth_url = None;
        let mut access_token_url = None;

        oauth2.iter().for_each(|a| match a {
            Oauth2Attributes::GrantType(g) => grant_type = Some(g.clone()),
            Oauth2Attributes::AuthUrl(u) => auth_url = Some(u.clone()),
            Oauth2Attributes::AccessTokenUrl(u) => access_token_url = Some(u.clone()),
            _ => {}
        });
        assert_eq!(grant_type, None);
        assert_eq!(
            auth_url,
            Some(Cow::Owned(
                "https://example.com/oauth2/authorization".to_string()
            ))
        );
        assert_eq!(
            access_token_url,
            Some(Cow::Owned("https://example.com/oauth2/token".to_string()))
        );
    }

    #[test]
    fn deserializes_apikey() {
        let fixture = get_fixture("api-key.postman.json");
        let spec: Spec<WrappedJson> = serde_json::from_str(&fixture).unwrap();
        let apikey = match spec.auth.unwrap().apikey.unwrap() {
            ApiKeyAttributeUnion::ApiKeyAttributes21(apikey) => apikey,
            _ => panic!("Expected ApikeyAttributeUnion21"),
        };

        let mut key = None;
        let mut location = None;
        let mut value = None;
        apikey.iter().for_each(|a| match a {
            ApiKeyAttributes::Key(k) => key = Some(k.clone()),
            ApiKeyAttributes::Location(l) => location = Some(l.clone()),
            ApiKeyAttributes::Value(v) => value = Some(v.clone()),
        });
        assert_eq!(key, Some(Cow::Borrowed("Authorization")));
        assert_eq!(location, None);
        assert_eq!(value, None);
    }

    fn get_fixture(filename: &str) -> String {
        use std::fs;

        let filename: std::path::PathBuf =
            [env!("CARGO_MANIFEST_DIR"), "./tests/fixtures/", filename]
                .iter()
                .collect();
        let file = filename.into_os_string().into_string().unwrap();
        fs::read_to_string(file).unwrap()
    }
}
