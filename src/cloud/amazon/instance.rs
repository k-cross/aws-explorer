pub mod collector {
    use aws_config::meta::region::RegionProviderChain;
    use aws_sdk_ec2::model::{Instance, Tag};
    use aws_sdk_ec2::{Client, Error, Region};
    use crate::cloud::definition::AmazonCollection;

    async fn match_instances(client: &Client) -> Result<(AmazonCollection, AmazonCollection), Error> {
        let resp = client.describe_instances().send().await?;

        let mut running_insts = Vec::new();
        let mut offline_insts = Vec::new();

        for reservation in resp.reservations().unwrap_or_default() {
            for instance in reservation.instances().unwrap_or_default() {
                match instance.state.clone().unwrap().name {
                    Some(aws_sdk_ec2::model::InstanceStateName::Running) => {
                        running_insts.push(instance.to_owned())
                    }
                    _ => offline_insts.push(instance.to_owned()),
                }
            }
        }

        let r = AmazonCollection::AmazonInstances(running_insts);
        let o = AmazonCollection::AmazonInstances(offline_insts);
        Ok((r, o))
    }

    pub async fn runner(region: &str) -> Result<(AmazonCollection, AmazonCollection), Box<dyn std::error::Error>> {
        let region_provider = RegionProviderChain::first_try(Region::new(region.to_owned()))
            .or_default_provider()
            .or_else(Region::new("us-west-2"));
        let shared_config = aws_config::from_env().region(region_provider).load().await;
        let client = Client::new(&shared_config);

        match match_instances(&client).await {
            Ok(res) => Ok(res),
            Err(e) => Err(e.into()),
        }
    }
}
