use super::API_CLIENT;
use super::model::{DevProduct, GamePass};

use crate::Result;
use crate::api::model::{DevProductPage, GamePassPage, ProductUpdateRequest};
use crate::sync::products::{MultiProduct, Product};

pub async fn fetch_all_products(universe_id: u64) -> Result<Vec<MultiProduct>> {
    let gamepasses = fetch_all_gamepasses(universe_id).await?;
    let products = fetch_all_dev_products(universe_id).await?;

    let mut all_products: Vec<MultiProduct> = Vec::new();

    all_products.extend(
        gamepasses
            .into_iter()
            .map(|x| MultiProduct::GamePass(Product::from(&x))),
    );

    all_products.extend(
        products
            .into_iter()
            .map(|x| MultiProduct::DevProduct(Product::from(&x))),
    );

    Ok(all_products)
}

pub async fn fetch_all_dev_products(universe_id: u64) -> Result<Vec<DevProduct>> {
    let mut products = vec![];

    let page_size = 100;
    let mut page_cursor: String = String::default();

    loop {
        let mut req = API_CLIENT
            .get(&format!(
                "https://apis.roblox.com/developer-products/v2/universes/{}/developer-products/creator",
                universe_id
            ))
            .query(&[("pageSize", page_size.to_string())]);

        if !page_cursor.is_empty() {
            req = req.query(&[("pageToken", page_cursor.clone())]);
        }

        let resp: DevProductPage = req.send().await?.json().await?;

        products.extend(resp.developer_products);

        match resp.next_page_token {
            Some(cursor) => {
                page_cursor = cursor;
            }
            None => break,
        }
    }

    Ok(products)
}

pub async fn fetch_all_gamepasses(universe_id: u64) -> Result<Vec<GamePass>> {
    let mut gamepasses = vec![];

    let page_size = 100;
    let mut page_cursor: String = String::default();

    loop {
        let mut req = API_CLIENT
            .get(&format!(
                "https://apis.roblox.com/game-passes/v1/universes/{}/game-passes/creator",
                universe_id
            ))
            .query(&[("pageSize", page_size.to_string())]);

        if !page_cursor.is_empty() {
            req = req.query(&[("pageToken", page_cursor.clone())]);
        }

        let resp: GamePassPage = req.send().await?.json().await?;

        gamepasses.extend(resp.game_passes);

        match resp.next_page_token {
            Some(cursor) => {
                page_cursor = cursor;
            }
            None => break,
        }
    }

    Ok(gamepasses)
}

pub async fn update_dev_product(
    universe_id: u64,
    product_id: u64,
    update: &ProductUpdateRequest,
) -> Result<()> {
    API_CLIENT
        .patch(&format!(
            "https://apis.roblox.com/developer-products/v2/universes/{}/developer-products/{}",
            universe_id, product_id
        ))
        .multipart(update.into())
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

pub async fn update_gamepass(
    universe_id: u64,
    game_pass_id: u64,
    update: &ProductUpdateRequest,
) -> Result<()> {
    API_CLIENT
        .patch(&format!(
            "https://apis.roblox.com/game-passes/v1/universes/{}/game-passes/{}",
            universe_id, game_pass_id
        ))
        .multipart(update.into())
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

pub async fn create_dev_product(
    universe_id: u64,
    product: &ProductUpdateRequest,
) -> Result<DevProduct> {
    let resp: DevProduct = API_CLIENT
        .post(&format!(
            "https://apis.roblox.com/developer-products/v2/universes/{}/developer-products",
            universe_id
        ))
        .multipart(product.into())
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(resp)
}

pub async fn create_gamepass(
    universe_id: u64,
    gamepass: &ProductUpdateRequest,
) -> Result<GamePass> {
    let resp: GamePass = API_CLIENT
        .post(&format!(
            "https://apis.roblox.com/game-passes/v1/universes/{}/game-passes",
            universe_id
        ))
        .multipart(gamepass.into())
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(resp)
}
