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
    pub async fn start_drafting(&self, assessment_id: &str) -> Result<(), CrowdmarkError> {
        self.client.post(format!("https://app.crowdmark.com/api/v2/student/assignments/{assessment_id}/start-drafting"))
            .header("X-Csrf-Token", self.csrf.clone())
            .send().await?;
        Ok(())
    }

    async fn clear_pages(&self, root: &AssessResponse) -> Result<(), CrowdmarkError> {
        let page_uuids: Vec<String> = root
            .included
            .iter()
            .filter(|item| item.type_ == "assignment-pages")
            .map(|item| item.id.clone())
            .collect();

        for uuid in page_uuids {
            let body = serde_json::json!({
                "data": {
                    "id": uuid,
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
                    "https://app.crowdmark.com/api/v2/student/assignment-pages/{uuid}"
                ))
                .header("Content-Type", "application/vnd.api+json")
                .json(&body)
                .header("X-Csrf-Token", self.csrf.clone())
                .send()
                .await?;
        }
        for q in &root.included {
            if q.type_ == "assignment-questions" {
                let body = serde_json::json!({
                    "data": {
                        "id": q.id,
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
                        q.id
                    ))
                    .header("Content-Type", "application/vnd.api+json")
                    .json(&body)
                    .header("X-Csrf-Token", self.csrf.clone())
                    .send()
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn upload_assessment(
        &self,
        assessment_id: &str,
        pages: Vec<(usize, Vec<u8>)>,
    ) -> Result<(), CrowdmarkError> {
        let resp = self
            .client
            .get(format!("https://app.crowdmark.com/api/v2/student/assignments/{assessment_id}?fields[exam-masters][]=type&fields[exam-masters][]=title"))
            .send()
            .await?;
        let text = resp.text().await?;

        let json_val: serde_json::Value = serde_json::from_str(&text)?;
        if json_val.get("included").is_none() {
            return Err(CrowdmarkError::InvalidAssessmentID());
        }

        let root: AssessResponse = serde_json::from_value(json_val)?;
        let assignment_id = root.data.id.clone();
        self.start_drafting(&assignment_id).await?;
        self.clear_pages(&root).await?;

        for (question, img) in pages {
            let question_id = root.included.iter().find_map(|inc| {
                if inc.type_ == "assignment-questions" && inc.attributes.sequence == Some(question)
                {
                    Some(inc.id.clone())
                } else {
                    None
                }
            });

            let Some(question_id) = question_id else {
                return Err(CrowdmarkError::TooManyPages());
            };

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

            let bucket_url = s3_policy_response["bucket"]
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
                .text("x-amz-meta-original-filename", assignment_id.clone());

            form = form.part(
                "file",
                multipart::Part::bytes(img.clone())
                    .file_name(assignment_id.clone())
                    .mime_str("image/jpeg")?,
            );

            self.client
                .post(bucket_url)
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

        let resp = self
            .client
            .get(format!("https://app.crowdmark.com/api/v2/student/assignments/{assessment_id}?fields[exam-masters][]=type&fields[exam-masters][]=title"))
            .send().await?;
        let text = resp.text().await?;

        let root: AssessResponse = serde_json::from_str(&text)?;

        let mut pages = Vec::new();

        for item in root.included {
            if item.type_ == "assignment-pages" {
                let filename = item.attributes.filename.unwrap_or_default();
                let uuid = item.attributes.uuid.unwrap_or_default();
                let number = item.attributes.number.unwrap_or_default();

                let question_id = item
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

                pages.push(TargetPage {
                    id: item.id,
                    question_id,
                    filename,
                    uuid,
                    number,
                });
            }
        }

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
            .ok_or(CrowdmarkError::S3PolicyError())?;

        let output = TargetOutput {
            pages,
            signature: signature.to_string(),
        };

        self.client
            .put(format!(
                "https://app.crowdmark.com/api/v2/student/assignments/{}",
                &root.data.id
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
