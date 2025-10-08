#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    #[arg(long, env)]
    pub crowdmark_session_token: String,
}

#[derive(clap::ValueEnum, Clone, Default)]
pub enum OutputFormat {
    #[default]
    Pretty,
    Plain,
    Json,
}

#[derive(clap::Subcommand)]
pub enum Commands {
    ListCourses {
        #[arg(short, long, value_enum, default_value_t)]
        // json: bool,
        format: OutputFormat,
    },
    ListAssessments {
        #[arg(env = "CLIMARK_DEFAULT_COURSE")]
        course_id: String,
        #[arg(short, long)]
        json: bool,
    },
    UploadAssessment {
        #[arg(env = "CLIMARK_DEFAULT_COURSE")]
        course_id: String,
        assessment_id: String,
    },
}
