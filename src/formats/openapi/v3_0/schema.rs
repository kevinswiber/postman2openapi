//! Schema specification for [OpenAPI 3.0.0](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.0.md)

use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    hash::{Hash, Hasher},
};

use super::components::{Components, ObjectOrReference};

/// top level document
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct Spec {
    /// This string MUST be the [semantic version number](https://semver.org/spec/v2.0.0.html)
    /// of the
    /// [OpenAPI Specification version](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#versions)
    /// that the OpenAPI document uses. The `openapi` field SHOULD be used by tooling
    /// specifications and clients to interpret the OpenAPI document. This is not related to
    /// the API
    /// [`info.version`](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#infoVersion)
    /// string.
    pub openapi: String,
    /// Provides metadata about the API. The metadata MAY be used by tooling as required.
    pub info: Info,
    /// An array of Server Objects, which provide connectivity information to a target server.
    /// If the `servers` property is not provided, or is an empty array, the default value would
    /// be a
    /// [Server Object](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#serverObject)
    /// with a
    /// [url](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#serverUrl)
    /// value of `/`.
    // FIXME: Provide a default value as specified in documentation instead of `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub servers: Option<Vec<Server>>,

    /// Holds the relative paths to the individual endpoints and their operations. The path is
    /// appended to the URL from the
    /// [`Server Object`](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#serverObject)
    /// in order to construct the full URL. The Paths MAY be empty, due to
    /// [ACL constraints](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#securityFiltering).
    pub paths: IndexMap<String, PathItem>,

    /// An element to hold various schemas for the specification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Components>,

    /// A declaration of which security mechanisms can be used across the API.
    /// The list of  values includes alternative security requirement objects that can be used.
    /// Only one of the security requirement objects need to be satisfied to authorize a request.
    /// Individual operations can override this definition.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<Vec<SecurityRequirement>>,

    /// A list of tags used by the specification with additional metadata.
    ///The order of the tags can be used to reflect on their order by the parsing tools.
    /// Not all tags that are used by the
    /// [Operation Object](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#operationObject)
    /// must be declared. The tags that are not declared MAY be organized randomly or
    /// based on the tools' logic. Each tag name in the list MUST be unique.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<IndexSet<Tag>>,

    /// Additional external documentation.
    #[serde(skip_serializing_if = "Option::is_none", rename = "externalDocs")]
    pub external_docs: Option<ExternalDoc>,
    // TODO: Add "Specification Extensions" https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#specificationExtensions}
}

/// General information about the API.
///
///
/// See <https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#infoObject>.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Default)]
// #[serde(rename_all = "lowercase")]
pub struct Info {
    /// The title of the application.
    pub title: String,
    /// A short description of the application. CommonMark syntax MAY be used for rich text representation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// A URL to the Terms of Service for the API. MUST be in the format of a URL.
    #[serde(rename = "termsOfService", skip_serializing_if = "Option::is_none")]
    pub terms_of_service: Option<String>,
    /// The version of the OpenAPI document (which is distinct from the [OpenAPI Specification
    /// version](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#oasVersion)
    /// or the API implementation version).
    pub version: String,
    /// The contact information for the exposed API.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<Contact>,
    /// The license information for the exposed API.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<License>,
}

/// Contact information for the exposed API.
///
/// See <https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#contactObject>.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct Contact {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    // TODO: Make sure the email is a valid email
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    // TODO: Add "Specification Extensions" https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#specificationExtensions
}

/// License information for the exposed API.
///
/// See <https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#licenseObject>.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct License {
    /// The license name used for the API.
    pub name: String,
    /// A URL to the license used for the API.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    // TODO: Add "Specification Extensions" https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#specificationExtensions}
}

/// An object representing a Server.
///
/// See <https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#serverObject>.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct Server {
    /// A URL to the target host. This URL supports Server Variables and MAY be relative, to
    /// indicate that the host location is relative to the location where the OpenAPI document
    /// is being served. Variable substitutions will be made when a variable is named
    /// in {brackets}.
    pub url: String,
    /// An optional string describing the host designated by the URL. CommonMark syntax MAY be used for rich text representation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// A map between a variable name and its value. The value is used for substitution in
    /// the server's URL template.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<BTreeMap<String, ServerVariable>>,
}

