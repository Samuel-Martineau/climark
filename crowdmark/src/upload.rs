use crate::error::CrowdmarkError;
use reqwest::multipart;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct AssessResponse {
    data: ResponseData,
    included: Vec<IncludedItem>,
}

#[derive(Debug, Deserialize)]
struct ResponseData {
    id: String,
}

#[derive(Debug, Deserialize)]
struct IncludedItem {
    id: String,
    #[serde(rename = "type")]
    type_: String,
    #[serde(default)]
    attributes: IncludedAttributes,
    #[serde(default)]
    relationships: Option<IncludedRelationships>,
}

#[derive(Debug, Deserialize, Default)]
struct IncludedAttributes {
    sequence: Option<usize>,
    filename: Option<String>,
    uuid: Option<String>,
    number: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct IncludedRelationships {
    question: Option<SingleRelationship>,
}

#[derive(Debug, Deserialize)]
struct SingleRelationship {
    data: Option<RelationData>,
}

#[derive(Debug, Deserialize)]
struct RelationData {
    id: Value,
}

impl crate::Client {
    /// Starts drafting  an assessment.
    ///
    /// # Errors
    ///
    /// Returns `CrowdmarkError` if the request to Crowdmark fails.
    pub async fn start_drafting(&self, assessment_id: &str) -> Result<(), CrowdmarkError> {
        self.client.post(format!("https://app.crowdmark.com/api/v2/student/assignments/{assessment_id}/start-drafting"))
            .header("X-Csrf-Token", self.csrf.clone())
            .send().await?;
        Ok(())
    }

    async fn clear_pages(&self, root: &AssessResponse) -> Result<(), CrowdmarkError> {
        for page in root
            .included
            .iter()
            .filter(|i| i.type_ == "assignment-pages")
        {
            let body = serde_json::json!({
                "data": {
                    "id": page.id,
                    "attributes": {
                        "state": "pending_delete",
                    },
                    "relationships": {
                        "question": {
                            "data": {
                                "type": "assignment-questions",
                                "id": "",
                            }
                        }
                    },
                    "type": "assignment-pages"
                }
            });
            self.client
                .patch(format!(
                    "https://app.crowdmark.com/api/v2/student/assignment-pages/{}",
                    page.id
                ))
                .header("Content-Type", "application/vnd.api+json")
                .json(&body)
                .header("X-Csrf-Token", self.csrf.clone())
                .send()
                .await?;
        }
        for question in root
            .included
            .iter()
            .filter(|i| i.type_ == "assignment-questions")
        {
            let body = serde_json::json!({
                "data": {
                    "id": question.id,
                    "relationships": {
                        "anchored-to-exam-page": {
                            "data": Value::Null,
                        },
                        "assignment": {
                            "data": {
                                "id": root.data.id,
                                "type": "assignments",
                            }
                            }
                    },
                    "type": "assignment-questions"
                }
            });
            self.client
                .patch(format!(
                    "https://app.crowdmark.com/api/v2/student/assignment-questions/{}",
                    question.id
                ))
                .header("Content-Type", "application/vnd.api+json")
                .json(&body)
                .header("X-Csrf-Token", self.csrf.clone())
                .send()
                .await?;
        }
        Ok(())
    }

    async fn fetch_assessment(
        &self,
        assessment_id: &str,
    ) -> Result<AssessResponse, CrowdmarkError> {
        let resp = self
            .client
            .get(format!(
                "https://app.crowdmark.com/api/v2/student/assignments/{assessment_id}?fields[exam-masters][]=type&fields[exam-masters][]=title",
            ))
            .send()
            .await?;
        let text = resp.text().await?;
        let json_val: Value = serde_json::from_str(&text)?;
        if json_val.get("included").is_none() {
            return Err(CrowdmarkError::InvalidAssessmentID());
        }
        Ok(serde_json::from_value(json_val)?)
    }

