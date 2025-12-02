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
        #[arg(short, long, value_enum, default_value_t)]
        format: OutputFormat,
        #[arg(help = "Don't print error messages", short, long)]
        silent: bool,
    },
    #[command(about = "Upload assessment")]
    UploadAssessment {
        #[arg(num_args = 1..=2)]
        ids: Vec<String>,
        #[arg(help = "Don't print error messages", long)]
        silent: bool,
        #[arg(help = "Don't submit assessment after upload", short, long)]
        nosubmit: bool,
    },
    #[command(about = "Login to Crowdmark")]
    Login,
}