/// An object representing a Server Variable for server URL template substitution.
///
/// See <https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#serverVariableObject>.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct ServerVariable {
    /// The default value to use for substitution, and to send, if an alternate value is not
    /// supplied. Unlike the Schema Object's default, this value MUST be provided by the consumer.
    pub default: String,
    /// An enumeration of string values to be used if the substitution options are from a limited
    /// set.
    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    pub substitutions_enum: Option<Vec<String>>,
    /// An optional description for the server variable. [CommonMark] syntax MAY be used for rich
    /// text representation.
    ///
    /// [CommonMark]: https://spec.commonmark.org/
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Describes the operations available on a single path.
///
/// A Path Item MAY be empty, due to [ACL
/// constraints](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#securityFiltering).
/// The path itself is still exposed to the documentation viewer but they will not know which
/// operations and parameters are available.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct PathItem {
    /// Allows for an external definition of this path item. The referenced structure MUST be
    /// in the format of a
    /// [Path Item Object](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#pathItemObject).
    /// If there are conflicts between the referenced definition and this Path Item's definition,
    /// the behavior is undefined.
    // FIXME: Should this ref be moved to an enum?
    #[serde(skip_serializing_if = "Option::is_none", rename = "$ref")]
    pub reference: Option<String>,

    /// An optional, string summary, intended to apply to all operations in this path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// An optional, string description, intended to apply to all operations in this path.
    /// [CommonMark syntax](http://spec.commonmark.org/) MAY be used for rich text representation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// A definition of a GET operation on this path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get: Option<Operation>,
    /// A definition of a PUT operation on this path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub put: Option<Operation>,
    /// A definition of a POST operation on this path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post: Option<Operation>,
    /// A definition of a DELETE operation on this path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete: Option<Operation>,
    /// A definition of a OPTIONS operation on this path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Operation>,
    /// A definition of a HEAD operation on this path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head: Option<Operation>,
    /// A definition of a PATCH operation on this path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<Operation>,
    /// A definition of a TRACE operation on this path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<Operation>,

    /// An alternative `server` array to service all operations in this path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub servers: Option<Vec<Server>>,

    /// A list of parameters that are applicable for all the operations described under this
    /// path. These parameters can be overridden at the operation level, but cannot be removed
    /// there. The list MUST NOT include duplicated parameters. A unique parameter is defined by
    /// a combination of a
    /// [name](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#parameterName)
    /// and
    /// [location](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#parameterIn).
    /// The list can use the
    /// [Reference Object](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#referenceObject)
    /// to link to parameters that are defined at the
    /// [OpenAPI Object's components/parameters](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#componentsParameters).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Vec<ObjectOrReference<Parameter>>>,
    // TODO: Add "Specification Extensions" https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#specificationExtensions}
}

/// Describes a single API operation on a path.
///
/// See <https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#operationObject>.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Default)]
// #[serde(rename_all = "lowercase")]
pub struct Operation {
    /// A list of tags for API documentation control. Tags can be used for logical grouping of
    /// operations by resources or any other qualifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// A short summary of what the operation does.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// A verbose explanation of the operation behavior.
    /// [CommonMark syntax](http://spec.commonmark.org/) MAY be used for rich text representation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Additional external documentation for this operation.
    #[serde(skip_serializing_if = "Option::is_none", rename = "externalDocs")]
    pub external_docs: Option<ExternalDoc>,
    /// Unique string used to identify the operation. The id MUST be unique among all operations
    /// described in the API. Tools and libraries MAY use the operationId to uniquely identify an
    /// operation, therefore, it is RECOMMENDED to follow common programming naming conventions.
    #[serde(skip_serializing_if = "Option::is_none", rename = "operationId")]
    pub operation_id: Option<String>,

    /// A list of parameters that are applicable for this operation. If a parameter is already
    /// defined at the
    /// [Path Item](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#pathItemParameters),
    /// the new definition will override it but can never remove it. The list MUST NOT
    /// include duplicated parameters. A unique parameter is defined by a combination of a
    /// [name](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#parameterName)
    /// and
    /// [location](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#parameterIn).
    /// The list can use the
    /// [Reference Object](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#referenceObject)
    /// to link to parameters that are defined at the
    /// [OpenAPI Object's components/parameters](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#componentsParameters).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Vec<ObjectOrReference<Parameter>>>,

    /// The request body applicable for this operation. The requestBody is only supported in HTTP methods where the HTTP 1.1 specification RFC7231 has explicitly defined semantics for request bodies. In other cases where the HTTP spec is vague, requestBody SHALL be ignored by consumers.
    #[serde(skip_serializing_if = "Option::is_none", rename = "requestBody")]
    pub request_body: Option<ObjectOrReference<RequestBody>>,

