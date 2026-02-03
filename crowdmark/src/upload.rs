use crate::error::CrowdmarkError;
use reqwest::multipart;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize)]
struct AssessResponse {
    data: ResponseData,
    included: Vec<IncludedItem>,
}

#[derive(Clone, Debug, Deserialize)]
struct ResponseData {
    id: String,
}

#[derive(Clone, Debug, Deserialize)]
struct IncludedItem {
    #[serde(default)]
    attributes: IncludedAttributes,
    id: String,
    #[serde(default)]
    relationships: Option<IncludedRelationships>,
    #[serde(rename = "type")]
    type_: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct IncludedAttributes {
    filename: Option<String>,
    number: Option<i64>,
    sequence: Option<usize>,
    uuid: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct IncludedRelationships {
    question: Option<SingleRelationship>,
}

#[derive(Clone, Debug, Deserialize)]
struct SingleRelationship {
    data: Option<RelationData>,
}

#[derive(Clone, Debug, Deserialize)]
struct RelationData {
    id: Value,
}

impl crate::Client {
    async fn clear_pages(
        &self,
        csrf: &str,
        shared_root: Arc<AssessResponse>,
    ) -> Result<(), CrowdmarkError> {
        let mut set = tokio::task::JoinSet::new();

        let csrf_token = csrf.to_owned();

        for item in shared_root.included.iter() {
            let client = self.client.clone();
            let cloned_item = item.clone();
            let token = csrf_token.clone();
            let root_ref = Arc::<AssessResponse>::clone(&shared_root);

            set.spawn(async move {
            if cloned_item.type_ == "assignment-pages" {
                let body = serde_json::json!({
                    "data": {
                        "id": cloned_item.id,
                        "type": "assignment-pages",
                        "attributes": { "state": "pending_delete" },
                        "relationships": {
                            "question": { "data": { "type": "assignment-questions", "id": "" } }
                        }
                    }
                });

                client.patch(format!("https://app.crowdmark.com/api/v2/student/assignment-pages/{}", cloned_item.id))
                    .header("Content-Type", "application/vnd.api+json")
                    .header("X-Csrf-Token", token)
                    .json(&body)
                    .send()
                    .await?
                    .error_for_status()?;
            }
            else if cloned_item.type_ == "assignment-questions" {
                let body = serde_json::json!({
                    "data": {
                        "id": cloned_item.id,
                        "type": "assignment-questions",
                        "relationships": {
                            "anchored-to-exam-page": { "data": serde_json::Value::Null },
                            "assignment": { "data": { "id": root_ref.data.id, "type": "assignments" } }
                        }
                    }
                });

                client.patch(format!("https://app.crowdmark.com/api/v2/student/assignment-questions/{}", cloned_item.id))
                    .header("Content-Type", "application/vnd.api+json")
                    .header("X-Csrf-Token", token)
                    .json(&body)
                    .send()
                    .await?
                    .error_for_status()?;
            }
            // Explicitly return Ok from the task
            Ok::<(), CrowdmarkError>(())
        });
        }

        // IMPORTANT: You must actually wait for the tasks to finish
        while let Some(res) = set.join_next().await {
            // res is a Result<Result<(), Error>, JoinError>
            res.map_err(|e| CrowdmarkError::AssessmentUpload(e.to_string()))??;
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
        Ok(serde_json::from_str::<AssessResponse>(&text)?)
    }

    /// Starts drafting an assessment.
    ///
    /// # Errors
    ///
    /// Returns [`CrowdmarkError`] if the request to Crowdmark fails.
    #[inline]
    pub async fn start_drafting(
        &self,
        csrf: &str,
        assessment_id: &str,
    ) -> Result<(), CrowdmarkError> {
        self.client.post(format!("https://app.crowdmark.com/api/v2/student/assignments/{assessment_id}/start-drafting"))
            .header("X-Csrf-Token", csrf)
            .send().await?;
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
    #[inline]
    pub async fn submit_assessment(
        &self,
        csrf: &str,
        assessment_id: &str,
    ) -> Result<(), CrowdmarkError> {
        #[derive(Debug, Serialize)]
        struct TargetOutput {
            pages: Vec<TargetPage>,
            signature: String,
        }

        #[derive(Debug, Serialize)]
        struct TargetPage {
            filename: String,
            id: String,
            number: i64,
            question_id: String,
            uuid: String,
        }

        #[derive(Debug, Deserialize)]
        struct S3Response {
            upload_signature: String,
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
                    .map(|d| match d.id.clone() {
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
            .json::<S3Response>()
            .await?;

        let signature = s3_policy_response.upload_signature;

        let output = TargetOutput { pages, signature };

        self.client
            .put(format!(
                "https://app.crowdmark.com/api/v2/student/assignments/{}",
                root.data.id
            ))
            .json(&output)
            .header("X-Csrf-Token", csrf)
            .send()
            .await?
            .error_for_status()
            .map_err(|msg| CrowdmarkError::AssessmentSubmit(msg.to_string()))?;

        Ok(())
    }

    /// Uploads pages for an assessment.
    ///
    /// # Errors
    ///
    /// Returns `CrowdmarkError` if:
    /// - The assessment ID is invalid.
    /// - There are too many pages or a page is missing.
    /// - Requests to S3 or Crowdmark fail.
    #[inline]
    pub async fn upload_assessment<I>(
        &self,
        csrf: &str,
        assessment_id: &str,
        pages: I,
    ) -> Result<(), CrowdmarkError>
    where
        I: IntoIterator<Item = (usize, Vec<u8>)>,
    {
        let root = self.fetch_assessment(assessment_id).await?;
        let assignment_id = root.data.id.clone();
        let shared_root = Arc::new(root);
        self.start_drafting(csrf, &assignment_id).await?;
        self.clear_pages(csrf, Arc::<AssessResponse>::clone(&shared_root))
            .await?;

        let mut set = tokio::task::JoinSet::new();

        for (question, img) in pages {
            let client = self.client.clone();

            let cloned_assignment_id = assignment_id.clone();
            let cloned_root = Arc::<AssessResponse>::clone(&shared_root);
            let cloned_csrf = csrf.to_owned();
            set.spawn(async move {
                upload_page(
                    client,
                    cloned_root,
                    cloned_csrf,
                    &cloned_assignment_id,
                    question + 1,
                    img,
                )
                .await
            });
        }

        while let Some(result) = set.join_next().await {
            result??;
        }

        Ok(())
    }
}

async fn upload_page(
    client: reqwest::Client,
    root: Arc<AssessResponse>,
    csrf: String,
    assignment_id: &str,
    question: usize,
    img: Vec<u8>,
) -> Result<(), CrowdmarkError> {
    #[derive(Deserialize)]
    struct S3Response {
        bucket: String,
        fields: Vec<(String, String)>,
        key: String,
    }

    let question_id = root
        .included
        .iter()
        .find(|i| i.type_ == "assignment-questions" && i.attributes.sequence == Some(question))
        .map(|i| i.id.clone())
        .ok_or(CrowdmarkError::TooManyPages())?;

    let uuid = Uuid::new_v4().to_string();

    let s3_policy = client
        .post("https://app.crowdmark.com/api/v1/s3_policies")
        .form(&[
            ("enrollment_uuid", assignment_id),
            ("requested_uuid", uuid.as_str()),
            ("original_filename", assignment_id),
            ("content_type", "image/jpeg"),
        ])
        .send()
        .await?
        .json::<S3Response>()
        .await?;

    let mut form = multipart::Form::new();

    for (name, value) in s3_policy.fields {
        form = form.text(name, value);
    }

    form = form
        .text("key", s3_policy.key)
        .text("Content-Type", "image/jpeg")
        .text("x-amz-meta-original-filename", assignment_id.to_owned())
        .part(
            "file",
            multipart::Part::stream(img).file_name(assignment_id.to_owned()),
        );

    client
        .post(s3_policy.bucket)
        .multipart(form)
        .send()
        .await?
        .error_for_status()
        .map_err(|msg| CrowdmarkError::S3Upload(msg.to_string()))?;

    let body = serde_json::json!({
        "data": {
            "type": "assignment-pages",
            "attributes": {
                "number": question,
                "filename": assignment_id,
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

    client
        .post("https://app.crowdmark.com/api/v2/student/assignment-pages")
        .header("Content-Type", "application/vnd.api+json")
        .header("X-Csrf-Token", csrf)
        .json(&body)
        .send()
        .await?
        .error_for_status()
        .map_err(|msg| CrowdmarkError::AssessmentUpload(msg.to_string()))?;
    Ok(())
}
