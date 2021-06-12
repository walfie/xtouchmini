use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Opt {
    #[structopt(long, env, default_value = "4444")]
    pub obs_port: u16,
    #[structopt(long, env, default_value = "localhost")]
    pub obs_host: String,
    #[structopt(long, env, hide_env_values = true)]
    pub obs_password: Option<String>,
}