    /// The list of possible responses as they are returned from executing this operation.
    ///
    /// A container for the expected responses of an operation. The container maps a HTTP
    /// response code to the expected response.
    ///
    /// The documentation is not necessarily expected to cover all possible HTTP response codes
    /// because they may not be known in advance. However, documentation is expected to cover
    /// a successful operation response and any known errors.
    ///
    /// The `default` MAY be used as a default response object for all HTTP codes that are not
    /// covered individually by the specification.
    ///
    /// The `Responses Object` MUST contain at least one response code, and it SHOULD be the
    /// response for a successful operation call.
    ///
    /// See <https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#responsesObject>.
    pub responses: BTreeMap<String, Response>,

    /// A map of possible out-of band callbacks related to the parent operation. The key is
    /// a unique identifier for the Callback Object. Each value in the map is a
    /// [Callback Object](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#callbackObject)
    /// that describes a request that may be initiated by the API provider and the
    /// expected responses. The key value used to identify the callback object is
    /// an expression, evaluated at runtime, that identifies a URL to use for the
    /// callback operation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callbacks: Option<BTreeMap<String, Callback>>,

    /// Declares this operation to be deprecated. Consumers SHOULD refrain from usage
    /// of the declared operation. Default value is `false`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<bool>,

    /// A declaration of which security mechanisms can be used for this operation. The list of
    /// values includes alternative security requirement objects that can be used. Only one
    /// of the security requirement objects need to be satisfied to authorize a request.
    /// This definition overrides any declared top-level
    /// [`security`](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#oasSecurity).
    /// To remove a top-level security declaration, an empty array can be used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<Vec<SecurityRequirement>>,

    /// An alternative `server` array to service this operation. If an alternative `server`
    /// object is specified at the Path Item Object or Root level, it will be overridden by
    /// this value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub servers: Option<Vec<Server>>,
}

// FIXME: Verify against OpenAPI 3.0
/// Describes a single operation parameter.
/// A unique parameter is defined by a combination of a
/// [name](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#parameterName)
/// and [location](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#parameterIn).
///
/// See <https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#parameterObject>.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct Parameter {
    /// The name of the parameter.
    pub name: String,
    /// values depend on parameter type
    /// may be `header`, `query`, 'path`, `formData`
    #[serde(rename = "in")]
    pub location: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<Schema>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "uniqueItems")]
    pub unique_items: Option<bool>,
    /// string, number, boolean, integer, array, file ( only for formData )
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    pub param_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    /// A brief description of the parameter. This could contain examples
    /// of use.  GitHub Flavored Markdown is allowed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    // collectionFormat: ???
    // default: ???
    // maximum ?
    // exclusiveMaximum ??
    // minimum ??
    // exclusiveMinimum ??
    // maxLength ??
    // minLength ??
    // pattern ??
    // maxItems ??
    // minItems ??
    // enum ??
    // multipleOf ??
    // allowEmptyValue ( for query / body params )
    /// Describes how the parameter value will be serialized depending on the type of the parameter
    /// value. Default values (based on value of in): for `query` - `form`; for `path` - `simple`; for
    /// `header` - `simple`; for cookie - `form`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<ParameterStyle>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ParameterStyle {
    Form,
    Simple,
}

