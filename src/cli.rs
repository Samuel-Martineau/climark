#[derive(clap::Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    #[arg(long, env)]
    pub crowdmark_session_token: Option<String>,
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
    #[command(about = "List courses")]
    ListCourses {
        #[arg(short, long, value_enum, default_value_t)]
        format: OutputFormat,
        #[arg(help = "Don't print error messages", short, long)]
        silent: bool,
    },
    #[command(about = "List assessments")]
    ListAssessments {
        #[arg(env = "CLIMARK_DEFAULT_COURSE")]
        course_id: String,
        #[arg(help = "Hide scores", short, long)]
        hide_scores: bool,
        #[arg(help = "Print in JSON Format", short, long)]
        json: bool,
        #[arg(help = "Don't print error messages", short, long)]
        silent: bool,
    },
    #[command(about = "Upload assessment")]
    UploadAssessment {
        assessment_id: String,
        #[arg(help = "Don't print error messages", long)]
        silent: bool,
        #[arg(help = "Submit assignment after upload", short, long)]
        submit: bool,
    },
    #[command(about = "Login to Crowdmark")]
    Login,
}
