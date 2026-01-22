use clap::Args;

#[derive(Args, Clone, Debug)]
pub struct RequestData {
    pub url: String,

    #[clap(
        short = 'H',
        long = "header",
        value_parser = RequestData::parse_header,
        value_name = "KEY:VALUE",
        num_args = 1..,
        required = false
    )]
    pub headers: Vec<(String, String)>,

    #[clap(short, long, default_value = "", required = false)]
    pub body: String,
}

impl RequestData {
    pub fn parse_header(s: &str) -> Result<(String, String), String> {
        let parts: Vec<&str> = s.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid header format: '{}'. Use KEY:VALUE", s));
        }
        Ok((parts[0].trim().to_string(), parts[1].trim().to_string()))
    }
}
