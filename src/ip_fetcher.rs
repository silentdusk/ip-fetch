#[derive(serde::Deserialize)]
pub struct IpDetails {
    pub r#as: String,
    pub city: String,
    pub country: String,
    #[serde(rename = "countryCode")]
    pub country_code: String,
    pub isp: String,
    pub lat: f64,
    pub lon: f64,
    pub org: String,
    pub query: String,
    pub region: String,
    #[serde(rename = "regionName")]
    pub region_name: String,
    pub status: String,
    pub timezone: String,
    pub zip: String,
}

#[derive(PartialEq)]
pub enum FetchState {
    Pending,
    Success,
    Failure,
}

pub struct IpFetcher {
    pub details: Option<IpDetails>,
}

impl IpFetcher {
    pub fn fetch(target: &String) -> Result<Self, Box<dyn std::error::Error>> {
        let url = format!("http://ip-api.com/json/{}", target);
        let details = reqwest::blocking::get(url)?.json::<IpDetails>()?;
        Ok(Self {
            details: Some(details),
        })
    }
}
