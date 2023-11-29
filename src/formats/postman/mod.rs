use serde::{Deserialize, Deserializer};

extern crate serde_json;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct Spec<'a> {
    #[serde(borrow, rename = "auth")]
    pub auth: Option<Auth<'a>>,

    #[serde(borrow, rename = "event")]
    pub event: Option<Vec<Event<'a>>>,

    #[serde(borrow, rename = "info")]
    pub info: Information<'a>,

    /// Items are the basic unit for a Postman collection. You can think of them as corresponding
    /// to a single API endpoint. Each Item has one request and may have multiple API responses
    /// associated with it.
    #[serde(borrow, rename = "item")]
    pub item: Vec<Items<'a>>,

    #[serde(borrow, rename = "variable")]
    pub variable: Option<Vec<Variable<'a>>>,
}

/// Represents authentication helpers provided by Postman
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Auth<'a> {
    /// The attributes for [AWS
    /// Auth](http://docs.aws.amazon.com/AmazonS3/latest/dev/RESTAuthentication.html).
    #[serde(borrow, rename = "awsv4")]
    pub awsv4: Option<AuthAttributeUnion<'a>>,

    /// The attributes for [Basic
    /// Authentication](https://en.wikipedia.org/wiki/Basic_access_authentication).
    #[serde(borrow, rename = "basic")]
    pub basic: Option<AuthAttributeUnion<'a>>,

    /// The helper attributes for [Bearer Token
    /// Authentication](https://tools.ietf.org/html/rfc6750)
    #[serde(borrow, rename = "bearer")]
    pub bearer: Option<AuthAttributeUnion<'a>>,

    #[serde(borrow, rename = "jwt")]
    pub jwt: Option<AuthAttributeUnion<'a>>,

    /// The attributes for [Digest
    /// Authentication](https://en.wikipedia.org/wiki/Digest_access_authentication).
    #[serde(borrow, rename = "digest")]
    pub digest: Option<AuthAttributeUnion<'a>>,

    /// The attributes for [Hawk Authentication](https://github.com/hueniverse/hawk)
    #[serde(borrow, rename = "hawk")]
    pub hawk: Option<AuthAttributeUnion<'a>>,

    #[serde(rename = "noauth")]
    pub noauth: Option<serde_json::Value>,

    /// The attributes for [NTLM
    /// Authentication](https://msdn.microsoft.com/en-us/library/cc237488.aspx)
    #[serde(borrow, rename = "ntlm")]
    pub ntlm: Option<AuthAttributeUnion<'a>>,

    /// The attributes for [Oauth2](https://oauth.net/1/)
    #[serde(borrow, rename = "oauth1")]
    pub oauth1: Option<AuthAttributeUnion<'a>>,

    /// Helper attributes for [Oauth2](https://oauth.net/2/)
    #[serde(rename = "oauth2")]
    pub oauth2: Option<Oauth2>,

    #[serde(rename = "apikey")]
    pub apikey: Option<ApiKey>,

    #[serde(rename = "type")]
    pub auth_type: AuthType,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ApiKey {
    pub key: Option<String>,
    pub location: ApiKeyLocation,
    pub value: Option<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub enum ApiKeyLocation {
    Header,
    Query,
}

impl<'de> Deserialize<'de> for ApiKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut key = None;
        let mut location = ApiKeyLocation::Header;
        let mut value = None;

        let deserialized = AuthAttributeUnion::deserialize(deserializer)?;
        if let AuthAttributeUnion::AuthAttribute21(v) = deserialized {
            for item in v {
                if let Some(serde_json::Value::String(str)) = item.value {
                    match item.key {
                        "key" => key = Some(str),
                        "in" => {
                            location = match str.as_str() {
                                "query" => ApiKeyLocation::Query,
                                "header" => ApiKeyLocation::Header,
                                _ => ApiKeyLocation::Header,
                            }
                        }
                        "value" => value = Some(str.to_string()),
                        _ => {}
                    }
                }
            }
        }
        Ok(ApiKey {
            key,
            location,
            value,
        })
    }
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct Oauth2 {
    pub grant_type: Oauth2GrantType,
    pub access_token_url: Option<String>,
    pub add_token_to: Option<String>,
    pub auth_url: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub refresh_token_url: Option<String>,
    pub scope: Option<Vec<String>>,
    pub state: Option<String>,
    pub token_name: Option<String>,
}