// FIXME: Verify against OpenAPI 3.0
/// The Schema Object allows the definition of input and output data types.
/// These types can be objects, but also primitives and arrays.
/// This object is an extended subset of the
/// [JSON Schema Specification Wright Draft 00](http://json-schema.org/).
/// For more information about the properties, see
/// [JSON Schema Core](https://tools.ietf.org/html/draft-wright-json-schema-00) and
/// [JSON Schema Validation](https://tools.ietf.org/html/draft-wright-json-schema-validation-00).
/// Unless stated otherwise, the property definitions follow the JSON Schema.
///
/// See <https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#schemaObject>.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct Schema {
    /// [JSON reference](https://tools.ietf.org/html/draft-pbryan-zyp-json-ref-03)
    /// path to another definition
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "$ref")]
    pub ref_path: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    pub schema_type: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "enum")]
    pub enum_values: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<Schema>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<BTreeMap<String, Schema>>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "readOnly")]
    pub read_only: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub nullable: Option<bool>,

    // FIXME: Why can this be a "boolean" (as per the spec)? It doesn't make sense. Here it's not.
    /// Value can be boolean or object. Inline or referenced schema MUST be of a
    /// [Schema Object](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#schemaObject)
    /// and not a standard JSON Schema.
    ///
    /// See <https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#properties>.
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "additionalProperties"
    )]
    pub additional_properties: Option<ObjectOrReference<Box<Schema>>>,

    /// A free-form property to include an example of an instance for this schema.
    /// To represent examples that cannot be naturally represented in JSON or YAML,
    /// a string value can be used to contain the example with escaping where necessary.
    /// NOTE: According to [spec], _Primitive data types in the OAS are based on the
    ///       types supported by the JSON Schema Specification Wright Draft 00._
    ///       This suggest using
    ///       [`serde_json::Value`](https://docs.serde.rs/serde_json/value/enum.Value.html). [spec][https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#data-types]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<serde_json::value::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    // The following properties are taken directly from the JSON Schema definition and
    // follow the same specifications:
    // multipleOf
    // maximum
    // exclusiveMaximum
    // minimum
    // exclusiveMinimum
    // maxLength
    // minLength
    // pattern (This string SHOULD be a valid regular expression, according to the ECMA 262 regular expression dialect)
    // maxItems
    // minItems
    // uniqueItems
    // maxProperties
    // minProperties
    // required
    // enum

    // The following properties are taken from the JSON Schema definition but their
    // definitions were adjusted to the OpenAPI Specification.
    // - type - Value MUST be a string. Multiple types via an array are not supported.
    // - allOf - Inline or referenced schema MUST be of a [Schema Object](#schemaObject) and not a standard JSON Schema.
    // - oneOf - Inline or referenced schema MUST be of a [Schema Object](#schemaObject) and not a standard JSON Schema.
    // - anyOf - Inline or referenced schema MUST be of a [Schema Object](#schemaObject) and not a standard JSON Schema.
    // - not - Inline or referenced schema MUST be of a [Schema Object](#schemaObject) and not a standard JSON Schema.
    // - items - Value MUST be an object and not an array. Inline or referenced schema MUST be of a [Schema Object](#schemaObject) and not a standard JSON Schema. `items` MUST be present if the `type` is `array`.
    // - properties - Property definitions MUST be a [Schema Object](#schemaObject) and not a standard JSON Schema (inline or referenced).
    // - additionalProperties - Value can be boolean or object. Inline or referenced schema MUST be of a [Schema Object](#schemaObject) and not a standard JSON Schema.
    // - description - [CommonMark syntax](http://spec.commonmark.org/) MAY be used for rich text representation.
    // - format - See [Data Type Formats](#dataTypeFormat) for further details. While relying on JSON Schema's defined formats, the OAS offers a few additional predefined formats.
    // - default - The default value represents what would be assumed by the consumer of the input as the value of the schema if one is not provided. Unlike JSON Schema, the value MUST conform to the defined type for the Schema Object defined at the same level. For example, if `type` is `string`, then `default` can be `"foo"` but cannot be `1`.
    /// The default value represents what would be assumed by the consumer of the input as the value
    /// of the schema if one is not provided. Unlike JSON Schema, the value MUST conform to the
    /// defined type for the Schema Object defined at the same level. For example, if type is
    /// `string`, then `default` can be `"foo"` but cannot be `1`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<serde_json::Value>,

    /// Inline or referenced schema MUST be of a [Schema Object](#schemaObject) and not a standard
    /// JSON Schema.
    /// [allOf](https://swagger.io/docs/specification/data-models/oneof-anyof-allof-not/#allof)
    #[serde(rename = "allOf", skip_serializing_if = "Option::is_none")]
    pub all_of: Option<Vec<ObjectOrReference<Schema>>>,

    /// Inline or referenced schema MUST be of a [Schema Object](#schemaObject) and not a standard
    /// JSON Schema.
    /// [oneOf](https://swagger.io/docs/specification/data-models/oneof-anyof-allof-not/#oneof)
    #[serde(rename = "oneOf", skip_serializing_if = "Option::is_none")]
    pub one_of: Option<Vec<ObjectOrReference<Schema>>>,

    /// Inline or referenced schema MUST be of a [Schema Object](#schemaObject) and not a standard
    /// JSON Schema.
    /// [anyOf](https://swagger.io/docs/specification/data-models/oneof-anyof-allof-not/#anyof)
    #[serde(rename = "anyOf", skip_serializing_if = "Option::is_none")]
    pub any_of: Option<Vec<ObjectOrReference<Schema>>>,

    /// Inline or referenced schema MUST be of a [Schema Object](#schemaObject) and not a standard
    /// JSON Schema.
    /// [not](https://swagger.io/docs/specification/data-models/oneof-anyof-allof-not/#not)
    #[serde(rename = "not", skip_serializing_if = "Option::is_none")]
    pub not: Option<Vec<ObjectOrReference<Schema>>>,

    #[serde(rename = "maxLength", skip_serializing_if = "Option::is_none")]
    pub max_length: Option<u32>,

    #[serde(rename = "minLength", skip_serializing_if = "Option::is_none")]
    pub min_length: Option<u32>,

    /// [Specification extensions](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.2.md#specificationExtensions)
    #[serde(flatten)]
    pub extensions: HashMap<String, String>,
}

