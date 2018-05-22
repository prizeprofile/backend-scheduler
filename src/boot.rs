use std::env;
use region::TweetRegion;
use rusoto_core::Region;
use serde_json::{Value, from_str, to_string};
use futures::{Stream, Future, IntoFuture};
use rusoto_s3::{S3, S3Client, GetObjectRequest};

pub fn get_regions() -> Result<Vec<TweetRegion>, &'static str> {
    let s3 = S3Client::simple(Region::EuWest1);

    let mut req: GetObjectRequest = Default::default();
    {
        req.bucket = env::var("S3_REGIONS_BUCKET").expect("Missing env var");
        req.key = env::var("S3_REGIONS_KEY").expect("Missing env var");
    }
    
    // TODO: Put this into one future.
    let json: String = s3.get_object(&req)
        .sync().expect("Regions S3 conf error")
        .body.expect("Regions S3 conf empty")
        .collect()
        .then(move |stream| stream.into_future())
        .wait().expect("Couldn't read bucket")
        .iter()
        .map(|line| String::from_utf8(line.to_vec()).expect("Invalid string"))
        .fold(String::new(), |acc, line| acc + &line);

    let json: Value = from_str(&json).expect("Invalid JSON");
    
    let regions: Vec<TweetRegion> = parse_json_to_regions(json)
        .expect("Json couldn't be parsed");
        // TODO: Load last since_id from DB.
    
    Ok(regions)
}

fn parse_json_to_regions(json: Value) -> Result<Vec<TweetRegion>, &'static str> {
	let regions = json["regions"].as_array().expect("Property regions isn't an array");
    // Minimum delay between scheduled output events in seconds per window (15 minutes).
    // Prevents overflooding the Twitter API with requests.
    let delay_between_calls: u64 = 900 / json["api_limits"]["search"].as_u64().unwrap_or(450);
    // Sum of all flex integers of all regions. The higher flex value is, the less frequently
    // API calls are made for that regions.
    let flex_sum: u64 = regions.iter().fold(0, |acc, region| acc + region["flex"].as_u64().unwrap_or(1));
    let base_tick: u64 = 1000 * delay_between_calls * flex_sum / regions.len() as u64;

    let regions: Vec<TweetRegion> = regions.iter()
        // TODO: Inform admin about region misconfiguration.
        .filter_map(|item| {
            let params: String = to_string(item["params"].as_object()?)
				.expect("Couldn't stringify params.");

            let mut region = TweetRegion::new(item["id"].as_u64()?, params);
            region.tick(item["flex"].as_u64()? * base_tick);
			region.since_id(1);
            Some(region)
        })
        .collect::<Vec<TweetRegion>>();

    Ok(regions)
}

