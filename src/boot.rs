use std::env;
use region::ResourceRegion;
use rusoto_core::Region;
use db::get_latest_id_for_region;
use futures::{Stream, Future, IntoFuture};
use serde_json::{Value, from_str, to_string};
use rusoto_s3::{S3, S3Client, GetObjectRequest};

pub fn get_regions() -> Result<Vec<ResourceRegion>, &'static str> {
    let s3 = S3Client::simple(Region::EuWest1);

    let mut req: GetObjectRequest = Default::default();
    {
        req.bucket = env::var("S3_REGIONS_BUCKET").expect("Config is missing some fields [S3_REGIONS_BUCKET]");
        req.key = env::var("S3_REGIONS_KEY").expect("Config is missing some fields [S3_REGIONS_KEY]");
    }
    
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
    
    let regions: Vec<ResourceRegion> = parse_json_to_regions(json)?;
    
    Ok(regions)
}

fn parse_json_to_regions(json: Value) -> Result<Vec<ResourceRegion>, &'static str> {
	let regions = json["regions"].as_array().expect("Property regions isn't an array");
    // Minimum delay between scheduled output events in seconds per window (15 minutes).
    // Prevents overflooding the Twitter API with requests.
    let delay_between_calls: u64 = 900 / json["api_limits"]["search"].as_u64().unwrap_or(450);
    // Sum of all flex integers of all regions. The higher flex value is, the less frequently
    // API calls are made for that regions.
    let flex_sum: u64 = regions.iter().fold(0, |acc, region| acc + region["flex"].as_u64().unwrap_or(1));
    let base_tick: u64 = 1000 * delay_between_calls * flex_sum / regions.len() as u64;

    let regions: Vec<ResourceRegion> = regions.iter()
        .filter_map(|item| {
            let params: String = to_string(item["params"].as_object()?)
				.expect("Couldn't stringify params.");
            let region_id: u64 = item["id"].as_u64()?;
            let since_id: u64 = get_latest_id_for_region(region_id).unwrap_or(0);
            let topic: String = item["sns_topic"].as_str()?.to_string();
            let compare_since_id: bool = item["compare_since_id"].as_bool()?;

            let mut region = ResourceRegion::new(region_id, topic, params);
            {
                region.tick(item["flex"].as_u64()? * base_tick);
			    region.since_id(since_id);
			    region.compare_since_id(compare_since_id);
            }

            Some(region)
        })
        .collect::<Vec<ResourceRegion>>();

    Ok(regions)
}
