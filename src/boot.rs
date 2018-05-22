use std::env;
use std::convert::From;
use region::TweetRegion;
use rusoto_core::Region;
use futures::stream::Collect;
use serde_json::{Value, Error, from_str, to_string};
use futures::{Stream, Future, IntoFuture};
use rusoto_s3::{S3, S3Client, GetObjectRequest, StreamingBody};

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
	let regions = json["regions"].as_array().expect("Property regions isn't an array").iter();
    /// Minimum delay between scheduled output events in seconds per window (15 minutes).
    /// Prevents overflooding the Twitter API with requests.
    let delay_between_calls: u64 = 900 / json["api_limits"]["search"].as_u64().unwrap_or(450);
    /// Sum of all flex integers of all regions. The higher flex value is, the less frequently
    /// API calls are made for that regions.
    /// 
    /// `Example`
    /// let delay_between_calls = 5 sec
    /// let uk = Region.flex(1)
    /// let us = Region.flex(2)
    /// let flex_sum = uk.flex + us.flex = 3
    /// us.mix_delay = delay_between_calls * flex_sum * us.flex / regions.len = 5*3*2/2
    /// That means that the fastest US region can make API calls is once every 15 seconds.
    /// n uk ukus n uk ukus n uk ukus n uk ukus
    let flex_sum: u64 = 0;
    // regions.fold(0, |acc, region| acc + region["flex"].as_u64().unwrap_or(1));

    println!("{}", base_tick);

    let regions: Vec<TweetRegion> = regions
        .filter_map(|item| {
            let params: String = to_string(item["params"].as_object()?)
				.expect("Couldn't stringify params.");

            let mut region = TweetRegion::new(item["id"].as_u64()?, params);
            region.tick(item["flex"].as_u64()?);
            flex_sum += item["flex"].as_u64?;
            Some(region)
        })
        .map(|region| region.tick(deplay_between_calls * flex_sum * region.tick))
        .collect::<Vec<TweetRegion>>();

    Ok(regions)
}