/// Describes a single response from an API Operation, including design-time, static `links`
/// to operations based on the response.
///
/// See <https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#responseObject>.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct Response {
    /// A short description of the response.
    /// [CommonMark syntax](http://spec.commonmark.org/) MAY be used for rich text representation.
    pub description: Option<String>,

    /// Maps a header name to its definition.
    /// [RFC7230](https://tools.ietf.org/html/rfc7230#page-22) states header names are case
    /// insensitive. If a response header is defined with the name `"Content-Type"`, it SHALL
    /// be ignored.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<BTreeMap<String, ObjectOrReference<Header>>>,

    /// A map containing descriptions of potential response payloads. The key is a media type
    /// or [media type range](https://tools.ietf.org/html/rfc7231#appendix-D) and the value
    /// describes it. For responses that match multiple keys, only the most specific key is
    /// applicable. e.g. text/plain overrides text/*
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<BTreeMap<String, MediaType>>,

    /// A map of operations links that can be followed from the response. The key of the map
    /// is a short name for the link, following the naming constraints of the names for
    /// [Component Objects](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#componentsObject).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<BTreeMap<String, ObjectOrReference<Link>>>,
    // TODO: Add "Specification Extensions" https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#specificationExtensions}
}

/// The Header Object follows the structure of the
/// [Parameter Object](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#parameterObject)
/// with the following changes:
/// 1. `name` MUST NOT be specified, it is given in the corresponding `headers` map.
/// 1. `in` MUST NOT be specified, it is implicitly in `header`.
/// 1. All traits that are affected by the location MUST be applicable to a location of
///    `header` (for example, [`style`](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#parameterStyle)).
///
/// See <https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#headerObject>.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct Header {
    // FIXME: Is the third change properly implemented?
    // FIXME: Merge `ObjectOrReference<Header>::Reference` and `ParameterOrRef::Reference`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<Schema>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "uniqueItems")]
    pub unique_items: Option<bool>,
    /// string, number, boolean, integer, array, file ( only for formData )
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    pub param_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    /// A brief description of the parameter. This could contain examples
    /// of use.  GitHub Flavored Markdown is allowed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    // collectionFormat: ???
    // default: ???
    // maximum ?
    // exclusiveMaximum ??
    // minimum ??
    // exclusiveMinimum ??
    // maxLength ??
    // minLength ??
    // pattern ??
    // maxItems ??
    // minItems ??
    // enum ??
    // multipleOf ??
    // allowEmptyValue ( for query / body params )
}

/// Describes a single request body.
///
/// See <https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#requestBodyObject>.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct RequestBody {
    /// A brief description of the request body. This could contain examples of use.
    /// [CommonMark syntax](http://spec.commonmark.org/) MAY be used for rich text representation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The content of the request body. The key is a media type or
    /// [media type range](https://tools.ietf.org/html/rfc7231#appendix-D) and the
    /// value describes it. For requests that match multiple keys, only the most specific key
    /// is applicable. e.g. text/plain overrides text/*
    pub content: BTreeMap<String, MediaType>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
}

