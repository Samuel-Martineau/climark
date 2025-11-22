pub mod error;
mod upload;

use chrono::{DateTime, Utc};
use error::CrowdmarkError;
use regex::Regex;
use reqwest::header;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

static DEFAULT_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

#[derive(Debug)]
pub struct Client {
    client: reqwest::Client,
    csrf: String,
}

#[non_exhaustive]
#[derive(Debug, Serialize)]
pub struct Course {
    pub id: String,
    pub name: String,
    pub archived: bool,
    pub assessment_count: usize,
}

#[non_exhaustive]
#[derive(Debug, Serialize)]
pub struct Assessment {
    pub id: String,
    pub title: String,
    pub kind: AssessmentKind,
    pub due: Option<DateTime<Utc>>,
    pub submitted: Option<DateTime<Utc>>,
    pub graded: Option<DateTime<Utc>>,
    pub score: Option<f32>,
}

#[non_exhaustive]
#[derive(Debug, Serialize)]
pub enum AssessmentKind {
    TakeHome,
    Proctored,
}

#[derive(Debug, Deserialize)]
struct ResponseRoot<DA, DR, I> {
    data: Vec<ResponseData<DA, DR>>,
    included: Vec<IncludedData<I>>,
}

#[derive(Debug, Deserialize)]
struct ResponseData<A, R> {
    id: String,

    #[serde(flatten)]
    attributes: A,
    relationships: R,
}

#[derive(Debug, Deserialize)]
struct IncludedData<A> {
    id: String,

    #[serde(flatten)]
    attributes: A,
}

#[derive(Debug, Deserialize)]
struct RelationshipId {
    id: String,
}

#[derive(Debug, Deserialize)]
struct OptionalData<T> {
    data: Option<T>,
}

#[derive(Debug, Deserialize)]
struct RequiredData<T> {
    data: T,
}

#[derive(Debug, serde::Deserialize)]
struct EmptyStruct {}

fn from_raw_normalized_points<'de, D>(deserializer: D) -> Result<Option<f32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum RawNormalizedPoints {
        #[allow(dead_code)]
        Zero(usize),
        Str(String),
    }

    match RawNormalizedPoints::deserialize(deserializer)? {
        RawNormalizedPoints::Zero(_) => Ok(None),
        RawNormalizedPoints::Str(s) => s.parse::<f32>().map(Some).map_err(serde::de::Error::custom),
    }
}

