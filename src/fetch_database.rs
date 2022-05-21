pub async fn update_database_thread(
    papers: crate::storage::PapersStorage,
    uri: url::Url,
    update_periodicity: std::time::Duration,
) {
    let mut interval = tokio::time::interval(update_periodicity);

    loop {
        interval.tick().await;

        let new_papers = update_paper_database(uri.clone()).await;

        match new_papers {
            Ok(parsed_papers) => {
                *papers
                    .lock()
                    .expect("An error occurred during papers mutex acquisition") = parsed_papers;

                log::info!(
                    "Papers database update executed successfully. Papers database size: {}",
                    papers.lock().unwrap().len()
                );
            }
            Err(e) => {
                log::info!("An error occurred during papers database update: {}", e);
            }
        }
    }
}

async fn update_paper_database(uri: url::Url) -> reqwest::Result<crate::storage::PaperDatabase> {
    let resp = reqwest::get(uri)
        .await?
        .json::<std::collections::HashMap<String, crate::storage::Paper>>()
        .await?;

    let new_papers = crate::storage::PaperDatabase::new(resp);

    Ok(new_papers)
}
