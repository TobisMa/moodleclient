use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Arguments {
    #[arg(long("login-url"))]
    pub login_url: Option<String>,

    #[arg(long("moodle-url"))]
    pub moodle_url: Option<String>,

    #[arg(short, long)]
    pub username: Option<String>,

    #[arg(short, long)]
    pub password: Option<String>,

    #[arg(long("home-dir"))]
    pub home_dir: Option<String>
}