impl<'de> Deserialize<'de> for Oauth2 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut grant_type = Oauth2GrantType::AuthorizationCode;
        let mut access_token_url = None;
        let mut add_token_to = None;
        let mut auth_url = None;
        let mut client_id = None;
        let mut client_secret = None;
        let mut refresh_token_url = None;
        let mut scope = None;
        let mut state = None;
        let mut token_name = None;

        let deserialized = AuthAttributeUnion::deserialize(deserializer)?;
        if let AuthAttributeUnion::AuthAttribute21(v) = deserialized {
            for item in v {
                if let Some(serde_json::Value::String(str)) = item.value {
                    match item.key {
                        "grantType" => {
                            grant_type = match str.as_str() {
                                "authorization_code" => Oauth2GrantType::AuthorizationCode,
                                "authorization_code_with_pkce" => {
                                    Oauth2GrantType::AuthorizationCodeWithPkce
                                }
                                "client_credentials" => Oauth2GrantType::ClientCredentials,
                                "implicit" => Oauth2GrantType::Implicit,
                                "password_credentials" => Oauth2GrantType::PasswordCredentials,
                                _ => Oauth2GrantType::AuthorizationCode,
                            }
                        }
                        "accessTokenUrl" => access_token_url = Some(str),
                        "addTokenTo" => add_token_to = Some(str),
                        "authUrl" => auth_url = Some(str),
                        "clientId" => client_id = Some(str),
                        "clientSecret" => client_secret = Some(str),
                        "refreshTokenUrl" => refresh_token_url = Some(str),
                        "scope" => scope = Some(str.split(' ').map(|s| s.to_string()).collect()),
                        "state" => state = Some(str),
                        "tokenName" => token_name = Some(str),
                        _ => {}
                    }
                }
            }
        }
        Ok(Oauth2 {
            grant_type,
            access_token_url,
            add_token_to,
            auth_url,
            client_id,
            client_secret,
            refresh_token_url,
            scope,
            state,
            token_name,
        })
    }
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
pub struct AuthAttribute<'a> {
    #[serde(rename = "key")]
    pub key: &'a str,

    #[serde(rename = "type")]
    pub auth_type: Option<&'a str>,

    #[serde(rename = "value")]
    pub value: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum AuthAttributeUnion<'a> {
    #[serde(borrow)]
    AuthAttribute21(Vec<AuthAttribute<'a>>),
    AuthAttribute20(Option<serde_json::Value>),
}

/// Postman allows you to configure scripts to run when specific events occur. These scripts
/// are stored here, and can be referenced in the collection by their ID.
///
/// Defines a script associated with an associated event name
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Event<'a> {
    /// Indicates whether the event is disabled. If absent, the event is assumed to be enabled.
    #[serde(rename = "disabled")]
    pub disabled: Option<bool>,

    /// A unique identifier for the enclosing event.
    #[serde(rename = "id")]
    pub id: Option<&'a str>,

    /// Can be set to `test` or `prerequest` for test scripts or pre-request scripts respectively.
    #[serde(rename = "listen")]
    pub listen: &'a str,

    #[serde(rename = "script")]
    pub script: Option<Script<'a>>,
}

