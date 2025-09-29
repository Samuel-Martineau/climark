use std::collections::HashMap;

use chrono::{DateTime, Utc};
use reqwest::Error;
use serde::Deserialize;

static DEFAULT_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

#[derive(Debug)]
pub struct Client {
    client: reqwest::Client,
}

#[non_exhaustive]
#[derive(Debug, serde::Serialize)]
pub struct Course {
    pub id: String,
    pub name: String,
    pub archived: bool,
    pub assessment_count: usize,
}

#[non_exhaustive]
#[derive(Debug, serde::Serialize)]
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
#[derive(Debug, serde::Serialize)]
pub enum AssessmentKind {
    TakeHome,
    Proctored,
}

#[derive(Debug, serde::Deserialize)]
struct ResponseRoot<DA, DR, I> {
    data: Vec<ResponseData<DA, DR>>,
    included: Vec<IncludedData<I>>,
}

#[derive(Debug, serde::Deserialize)]
struct ResponseData<A, R> {
    id: String,

    #[serde(flatten)]
    attributes: A,
    relationships: R,
}

#[derive(Debug, serde::Deserialize)]
struct IncludedData<A> {
    id: String,

    #[serde(flatten)]
    attributes: A,
}

#[derive(Debug, serde::Deserialize)]
struct RelationshipId {
    id: String,
}

#[derive(Debug, serde::Deserialize)]
struct OptionalData<T> {
    data: Option<T>,
}

#[derive(Debug, serde::Deserialize)]
struct RequiredData<T> {
    data: T,
}

#[derive(Debug, serde::Deserialize)]
struct EmptyStruct {}

fn from_raw_normalized_points<'de, D>(deserializer: D) -> Result<Option<f32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(serde::Deserialize)]
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
    pub fn new(session_token: &str) -> Self {
        let mut headers = reqwest::header::HeaderMap::new();

        headers.insert(reqwest::header::COOKIE, {
            let mut auth_value =
                reqwest::header::HeaderValue::from_str(&format!("cm_session_id={session_token}"))
                    .unwrap();
            auth_value.set_sensitive(true);
            auth_value
        });

        let client = reqwest::Client::builder()
            .user_agent(DEFAULT_USER_AGENT)
            .default_headers(headers)
            .build()
            .unwrap();

        Self { client }
    }

    pub async fn list_courses(&self) -> Result<Vec<Course>, Error> {
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
            .await?
            .json::<ResponseRoot<ResponseDataItem, ResponseRelationship, EmptyStruct>>()
            .await?;

        let courses: Vec<_> = resp
            .data
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
                        id: id,
                        name: name,
                        assessment_count: exam_master_count,
                        archived: relationships.course_archivation.data.is_some(),
                    },
                },
            )
            .collect();

        Ok(courses)
    }

    pub async fn list_assessments(&self, course_id: &str) -> Result<Vec<Assessment>, Error> {
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
            .await?
            .json::<ResponseRoot<ResponseDataItem, ResponseRelationship, IncludedDataItem>>()
            .await?;

        let exam_masters: HashMap<_, _> = resp
            .included
            .into_iter()
            .map(|IncludedData { id, attributes }| match attributes {
                IncludedDataItem::ExamMaster(e) => (id, e),
            })
            .collect();

        let assessments: Vec<_> = resp
            .data
            .into_iter()
            .map(
                |ResponseData {
                     id,
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
                                .unwrap();
                            Assessment {
                                id,
                                title: exam_master.title.clone(),
                                kind: match exam_master.kind {
                                    ExamMasterKind::AtHome => AssessmentKind::TakeHome,
                                    ExamMasterKind::Proctored => AssessmentKind::Proctored,
                                },
                                due: due,
                                submitted: submitted_at,
                                graded: marks_sent_at,
                                score: normalized_points,
                            }
                        }
                    }
                },
            )
            .collect();

        Ok(assessments)
    }
}
