use region::Region;

pub fn get_regions() -> Vec<Region> {
    // TODO: Load regions from S3.
    // TODO: Load last since_id from DB.
    let mut regions: Vec<Region> = Vec::new();

    let mut region = Region::new(1, String::from("region 1"));
    region.since_id(1).tick(1);
    regions.push(region);

    let mut region = Region::new(2, String::from("region 2"));
    region.since_id(2).tick(2);
    regions.push(region);

    regions
}