impl Client {
    /// Creates a new [`Client`] instance using the provided session token.
    ///
    /// # Arguments
    ///
    /// * `session_token` - The Crowdmark session token
    ///
    /// # Returns
    ///
    /// Returns a [`Result`] containing:
    /// * `Ok(Client)` - If the `session_token` is properly formatted.
    /// * `Err(CrowdmarkError)` - If the `session_token` is incorrectly formatted.
    ///
    /// # Errors
    ///
    /// This function returns a [`CrowdmarkError::HeaderError`] if the provided
    /// `session_token` is incorrectly formatted.
    pub async fn new(session_token: &str) -> Result<Self, CrowdmarkError> {
        let mut headers = header::HeaderMap::new();
        let cookie_string = format!("cm_session_id={session_token}");

        let mut cookie_value = header::HeaderValue::from_str(&cookie_string)?;
        cookie_value.set_sensitive(true);
        headers.insert(header::COOKIE, cookie_value);

        let client = reqwest::Client::builder()
            .user_agent(DEFAULT_USER_AGENT)
            .default_headers(headers)
            .build()?;

        let res = client
            .get("https://app.crowdmark.com/student")
            .send()
            .await?;
        let html = res.text().await?;
        let re = Regex::new(r#"<meta\s+name="csrf-token"\s+content="([^"]+)""#)?;
        let csrf = match re.captures(&html) {
            Some(captures) => captures[1].to_string(),
            None => {
                return Err(CrowdmarkError::NotAuthenticated(
                    "Missing CSRF Token".to_string(),
                ));
            }
        };
        Ok(Self { client, csrf })
    }

    /// Retrieves the list of courses available to the authenticated student.
    ///
    /// # Returns
    ///
    /// Returns a [`Result`] containing:
    /// * `Ok(Vec<Course>)` — A vector of [`Course`]
    /// * `Err(CrowdmarkError)` — If the HTTP request fails or the response
    ///   cannot be parsed.
    ///
    /// # Errors
    ///
    /// This function returns a [`CrowdmarkError`] if:
    /// * The request to the Crowdmark API fails.
    /// * The API returns an unexpected response format.
    pub async fn list_courses(&self) -> Result<Vec<Course>, CrowdmarkError> {
        #[derive(Debug, serde::Deserialize)]
        #[serde(tag = "type", content = "attributes", rename_all_fields = "kebab-case")]
        enum ResponseDataItem {
            #[serde(rename = "courses")]
            Course {
                name: String,
                exam_master_count: usize,
            },
        }

        #[derive(Debug, serde::Deserialize)]
        #[serde(rename_all = "kebab-case")]
        struct ResponseRelationship {
            course_archivation: OptionalData<EmptyStruct>,
        }

        let resp = self
            .client
            .get("https://app.crowdmark.com/api/v2/student/courses?include[]=course-archivation")
            .send()
            .await?;

        if resp.status() == reqwest::StatusCode::FOUND {
            return Err(CrowdmarkError::NotAuthenticated(
                "Could not get courses".to_string(),
            ));
        }

        let data = resp
            .json::<ResponseRoot<ResponseDataItem, ResponseRelationship, EmptyStruct>>()
            .await?
            .data;

        let courses: Vec<_> = data
            .into_iter()
            .map(
                |ResponseData {
                     id,
                     attributes,
                     relationships,
                 }| match attributes {
                    ResponseDataItem::Course {
                        name,
                        exam_master_count,
                    } => Course {
                        id,
                        name,
                        assessment_count: exam_master_count,
                        archived: relationships.course_archivation.data.is_some(),
                    },
                },
            )
            .collect();

        Ok(courses)
    }

    /// Retrieves the list of assessments for `course_id`.
    ///
    /// # Arguments
    ///
    /// * `course_id` - The course ID to retrieve assessments for.
    ///
    /// # Returns
    ///
    /// Returns a [`Result`] containing:
    /// * `Ok(Vec<Course>)` — A vector of [`Course`]
    /// * `Err(CrowdmarkError)` — If the HTTP request fails or the response
    ///   cannot be parsed.
    ///
    /// # Errors
    ///
    /// This function returns a [`CrowdmarkError`] if:
    /// * The request to the Crowdmark API fails.
    /// * The API returns an unexpected response format.
    pub async fn list_assessments(
        &self,
        course_id: &str,
    ) -> Result<Vec<Assessment>, CrowdmarkError> {
        #[derive(Debug, serde::Deserialize)]
        #[serde(tag = "type", content = "attributes", rename_all_fields = "kebab-case")]
        enum ResponseDataItem {
            #[serde(rename = "assignments")]
            Assignment {
                #[serde(deserialize_with = "from_raw_normalized_points")]
                normalized_points: Option<f32>,
                submitted_at: Option<DateTime<Utc>>,
                due: Option<DateTime<Utc>>,
                marks_sent_at: Option<DateTime<Utc>>,
            },
        }

        #[derive(Debug, serde::Deserialize)]
        #[serde(rename_all = "kebab-case")]
        struct ResponseRelationship {
            exam_master: RequiredData<RelationshipId>,
        }

        #[derive(Debug, serde::Deserialize)]
        enum ExamMasterKind {
            #[serde(rename = "ExamMaster::AtHome")]
            AtHome,
            #[serde(rename = "ExamMaster::Proctored")]
            Proctored,
        }

        #[derive(Debug, serde::Deserialize)]
        struct ExamMasterData {
            title: String,
            #[serde(rename = "type")]
            kind: ExamMasterKind,
        }

        #[derive(Debug, serde::Deserialize)]
        #[serde(tag = "type", content = "attributes", rename_all_fields = "kebab-case")]
        enum IncludedDataItem {
            #[serde(rename = "exam-masters")]
            ExamMaster(ExamMasterData),
        }

        let resp = self
            .client
            .get(format!("https://app.crowdmark.com/api/v2/student/assignments?fields[exam-masters][]=type&fields[exam-masters][]=title&filter[course]={course_id}"))
            .send()
            .await?;

        if resp.status() == reqwest::StatusCode::FOUND {
            return Err(CrowdmarkError::NotAuthenticated(
                "Could not get assessments".to_string(),
            ));
        }

        let text = resp.text().await?;

        let json_val: Value = serde_json::from_str(&text)?;
        if json_val.get("included").is_none() {
            return Err(CrowdmarkError::InvalidCourseID());
        }

        let root: ResponseRoot<ResponseDataItem, ResponseRelationship, IncludedDataItem> =
            serde_json::from_value(json_val)?;

        let exam_masters: HashMap<_, _> = root
            .included
            .into_iter()
            .map(|IncludedData { id, attributes }| match attributes {
                IncludedDataItem::ExamMaster(e) => (id, e),
            })
            .collect();

        let assessments: Result<Vec<Assessment>, CrowdmarkError> = root
            .data
            .into_iter()
            .map(
                |ResponseData {
                     id: _,
                     attributes,
                     relationships,
                 }| {
                    match attributes {
                        ResponseDataItem::Assignment {
                            normalized_points,
                            submitted_at,
                            due,
                            marks_sent_at,
                        } => {
                            let exam_master = exam_masters
                                .get(&relationships.exam_master.data.id)
                                .ok_or_else(|| {
                                    CrowdmarkError::DecodeError(format!(
                                        "Missing exam_master for id {}",
                                        relationships.exam_master.data.id
                                    ))
                                })?;

                            Ok(Assessment {
                                id: relationships.exam_master.data.id,
                                title: exam_master.title.clone(),
                                kind: match exam_master.kind {
                                    ExamMasterKind::AtHome => AssessmentKind::TakeHome,
                                    ExamMasterKind::Proctored => AssessmentKind::Proctored,
                                },
                                due,
                                submitted: submitted_at,
                                graded: marks_sent_at,
                                score: normalized_points,
                            })
                        }
                    }
                },
            )
            .collect();

        assessments
    }
}