/// A script is a snippet of Javascript code that can be used to to perform setup or teardown
/// operations on a particular response.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Script<'a> {
    #[serde(rename = "exec")]
    pub exec: Option<Host<'a>>,

    /// A unique, user defined identifier that can  be used to refer to this script from requests.
    #[serde(rename = "id")]
    pub id: Option<&'a str>,

    /// Script name
    #[serde(rename = "name")]
    pub name: Option<&'a str>,

    #[serde(rename = "src")]
    pub src: Option<Url<'a>>,

    /// Type of the script. E.g: 'text/javascript'
    #[serde(rename = "type")]
    pub script_type: Option<&'a str>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct UrlClass<'a> {
    /// Contains the URL fragment (if any). Usually this is not transmitted over the network, but
    /// it could be useful to store this in some cases.
    #[serde(rename = "hash")]
    pub hash: Option<&'a str>,

    /// The host for the URL, E.g: api.yourdomain.com. Can be stored as a string or as an array
    /// of strings.
    #[serde(rename = "host")]
    pub host: Option<Host<'a>>,

    #[serde(rename = "path")]
    pub path: Option<UrlPath<'a>>,

    /// The port number present in this URL. An empty value implies 80/443 depending on whether
    /// the protocol field contains http/https.
    #[serde(rename = "port")]
    pub port: Option<&'a str>,

    /// The protocol associated with the request, E.g: 'http'
    #[serde(rename = "protocol")]
    pub protocol: Option<&'a str>,

    /// An array of QueryParams, which is basically the query string part of the URL, parsed into
    /// separate variables
    #[serde(rename = "query")]
    pub query: Option<Vec<QueryParam<'a>>>,

    /// The string representation of the request URL, including the protocol, host, path, hash,
    /// query parameter(s) and path variable(s).
    #[serde(rename = "raw")]
    pub raw: Option<&'a str>,

    /// Postman supports path variables with the syntax `/path/:variableName/to/somewhere`. These
    /// variables are stored in this field.
    #[serde(rename = "variable")]
    pub variable: Option<Vec<Variable<'a>>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PathClass<'a> {
    #[serde(rename = "type")]
    pub path_type: Option<&'a str>,

    #[serde(rename = "value")]
    pub value: Option<&'a str>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct GraphQlBodyClass<'a> {
    #[serde(rename = "query")]
    pub query: Option<&'a str>,

    #[serde(rename = "variables")]
    pub variables: Option<&'a str>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct QueryParam<'a> {
    #[serde(rename = "description")]
    pub description: Option<DescriptionUnion<'a>>,

    /// If set to true, the current query parameter will not be sent with the request.
    #[serde(rename = "disabled")]
    pub disabled: Option<bool>,

    #[serde(rename = "key")]
    pub key: Option<&'a str>,

    #[serde(rename = "value")]
    pub value: Option<&'a str>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct Description<'a> {
    /// The content of the description goes here, as a raw string.
    #[serde(rename = "content")]
    pub content: Option<&'a str>,

    /// Holds the mime type of the raw description content. E.g: 'text/markdown' or 'text/html'.
    /// The type is used to correctly render the description when generating documentation, or in
    /// the Postman app.
    #[serde(rename = "type")]
    pub description_type: Option<&'a str>,

    /// Description can have versions associated with it, which should be put in this property.
    #[serde(rename = "version")]
    pub version: Option<serde_json::Value>,
}

/// Collection variables allow you to define a set of variables, that are a *part of the
/// collection*, as opposed to environments, which are separate entities.
/// *Note: Collection variables must not contain any sensitive information.*
///
/// Using variables in your Postman requests eliminates the need to duplicate requests, which
/// can save a lot of time. Variables can be defined, and referenced to from any part of a
/// request.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct Variable<'a> {
    #[serde(rename = "description")]
    pub description: Option<DescriptionUnion<'a>>,

    #[serde(rename = "disabled")]
    pub disabled: Option<bool>,

    /// A variable ID is a unique user-defined value that identifies the variable within a
    /// collection. In traditional terms, this would be a variable name.
    #[serde(rename = "id")]
    pub id: Option<&'a str>,

    /// A variable key is a human friendly value that identifies the variable within a
    /// collection. In traditional terms, this would be a variable name.
    #[serde(rename = "key")]
    pub key: Option<&'a str>,

    /// Variable name
    #[serde(rename = "name")]
    pub name: Option<&'a str>,

    /// When set to true, indicates that this variable has been set by Postman
    #[serde(rename = "system")]
    pub system: Option<bool>,

    /// A variable may have multiple types. This field specifies the type of the variable.
    #[serde(rename = "type")]
    pub variable_type: Option<VariableType>,

    /// The value that a variable holds in this collection. Ultimately, the variables will be
    /// replaced by this value, when say running a set of requests from a collection
    #[serde(rename = "value")]
    pub value: Option<serde_json::Value>,
}

/// Detailed description of the info block
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct Information<'a> {
    /// Every collection is identified by the unique value of this field. The value of this field
    /// is usually easiest to generate using a UID generator function. If you already have a
    /// collection, it is recommended that you maintain the same id since changing the id usually
    /// implies that is a different collection than it was originally.
    /// *Note: This field exists for compatibility reasons with Collection Format V1.*
    #[serde(rename = "_postman_id")]
    pub postman_id: Option<&'a str>,

    #[serde(rename = "_exporter_id")]
    pub exporter_id: Option<&'a str>,

    #[serde(rename = "description")]
    pub description: Option<DescriptionUnion<'a>>,

    /// A collection's friendly name is defined by this field. You would want to set this field
    /// to a value that would allow you to easily identify this collection among a bunch of other
    /// collections, as such outlining its usage or content.
    #[serde(rename = "name")]
    pub name: &'a str,

    /// This should ideally hold a link to the Postman schema that is used to validate this
    /// collection. E.g: https://schema.getpostman.com/collection/v1
    #[serde(rename = "schema")]
    pub schema: &'a str,

    #[serde(rename = "version")]
    pub version: Option<CollectionVersion<'a>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CollectionVersionClass<'a> {
    /// A human friendly identifier to make sense of the version numbers. E.g: 'beta-3'
    #[serde(rename = "identifier")]
    pub identifier: Option<&'a str>,

    /// Increment this number if you make changes to the collection that changes its behaviour.
    /// E.g: Removing or adding new test scripts. (partly or completely).
    #[serde(rename = "major")]
    pub major: i64,

    #[serde(rename = "meta")]
    pub meta: Option<serde_json::Value>,

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
pub struct Items<'a> {
    #[serde(rename = "description")]
    pub description: Option<DescriptionUnion<'a>>,

    #[serde(rename = "event")]
    pub event: Option<Vec<Event<'a>>>,

    /// A unique ID that is used to identify collections internally
    #[serde(rename = "id")]
    pub id: Option<&'a str>,

    /// A human readable identifier for the current item.
    ///
    /// A folder's friendly name is defined by this field. You would want to set this field to a
    /// value that would allow you to easily identify this folder.
    #[serde(rename = "name")]
    pub name: Option<&'a str>,

    /// Set of configurations used to alter the usual behavior of sending the request
    #[serde(rename = "protocolProfileBehavior")]
    pub protocol_profile_behavior: Option<ProtocolProfileBehavior>,

    #[serde(rename = "request")]
    pub request: Option<RequestUnion<'a>>,

    #[serde(rename = "response")]
    pub response: Option<Vec<Option<ResponseClass<'a>>>>,

    #[serde(rename = "variable")]
    pub variable: Option<Vec<Variable<'a>>>,

    #[serde(rename = "auth")]
    pub auth: Option<Auth<'a>>,

    /// Items are entities which contain an actual HTTP request, and sample responses attached to
    /// it. Folders may contain many items.
    #[serde(rename = "item")]
    pub item: Option<Vec<Items<'a>>>,
}

/// Set of configurations used to alter the usual behavior of sending the request
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ProtocolProfileBehavior {
    /// Disable body pruning for GET, COPY, HEAD, PURGE and UNLOCK request methods.
    #[serde(rename = "disableBodyPruning")]
    pub disable_body_pruning: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct RequestClass<'a> {
    #[serde(rename = "auth")]
    pub auth: Option<Auth<'a>>,

    #[serde(rename = "body")]
    pub body: Option<Body<'a>>,

    #[serde(rename = "certificate")]
    pub certificate: Option<Certificate<'a>>,

    #[serde(rename = "description")]
    pub description: Option<DescriptionUnion<'a>>,

    #[serde(rename = "header")]
    pub header: Option<HeaderUnion<'a>>,

    #[serde(rename = "method")]
    pub method: Option<&'a str>,

    #[serde(rename = "proxy")]
    pub proxy: Option<ProxyConfig<'a>>,

    #[serde(rename = "url")]
    pub url: Option<Url<'a>>,
}

/// This field contains the data usually contained in the request body.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Body<'a> {
    /// When set to true, prevents request body from being sent.
    #[serde(rename = "disabled")]
    pub disabled: Option<bool>,

    #[serde(rename = "file")]
    pub file: Option<File<'a>>,

    #[serde(rename = "formdata")]
    pub formdata: Option<Vec<FormParameter<'a>>>,

    /// Postman stores the type of data associated with this request in this field.
    #[serde(rename = "mode")]
    pub mode: Option<Mode>,

    #[serde(rename = "raw")]
    pub raw: Option<&'a str>,

    #[serde(rename = "options")]
    pub options: Option<BodyOptions<'a>>,

    #[serde(rename = "urlencoded")]
    pub urlencoded: Option<Vec<UrlEncodedParameter<'a>>>,

    #[serde(rename = "graphql")]
    pub graphql: Option<GraphQlBody<'a>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct BodyOptions<'a> {
    #[serde(borrow, rename = "raw")]
    pub raw: Option<RawOptions<'a>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct RawOptions<'a> {
    #[serde(rename = "language")]
    pub language: Option<&'a str>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct File<'a> {
    #[serde(rename = "content")]
    pub content: Option<&'a str>,

    #[serde(rename = "src")]
    pub src: Option<&'a str>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct FormParameter<'a> {
    /// Override Content-Type header of this form data entity.
    #[serde(rename = "contentType")]
    pub content_type: Option<&'a str>,

    #[serde(rename = "description")]
    pub description: Option<DescriptionUnion<'a>>,

    /// When set to true, prevents this form data entity from being sent.
    #[serde(rename = "disabled")]
    pub disabled: Option<bool>,

    #[serde(rename = "key")]
    pub key: &'a str,

    #[serde(rename = "type")]
    pub form_parameter_type: Option<&'a str>,

    #[serde(rename = "value")]
    pub value: Option<&'a str>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct UrlEncodedParameter<'a> {
    #[serde(rename = "description")]
    pub description: Option<DescriptionUnion<'a>>,

    #[serde(rename = "disabled")]
    pub disabled: Option<bool>,

    #[serde(rename = "key")]
    pub key: &'a str,

    #[serde(rename = "value")]
    pub value: Option<&'a str>,
}

/// A representation of an ssl certificate
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Certificate<'a> {
    /// An object containing path to file certificate, on the file system
    #[serde(rename = "cert")]
    pub cert: Option<Cert>,

    /// An object containing path to file containing private key, on the file system
    #[serde(rename = "key")]
    pub key: Option<Key>,

    /// A list of Url match pattern strings, to identify Urls this certificate can be used for.
    #[serde(rename = "matches")]
    pub matches: Option<Vec<Option<serde_json::Value>>>,

    /// A name for the certificate for user reference
    #[serde(rename = "name")]
    pub name: Option<&'a str>,

    /// The passphrase for the certificate
    #[serde(rename = "passphrase")]
    pub passphrase: Option<&'a str>,
}

/// An object containing path to file certificate, on the file system
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Cert {
    /// The path to file containing key for certificate, on the file system
    #[serde(rename = "src")]
    pub src: Option<serde_json::Value>,
}

/// An object containing path to file containing private key, on the file system
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Key {
    /// The path to file containing key for certificate, on the file system
    #[serde(rename = "src")]
    pub src: Option<serde_json::Value>,
}

/// A representation for a list of headers
///
/// Represents a single HTTP Header
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Header<'a> {
    #[serde(rename = "description")]
    pub description: Option<DescriptionUnion<'a>>,

    /// If set to true, the current header will not be sent with requests.
    #[serde(rename = "disabled")]
    pub disabled: Option<bool>,

    /// This holds the LHS of the HTTP Header, e.g ``Content-Type`` or ``X-Custom-Header``
    #[serde(rename = "key")]
    pub key: &'a str,

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
    #[serde(rename = "host")]
    pub host: Option<&'a str>,

    /// The Url match for which the proxy config is defined
    #[serde(rename = "match")]
    pub proxy_config_match: Option<&'a str>,

    /// The proxy server port
    #[serde(rename = "port")]
    pub port: Option<i64>,

    /// The tunneling details for the proxy config
    #[serde(rename = "tunnel")]
    pub tunnel: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ResponseClass<'a> {
    /// The name of the response.
    #[serde(rename = "name")]
    pub name: Option<&'a str>,

    /// The raw text of the response.
    #[serde(rename = "body")]
    pub body: Option<&'a str>,

    /// The numerical response code, example: 200, 201, 404, etc.
    #[serde(rename = "code")]
    pub code: Option<i64>,

    #[serde(rename = "cookie")]
    pub cookie: Option<Vec<Cookie<'a>>>,

    #[serde(rename = "header")]
    pub header: Option<Headers<'a>>,

    /// A unique, user defined identifier that can  be used to refer to this response from
    /// requests.
    #[serde(rename = "id")]
    pub id: Option<&'a str>,

    #[serde(rename = "originalRequest")]
    pub original_request: Option<RequestClass<'a>>,

    /// The time taken by the request to complete. If a number, the unit is milliseconds. If the
    /// response is manually created, this can be set to `null`.
    #[serde(rename = "responseTime")]
    pub response_time: Option<ResponseTime<'a>>,

    /// The response status, e.g: '200 OK'
    #[serde(rename = "status")]
    pub status: Option<&'a str>,
}

/// A Cookie, that follows the [Google Chrome
/// format](https://developer.chrome.com/extensions/cookies)
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Cookie<'a> {
    /// The domain for which this cookie is valid.
    #[serde(rename = "domain")]
    pub domain: Option<&'a str>,

    /// When the cookie expires.
    #[serde(rename = "expires")]
    pub expires: Option<&'a str>,

    /// Custom attributes for a cookie go here, such as the [Priority
    /// Field](https://code.google.com/p/chromium/issues/detail?id=232693)
    #[serde(rename = "extensions")]
    pub extensions: Option<Vec<Option<serde_json::Value>>>,

    /// True if the cookie is a host-only cookie. (i.e. a request's URL domain must exactly match
    /// the domain of the cookie).
    #[serde(rename = "hostOnly")]
    pub host_only: Option<bool>,

    /// Indicates if this cookie is HTTP Only. (if True, the cookie is inaccessible to
    /// client-side scripts)
    #[serde(rename = "httpOnly")]
    pub http_only: Option<bool>,

    #[serde(rename = "maxAge")]
    pub max_age: Option<&'a str>,

    /// This is the name of the Cookie.
    #[serde(rename = "name")]
    pub name: Option<&'a str>,

    /// The path associated with the Cookie.
    #[serde(rename = "path")]
    pub path: Option<&'a str>,

    /// Indicates if the 'secure' flag is set on the Cookie, meaning that it is transmitted over
    /// secure connections only. (typically HTTPS)
    #[serde(rename = "secure")]
    pub secure: Option<bool>,

    /// True if the cookie is a session cookie.
    #[serde(rename = "session")]
    pub session: Option<bool>,

    /// The value of the Cookie.
    #[serde(rename = "value")]
    pub value: Option<&'a str>,
}

/// The host for the URL, E.g: api.yourdomain.com. Can be stored as a string or as an array
/// of strings.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Host<'a> {
    String(&'a str),

    StringArray(Vec<&'a str>),
}

/// If object, contains the complete broken-down URL for this request. If string, contains
/// the literal request URL.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Url<'a> {
    String(&'a str),

    UrlClass(UrlClass<'a>),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum UrlPath<'a> {
    String(&'a str),

    UnionArray(Vec<PathElement<'a>>),
}

/// The complete path of the current url, broken down into segments. A segment could be a
/// string, or a path variable.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum PathElement<'a> {
    PathClass(PathClass<'a>),

    String(&'a str),
}

/// A Description can be a raw text, or be an object, which holds the description along with
/// its format.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum DescriptionUnion<'a> {
    #[serde(borrow)]
    Description(Description<'a>),

    String(&'a str),
}

impl<'a> From<&'a DescriptionUnion<'a>> for &'a str {
    fn from(description: &'a DescriptionUnion) -> Self {
        match description {
            DescriptionUnion::Description(desc) => desc.content.unwrap_or_default(),
            DescriptionUnion::String(str) => str,
        }
    }
}

impl<'a> From<&'a DescriptionUnion<'a>> for String {
    fn from(description: &'a DescriptionUnion) -> Self {
        match description {
            DescriptionUnion::Description(desc) => {
                desc.content.map(|s| s.to_string()).unwrap_or_default()
            }
            DescriptionUnion::String(str) => str.to_string(),
        }
    }
}

/// Postman allows you to version your collections as they grow, and this field holds the
/// version number. While optional, it is recommended that you use this field to its fullest
/// extent!
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum CollectionVersion<'a> {
    CollectionVersionClass(CollectionVersionClass<'a>),

    String(&'a str),
}

/// A request represents an HTTP request. If a string, the string is assumed to be the
/// request URL and the method is assumed to be 'GET'.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum RequestUnion<'a> {
    RequestClass(Box<RequestClass<'a>>),

    String(&'a str),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum HeaderUnion<'a> {
    HeaderArray(Vec<Header<'a>>),

    String(&'a str),
}

/// A response represents an HTTP response.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Response<'a> {
    //AnythingArray(Vec<Option<serde_json::Value>>),

    //Bool(bool),

    //Double(f64),

    //Integer(i64),
    ResponseClass(Box<ResponseClass<'a>>),

    String(&'a str),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Headers<'a> {
    String(&'a str),

    UnionArray(Vec<HeaderElement<'a>>),
}

/// No HTTP request is complete without its headers, and the same is true for a Postman
/// request. This field is an array containing all the headers.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum HeaderElement<'a> {
    Header(Header<'a>),

    String(&'a str),
}

/// The time taken by the request to complete. If a number, the unit is milliseconds. If the
/// response is manually created, this can be set to `null`.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ResponseTime<'a> {
    Number(u64),

    String(&'a str),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum GraphQlBody<'a> {
    String(&'a str),

    GraphQlBodyClass(GraphQlBodyClass<'a>),
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
    use super::*;

    #[test]
    fn deserializes_oauth2() {
        let fixture = get_fixture("oauth2-code.postman.json");
        let spec: Spec = serde_json::from_str(&fixture).unwrap();
        let oauth2 = spec.auth.unwrap().oauth2.unwrap();

        assert_eq!(oauth2.grant_type, Oauth2GrantType::AuthorizationCode);
        assert_eq!(
            oauth2.auth_url,
            Some("https://example.com/oauth2/authorization".to_string())
        );
        assert_eq!(
            oauth2.access_token_url,
            Some("https://example.com/oauth2/token".to_string())
        );
    }

    #[test]
    fn deserializes_apikey() {
        let fixture = get_fixture("api-key.postman.json");
        let spec: Spec = serde_json::from_str(fixture.as_str()).unwrap();
        let apikey = spec.auth.unwrap().apikey.unwrap();

        assert_eq!(apikey.key, Some("Authorization".to_string()));
        assert_eq!(apikey.location, ApiKeyLocation::Header);
        assert_eq!(apikey.value, None);
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