    /// Uploads pages for an assessment.
    ///
    /// # Errors
    ///
    /// Returns `CrowdmarkError` if:
    /// - The assessment ID is invalid.
    /// - There are too many pages or a page is missing.
    /// - Requests to S3 or Crowdmark fail.
    pub async fn upload_assessment(
        &self,
        assessment_id: &str,
        pages: Vec<(usize, Vec<u8>)>,
    ) -> Result<(), CrowdmarkError> {
        let root = self.fetch_assessment(assessment_id).await?;
        let assignment_id = root.data.id.clone();
        self.start_drafting(&assignment_id).await?;
        self.clear_pages(&root).await?;

        for (question, img) in pages {
            let question_id = root
                .included
                .iter()
                .find(|i| {
                    i.type_ == "assignment-questions" && i.attributes.sequence == Some(question)
                })
                .map(|i| i.id.clone())
                .ok_or(CrowdmarkError::TooManyPages())?;

            let uuid = Uuid::new_v4().to_string();

            let s3_policy_response = self
                .client
                .post("https://app.crowdmark.com/api/v1/s3_policies")
                .form(&[
                    ("enrollment_uuid", assignment_id.as_str()),
                    ("requested_uuid", uuid.as_str()),
                    ("original_filename", assignment_id.as_str()),
                    ("content_type", "image/jpeg"),
                ])
                .send()
                .await?
                .json::<serde_json::Value>()
                .await?;

            let bucket = s3_policy_response["bucket"]
                .as_str()
                .ok_or(CrowdmarkError::S3PolicyError())?;
            let key = s3_policy_response["key"]
                .as_str()
                .ok_or(CrowdmarkError::S3PolicyError())?;
            let fields = s3_policy_response["fields"]
                .as_array()
                .ok_or(CrowdmarkError::S3PolicyError())?;

            let mut form = multipart::Form::new();

            for field in fields {
                let name = field[0].as_str().ok_or(CrowdmarkError::S3PolicyError())?;
                let value = field[1].as_str().ok_or(CrowdmarkError::S3PolicyError())?;
                form = form.text(name.to_string(), value.to_string());
            }

            form = form
                .text("key", key.to_string())
                .text("Content-Type", "image/jpeg")
                .text("x-amz-meta-original-filename", assignment_id.clone())
                .part(
                    "file",
                    multipart::Part::bytes(img.clone())
                        .file_name(assignment_id.clone())
                        .mime_str("image/jpeg")?,
                );

            self.client
                .post(bucket)
                .multipart(form)
                .send()
                .await?
                .error_for_status()
                .map_err(|msg| CrowdmarkError::S3UploadError(msg.to_string()))?;

            let body = serde_json::json!({
                "data": {
                    "type": "assignment-pages",
                    "attributes": {
                        "number": question,
                        "filename": assignment_id.as_str(),
                        "uuid": uuid,
                        "is-anchor": true,
                    },
                    "relationships": {
                        "question": {
                            "data": {
                                "type": "assignment-questions",
                                "id": question_id
                            }
                        }
                    }
                }
            });

            self.client
                .post("https://app.crowdmark.com/api/v2/student/assignment-pages")
                .header("Content-Type", "application/vnd.api+json")
                .header("X-Csrf-Token", self.csrf.clone())
                .json(&body)
                .send()
                .await?
                .error_for_status()
                .map_err(|msg| CrowdmarkError::AssignmentUploadError(msg.to_string()))?;
        }
        Ok(())
    }

    /// Submits an assessment.
    ///
    /// # Errors
    ///
    /// Returns `CrowdmarkError` if:
    /// - The assessment cannot be fetched.
    /// - Generating the submission payload fails.
    /// - The submission request fails.
    pub async fn submit_assessment(&self, assessment_id: &str) -> Result<(), CrowdmarkError> {
        #[derive(Debug, Serialize)]
        struct TargetOutput {
            pages: Vec<TargetPage>,
            signature: String,
        }

        #[derive(Debug, Serialize)]
        struct TargetPage {
            id: String,
            question_id: String,
            filename: String,
            uuid: String,
            number: i64,
        }

        let root = self.fetch_assessment(assessment_id).await?;

        let pages: Vec<_> = root
            .included
            .into_iter()
            .filter(|i| i.type_ == "assignment-pages")
            .map(|i| {
                let question_id = i
                    .relationships
                    .as_ref()
                    .and_then(|r| r.question.as_ref())
                    .and_then(|q| q.data.as_ref())
                    .map(|d| match &d.id {
                        Value::String(s) => s.clone(),
                        Value::Number(n) => n.to_string(),
                        _ => String::new(),
                    })
                    .unwrap_or_default();

                TargetPage {
                    id: i.id,
                    question_id,
                    filename: i.attributes.filename.unwrap_or_default(),
                    uuid: i.attributes.uuid.unwrap_or_default(),
                    number: i.attributes.number.unwrap_or_default(),
                }
            })
            .collect();

        let s3_policy_response = self
            .client
            .post("https://app.crowdmark.com/api/v1/s3_policies")
            .form(&[("enrollment_uuid", root.data.id.clone())])
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        let signature = s3_policy_response["upload_signature"]
            .as_str()
            .ok_or(CrowdmarkError::S3PolicyError())?
            .to_string();

        let output = TargetOutput { pages, signature };

        self.client
            .put(format!(
                "https://app.crowdmark.com/api/v2/student/assignments/{}",
                root.data.id
            ))
            .json(&output)
            .header("X-Csrf-Token", self.csrf.clone())
            .send()
            .await?
            .error_for_status()
            .map_err(|msg| CrowdmarkError::AssignmentSubmitError(msg.to_string()))?;

        Ok(())
    }
}
