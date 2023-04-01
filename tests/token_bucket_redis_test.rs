// WARN: cargo test --all -- --test-threads 1
async fn initialize_redis() -> Result<(), ()> {
    let redis_address: &str = "redis://127.0.0.1:6379/";
    let client = redis::Client::open(redis_address).map_err(|err| {
        eprintln!("Error: could not open the connection to the Redis({redis_address}): {err}")
    })?;

    let mut conn = client.get_connection().map_err(|err| {
        eprintln!("Error: client could not get the connection to the Redis: {err}")
    })?;

    let _: String = redis::cmd("FLUSHALL")
        .query(&mut conn)
        .map_err(|err| eprintln!("Error: could not flush all data in Redis: {err}"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rrr::rate_limiter_redis;
    use std::time::Duration;

    const CONN: &str = "redis://127.0.0.1:6379/";

    /// Tests the requests exceed the rate limit.
    #[tokio::test]
    async fn token_bucket_redis_case1() -> Result<(), ()> {
        // prev
        initialize_redis().await?;

        // arrange
        let limit_count = 1;
        let size = Duration::from_secs(1);
        let mut client = rate_limiter_redis::RateLimiterRedis::open(CONN, limit_count).await?;
        let key_prefix = "test5";
        let resource = "data";
        let subject = "andy";

        // act
        let actual = client
            .record_token_bucket(key_prefix, resource, subject, size)
            .await?;
        assert!(actual);

        let actual = client
            .record_token_bucket(key_prefix, resource, subject, size)
            .await?;
        assert!(!actual);

        Ok(())
    }

    /// Tests the requests do not exceed the rate limit.
    #[tokio::test]
    async fn token_bucket_redis_case2() -> Result<(), ()> {
        // prev
        initialize_redis().await?;

        // arrange
        let limit_count = 1;
        let size = Duration::from_secs(1);
        let mut client = rate_limiter_redis::RateLimiterRedis::open(CONN, limit_count).await?;
        let key_prefix = "test5";
        let resource = "data";
        let subject = "andy";

        // act && assert
        let actual = client
            .record_token_bucket(key_prefix, resource, subject, size)
            .await?;
        assert!(actual);

        let actual = client
            .record_token_bucket(key_prefix, resource, subject, size)
            .await?;
        assert!(!actual);

        std::thread::sleep(Duration::from_secs(1));

        let actual = client
            .record_token_bucket(key_prefix, resource, subject, size)
            .await?;
        assert!(actual);

        Ok(())
    }
}
