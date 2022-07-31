use super::schema::{
    Callback, Example, Header, Link, Parameter, RequestBody, Response, Schema, SecurityScheme,
};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ObjectOrReference<T> {
    Object(T),
    Ref {
        #[serde(rename = "$ref")]
        ref_path: String,
    },
}

/// Holds a set of reusable objects for different aspects of the OAS.
///
/// All objects defined within the components object will have no effect on the API unless
/// they are explicitly referenced from properties outside the components object.
///
/// See <https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#componentsObject>.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct Components {
    /// An object to hold reusable Schema Objects.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schemas: Option<BTreeMap<String, ObjectOrReference<Schema>>>,

    /// An object to hold reusable Response Objects.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub responses: Option<BTreeMap<String, ObjectOrReference<Response>>>,

    /// An object to hold reusable Parameter Objects.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<BTreeMap<String, ObjectOrReference<Parameter>>>,

    /// An object to hold reusable Example
    #[serde(skip_serializing_if = "Option::is_none")]
    pub examples: Option<BTreeMap<String, ObjectOrReference<Example>>>,

    /// An object to hold reusable Request Body Objects.
    #[serde(skip_serializing_if = "Option::is_none", rename = "requestBodies")]
    pub request_bodies: Option<BTreeMap<String, ObjectOrReference<RequestBody>>>,

    /// An object to hold reusable Header Objects.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<BTreeMap<String, ObjectOrReference<Header>>>,

    /// An object to hold reusable Security Scheme Objects.
    #[serde(skip_serializing_if = "Option::is_none", rename = "securitySchemes")]
    pub security_schemes: Option<BTreeMap<String, ObjectOrReference<SecurityScheme>>>,

    /// An object to hold reusable Link Objects.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<BTreeMap<String, ObjectOrReference<Link>>>,

    /// An object to hold reusable Callback Objects.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callbacks: Option<BTreeMap<String, ObjectOrReference<Callback>>>,
    // TODO: Add "Specification Extensions" https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.0.1.md#specificationExtensions}
}