/// The Link object represents a possible design-time link for a response.
///
/// The presence of a link does not guarantee the caller's ability to successfully invoke it,
/// rather it provides a known relationship and traversal mechanism between responses and
/// other operations.
///
/// Unlike _dynamic_ links (i.e. links provided *in* the response payload), the OAS linking
/// mechanism does not require link information in the runtime response.
///
/// For computing links, and providing instructions to execute them, a
/// [runtime expression](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#runtimeExpression)
/// is used for accessing values in an operation and using them as parameters while invoking
/// the linked operation.
///
/// See <https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#linkObject>.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Link {
    /// A relative or absolute reference to an OAS operation. This field is mutually exclusive
    /// of the `operationId` field, and MUST point to an
    /// [Operation Object](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#operationObject).
    /// Relative `operationRef` values MAY be used to locate an existing
    /// [Operation Object](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#operationObject)
    /// in the OpenAPI definition.
    Ref {
        #[serde(rename = "operationRef")]
        operation_ref: String,

        // FIXME: Implement
        // /// A map representing parameters to pass to an operation as specified with `operationId`
        // /// or identified via `operationRef`. The key is the parameter name to be used, whereas
        // /// the value can be a constant or an expression to be evaluated and passed to the
        // /// linked operation. The parameter name can be qualified using the
        // /// [parameter location](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#parameterIn)
        // /// `[{in}.]{name}` for operations that use the same parameter name in different
        // /// locations (e.g. path.id).
        // parameters: BTreeMap<String, Any | {expression}>,
        #[serde(skip_serializing_if = "Option::is_none")]
        parameters: Option<BTreeMap<String, String>>,

        // FIXME: Implement
        // /// A literal value or
        // /// [{expression}](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#runtimeExpression)
        // /// to use as a request body when calling the target operation.
        // #[serde(rename = "requestBody")]
        // request_body: Any | {expression}
        /// A description of the link. [CommonMark syntax](http://spec.commonmark.org/) MAY be
        /// used for rich text representation.
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,

        /// A server object to be used by the target operation.
        #[serde(skip_serializing_if = "Option::is_none")]
        server: Option<Server>,
        // TODO: Add "Specification Extensions" https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#specificationExtension
    },
    /// The name of an _existing_, resolvable OAS operation, as defined with a unique
    /// `operationId`. This field is mutually exclusive of the `operationRef` field.
    Id {
        #[serde(rename = "operationId")]
        operation_id: String,

        // FIXME: Implement
        // /// A map representing parameters to pass to an operation as specified with `operationId`
        // /// or identified via `operationRef`. The key is the parameter name to be used, whereas
        // /// the value can be a constant or an expression to be evaluated and passed to the
        // /// linked operation. The parameter name can be qualified using the
        // /// [parameter location](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#parameterIn)
        // /// `[{in}.]{name}` for operations that use the same parameter name in different
        // /// locations (e.g. path.id).
        // parameters: BTreeMap<String, Any | {expression}>,
        #[serde(skip_serializing_if = "Option::is_none")]
        parameters: Option<BTreeMap<String, String>>,

        // FIXME: Implement
        // /// A literal value or
        // /// [{expression}](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#runtimeExpression)
        // /// to use as a request body when calling the target operation.
        // #[serde(rename = "requestBody")]
        // request_body: Any | {expression}
        /// A description of the link. [CommonMark syntax](http://spec.commonmark.org/) MAY be
        /// used for rich text representation.
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,

        /// A server object to be used by the target operation.
        #[serde(skip_serializing_if = "Option::is_none")]
        server: Option<Server>,
        // TODO: Add "Specification Extensions" https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#specificationExtension
    },
}

/// Each Media Type Object provides schema and examples for the media type identified by its key.
///
/// See <https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#media-type-object>.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct MediaType {
    /// The schema defining the type used for the request body.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<ObjectOrReference<Schema>>,

    /// Example of the media type.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub examples: Option<MediaTypeExample>,

    /// A map between a property name and its encoding information. The key, being the
    /// property name, MUST exist in the schema as a property. The encoding object SHALL
    /// only apply to `requestBody` objects when the media type is `multipart`
    /// or `application/x-www-form-urlencoded`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<BTreeMap<String, Encoding>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum MediaTypeExample {
    /// Example of the media type. The example object SHOULD be in the correct format as
    /// specified by the media type. The `example` field is mutually exclusive of the
    /// `examples` field. Furthermore, if referencing a `schema` which contains an example,
    /// the `example` value SHALL override the example provided by the schema.
    Example { example: serde_json::Value },
    /// Examples of the media type. Each example object SHOULD match the media type and
    /// specified schema if present. The `examples` field is mutually exclusive of
    /// the `example` field. Furthermore, if referencing a `schema` which contains an
    /// example, the `examples` value SHALL override the example provided by the schema.
    Examples {
        examples: BTreeMap<String, ObjectOrReference<Example>>,
    },
}

