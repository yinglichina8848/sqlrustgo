use clap::Parser;

#[derive(Parser, Debug)]
pub struct TpchArgs {
    #[arg(long, default_value = "1")]
    pub scale: f64,
    #[arg(long, default_value = "3")]
    pub iterations: u32,
    #[arg(long, default_value = "Q1,Q3,Q6,Q10")]
    pub queries: String,
    #[arg(long)]
    pub output: Option<String>,
}

#[derive(Parser, Debug)]
pub struct OltpArgs {
    #[arg(long, default_value = "1")]
    pub threads: u32,
    #[arg(long, default_value = "60")]
    pub duration: u64,
    #[arg(long, default_value = "read")]
    pub workload: String,
    #[arg(long)]
    pub output: Option<String>,
}

#[derive(Parser, Debug)]
pub struct CustomArgs {
    #[arg(long)]
    pub file: String,
    #[arg(long, default_value = "1")]
    pub iterations: u32,
    #[arg(long, default_value = "1")]
    pub parallel: u32,
    #[arg(long)]
    pub output: Option<String>,
}
