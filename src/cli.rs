#[derive(clap::Parser)]
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
        // json: bool,
        #[arg(short, long)]
        silent: bool,
        #[arg(short, long, value_enum, default_value_t)]
        format: OutputFormat,
    },
    ListAssessments {
        #[arg(env = "CLIMARK_DEFAULT_COURSE")]
        course_id: String,
        #[arg(short, long)]
        json: bool,
        #[arg(short, long)]
        silent: bool,
    },
    UploadAssessment {
        assessment_id: String,
        #[arg(short, long)]
        submit: bool,
    },
}