/// A single encoding definition applied to a single schema property.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct Encoding {
    /// The Content-Type for encoding a specific property. Default value depends on the
    /// property type: for `string` with `format` being `binary` – `application/octet-stream`;
    /// for other primitive types – `text/plain`; for `object` - `application/json`;
    /// for `array` – the default is defined based on the inner type. The value can be a
    /// specific media type (e.g. `application/json`), a wildcard media type
    /// (e.g. `image/*`), or a comma-separated list of the two types.
    #[serde(skip_serializing_if = "Option::is_none", rename = "contentType")]
    pub content_type: Option<String>,

    /// A map allowing additional information to be provided as headers, for example
    /// `Content-Disposition`.  `Content-Type` is described separately and SHALL be
    /// ignored in this section. This property SHALL be ignored if the request body
    /// media type is not a `multipart`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<BTreeMap<String, ObjectOrReference<Header>>>,

    /// Describes how a specific property value will be serialized depending on its type.
    /// See [Parameter Object](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#parameterObject)
    /// for details on the
    /// [`style`](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#parameterStyle)
    /// property. The behavior follows the same values as `query` parameters, including
    /// default values. This property SHALL be ignored if the request body media type
    /// is not `application/x-www-form-urlencoded`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,

    /// When this is true, property values of type `array` or `object` generate
    /// separate parameters for each value of the array, or key-value-pair of the map.
    /// For other types of properties this property has no effect. When
    /// [`style`](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#encodingStyle)
    /// is `form`, the default value is `true`. For all other styles, the default value
    /// is `false`. This property SHALL be ignored if the request body media type is
    /// not `application/x-www-form-urlencoded`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explode: Option<bool>,

    /// Determines whether the parameter value SHOULD allow reserved characters, as defined
    /// by [RFC3986](https://tools.ietf.org/html/rfc3986#section-2.2) `:/?#[]@!$&'()*+,;=`
    /// to be included without percent-encoding. The default value is `false`. This
    /// property SHALL be ignored if the request body media type is
    /// not `application/x-www-form-urlencoded`.
    #[serde(skip_serializing_if = "Option::is_none", rename = "allowReserved")]
    pub allow_reserved: Option<bool>,
}

/// See <https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#exampleObject>.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct Example {
    /// Short description for the example.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,

    /// Long description for the example.
    /// [CommonMark syntax](http://spec.commonmark.org/) MAY be used for rich text representation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    // FIXME: Implement (merge with externalValue as enum)
    /// Embedded literal example. The `value` field and `externalValue` field are mutually
    /// exclusive. To represent examples of media types that cannot naturally represented
    /// in JSON or YAML, use a string value to contain the example, escaping where necessary.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
    // FIXME: Implement (merge with value as enum)
    // /// A URL that points to the literal example. This provides the capability to reference
    // /// examples that cannot easily be included in JSON or YAML documents. The `value` field
    // /// and `externalValue` field are mutually exclusive.
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub externalValue: Option<String>,

    // TODO: Add "Specification Extensions" https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#specificationExtensions}
}

/// Defines a security scheme that can be used by the operations. Supported schemes are
/// HTTP authentication, an API key (either as a header or as a query parameter),
///OAuth2's common flows (implicit, password, application and access code) as defined
/// in [RFC6749](https://tools.ietf.org/html/rfc6749), and
/// [OpenID Connect Discovery](https://tools.ietf.org/html/draft-ietf-oauth-discovery-06).
///
/// See <https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#securitySchemeObject>.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum SecurityScheme {
    #[serde(rename = "apiKey")]
    ApiKey {
        name: String,
        #[serde(rename = "in")]
        location: String,
    },
    #[serde(rename = "http")]
    Http {
        scheme: String,
        #[serde(rename = "bearerFormat", skip_serializing_if = "Option::is_none")]
        bearer_format: Option<String>,
    },
    #[serde(rename = "oauth2")]
    OAuth2 { flows: Box<Flows> },
    #[serde(rename = "openIdConnect")]
    OpenIdConnect {
        #[serde(rename = "openIdConnectUrl")]
        open_id_connect_url: String,
    },
}

/// Allows configuration of the supported OAuth Flows.
/// See [link]
/// [link][https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#oauth-flows-object]
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Flows {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implicit: Option<ImplicitFlow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<PasswordFlow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_credentials: Option<ClientCredentialsFlow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_code: Option<AuthorizationCodeFlow>,
}

/// Configuration details for a implicit OAuth Flow
/// See [link]
/// [link](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#oauth-flow-object)
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ImplicitFlow {
    pub authorization_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_url: Option<String>,
    pub scopes: BTreeMap<String, String>,
}

/// Configuration details for a password OAuth Flow
/// See [link]
/// [link](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#oauth-flow-object
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PasswordFlow {
    pub token_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_url: Option<String>,
    pub scopes: BTreeMap<String, String>,
}

/// Configuration details for a client credentials OAuth Flow
/// See [link]
/// [link](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#oauth-flow-object
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ClientCredentialsFlow {
    pub token_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_url: Option<String>,
    pub scopes: BTreeMap<String, String>,
}

/// Configuration details for a authorization code OAuth Flow
/// See [link]
/// [link](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#oauth-flow-object
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizationCodeFlow {
    pub authorization_url: String,
    pub token_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_url: Option<String>,
    pub scopes: BTreeMap<String, String>,
}

// TODO: Implement
/// A map of possible out-of band callbacks related to the parent operation. Each value in
/// the map is a Path Item Object that describes a set of requests that may be initiated by
/// the API provider and the expected responses. The key value used to identify the callback
/// object is an expression, evaluated at runtime, that identifies a URL to use for the
/// callback operation.
///
/// See <https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#callbackObject>.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct Callback(
    /// A Path Item Object used to define a callback request and expected responses.
    serde_json::Value, // TODO: Add "Specification Extensions" https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#specificationExtensions}
);

/// Allows configuration of the supported OAuth Flows.
/// https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#oauthFlowsObject
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct OAuthFlows {}

/// Adds metadata to a single tag that is used by the
/// [Operation Object](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#operationObject).
/// It is not mandatory to have a Tag Object per tag defined in the Operation Object instances.
///
/// See <https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#tagObject>.
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct Tag {
    /// The name of the tag.
    pub name: String,

    /// A short description for the tag.
    /// [CommonMark syntax](http://spec.commonmark.org/) MAY be used for rich text representation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    // /// Additional external documentation for this tag.
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub external_docs: Option<Vec<ExternalDoc>>,

    // TODO: Add "Specification Extensions" https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#specificationExtensions}
}

impl Hash for Tag {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}
impl PartialEq for Tag {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
impl Eq for Tag {}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct SecurityRequirement {
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub requirement: Option<BTreeMap<String, Vec<String>>>,
}

/// Allows referencing an external resource for extended documentation.
///
/// See <https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#externalDocumentationObject>.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ExternalDoc {
    /// The URL for the target documentation.
    pub url: String,

    /// A short description of the target documentation.
    /// [CommonMark syntax](http://spec.commonmark.org/) MAY be used for rich text representation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    // TODO: Add "Specification Extensions" https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#specificationExtensions}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_scheme_oauth_deser() {
        const IMPLICIT_OAUTH2_SAMPLE: &str = r#"{
          "type": "oauth2",
          "flows": {
            "implicit": {
              "authorizationUrl": "https://example.com/api/oauth/dialog",
              "scopes": {
                "write:pets": "modify pets in your account",
                "read:pets": "read your pets"
              }
            },
            "authorizationCode": {
              "authorizationUrl": "https://example.com/api/oauth/dialog",
              "tokenUrl": "https://example.com/api/oauth/token",
              "scopes": {
                "write:pets": "modify pets in your account",
                "read:pets": "read your pets"
              }
            }
          }
        }"#;
        let obj: SecurityScheme = serde_json::from_str(IMPLICIT_OAUTH2_SAMPLE).unwrap();
        match obj {
            SecurityScheme::OAuth2 { flows } => {
                assert!(flows.implicit.is_some());
                let implicit = flows.implicit.unwrap();
                assert_eq!(
                    implicit.authorization_url,
                    "https://example.com/api/oauth/dialog".to_string()
                );
                assert!(implicit.scopes.contains_key("write:pets"));
                assert!(implicit.scopes.contains_key("read:pets"));

                assert!(flows.authorization_code.is_some());
                let auth_code = flows.authorization_code.unwrap();
                assert_eq!(
                    auth_code.authorization_url,
                    "https://example.com/api/oauth/dialog".to_string()
                );
                assert_eq!(
                    auth_code.token_url,
                    "https://example.com/api/oauth/token".to_string()
                );
                assert!(implicit.scopes.contains_key("write:pets"));
                assert!(implicit.scopes.contains_key("read:pets"));
            }
            _ => panic!("wrong security scheme type"),
        }
    }
}
