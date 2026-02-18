use log::info;

use crate::Result;
use crate::api::model::ProductUpdateRequest;
use crate::api::products::{
    create_dev_product, create_gamepass, fetch_all_products, update_dev_product, update_gamepass,
};
use crate::sync::products::{MultiProduct, Product, ProductType, VCSProducts};
use crate::ui::confirm::{ConfirmState, ConfirmViewer};
use crate::ui::diffs::{DiffViewer, ProductDiffs};

pub struct Uploader {
    local_products: VCSProducts,
    remote_products: Vec<MultiProduct>,
}

fn apply_discount_prefix(product: &mut Product, prefix: Option<String>) {
    let prefix = prefix.unwrap_or("💲{}% OFF💲".to_string());

    if product.has_discount() {
        product.name = format!(
            "{} {}",
            prefix.replace("{}", &product.discount.unwrap_or(0).to_string()),
            product.name
        );
    }
}

impl Uploader {
    fn has_empty_products(&self) -> bool {
        let has_empty_gamepasses = self
            .local_products
            .gamepasses
            .iter()
            .any(|(_, gp)| gp.id.is_none());
        let has_empty_devproducts = self
            .local_products
            .products
            .iter()
            .any(|(_, dp)| dp.id.is_none());

        has_empty_gamepasses || has_empty_devproducts
    }

    async fn upload_empty(&mut self, overwrite: bool) -> Result<()> {
        if !self.has_empty_products() {
            return Ok(());
        }

        if !overwrite {
            let prompt_result =
                ConfirmViewer::show_prompt("Would you like to upload non-existant products?").await;

            if prompt_result != ConfirmState::Confirmed {
                info!("not uploading non-existant products");
                return Ok(());
            }
        }

        let universe_id = self.local_products.metadata.universe_id.clone();
        let upload_product =
            async |universe_id: u64, product: Product, product_type: ProductType| -> Result<u64> {
                let update_request = ProductUpdateRequest::from(&product);
                let product_id = match product_type {
                    ProductType::GamePass => {
                        create_gamepass(universe_id, &update_request)
                            .await?
                            .game_pass_id
                    }

                    ProductType::DevProduct => {
                        create_dev_product(universe_id, &update_request)
                            .await?
                            .product_id
                    }
                };

                info!(
                    "uploaded {:?} '{}' with id {}",
                    product_type, product.name, product_id
                );

                Ok(product_id)
            };

        let mut gamepass_futures = vec![];
        let mut devproduct_futures = vec![];

        info!(
            "uploading {} products, and {} gamepasses in universe {}",
            self.local_products.products.len(),
            self.local_products.gamepasses.len(),
            universe_id
        );

        for (name, gamepass) in &self.local_products.gamepasses {
            if gamepass.id.is_none() {
                let universe_id = universe_id.clone();
                let name = name.clone();
                let mut gamepass = gamepass.clone();

                apply_discount_prefix(
                    &mut gamepass,
                    self.local_products.metadata.discount_prefix.clone(),
                );

                let future = (async move {
                    let product_id =
                        upload_product(universe_id, gamepass, ProductType::GamePass).await;

                    match product_id {
                        Ok(id) => Some((name, id)),
                        Err(e) => {
                            log::error!("failed to upload gamepass '{}': {}", name, e);
                            None
                        }
                    }
                })
                .await;

                gamepass_futures.push(future);
            }
        }

        for (name, devproduct) in &self.local_products.products {
            if devproduct.id.is_none() {
                let universe_id = universe_id.clone();
                let name = name.clone();
                let mut devproduct = devproduct.clone();

                apply_discount_prefix(
                    &mut devproduct,
                    self.local_products.metadata.discount_prefix.clone(),
                );

                let future = (async move {
                    let product_id =
                        upload_product(universe_id, devproduct.clone(), ProductType::DevProduct)
                            .await;

                    match product_id {
                        Ok(id) => Some((name, id)),
                        Err(e) => {
                            log::error!("failed to upload dev product '{}': {}", name, e);
                            None
                        }
                    }
                })
                .await;

                devproduct_futures.push(future);
            }
        }

        gamepass_futures.into_iter().for_each(|res| {
            if let Some((name, id)) = res {
                self.local_products
                    .gamepasses
                    .get_mut(name.as_str())
                    .unwrap()
                    .id = Some(id as u64);
            }
        });

        devproduct_futures.into_iter().for_each(|res| {
            if let Some((name, id)) = res {
                self.local_products
                    .products
                    .get_mut(name.as_str())
                    .unwrap()
                    .id = Some(id as u64);
            }
        });

        self.local_products.save_products().await?;
        self.local_products.serialize_luau().await?;

        Ok(())
    }

    async fn upload_modified(&mut self, overwrite: bool) -> Result<()> {
        let mut product_diffs = vec![];

        let universe_id = self.local_products.metadata.universe_id.clone();
        let products = &self.remote_products;
        let mut all_local_products = vec![];

        all_local_products.extend(self.local_products.gamepasses.values().cloned());
        all_local_products.extend(self.local_products.products.values().cloned());

        product_diffs.extend(
            all_local_products
                .iter()
                .filter_map(|local_product| {
                    let id = match local_product.id {
                        Some(id) => id,
                        None => return None,
                    };

                    let (product_type, remote_product) =
                        match products.iter().find(|multi_product| match multi_product {
                            MultiProduct::GamePass(pass) => pass.id.unwrap() == id,
                            MultiProduct::DevProduct(prod) => prod.id.unwrap() == id,
                        }) {
                            Some(MultiProduct::GamePass(pass)) => (ProductType::GamePass, pass),
                            Some(MultiProduct::DevProduct(prod)) => (ProductType::DevProduct, prod),
                            None => return None,
                        };

                    local_product
                        .diff(&remote_product, Some(&self.local_products.metadata))
                        .map(|diff| (product_type, diff))
                })
                .collect::<Vec<_>>(),
        );

        let mut all_diffs = vec![] as Vec<(ProductType, ProductDiffs)>;
        all_diffs.extend(product_diffs);
        all_diffs.sort_by(|a, b| match b.0.cmp(&a.0) {
            std::cmp::Ordering::Equal => a.1.id.cmp(&b.1.id),
            other => other,
        });

        if all_diffs.len() == 0 {
            info!("no differences found between local and universe products.");
            return Ok(());
        }

        let diffs: Vec<(ProductType, u64)>;

        if !overwrite {
            diffs = DiffViewer::confirm_diffs(all_diffs.iter().cloned().collect()).await;

            let apply = ConfirmViewer::show_prompt("Would you like to sync products?").await;

            if apply == ConfirmState::Closed {
                info!("user aborted sync.");
                return Ok(());
            }
        } else {
            diffs = all_diffs
                .iter()
                .cloned()
                .map(|(product_type, diff)| (product_type, diff.id))
                .collect::<Vec<_>>();
        }

        if diffs.len() == 0 {
            info!("No changes to apply.");
            return Ok(());
        }

        info!("syncing {} product(s)", diffs.len());

        for (product_type, id) in diffs {
            let mut local_product = match product_type {
                ProductType::GamePass => self
                    .local_products
                    .gamepasses
                    .values()
                    .find(|gp| gp.id == Some(id)),
                ProductType::DevProduct => self
                    .local_products
                    .products
                    .values()
                    .find(|prod| prod.id == Some(id)),
            }
            .unwrap()
            .clone();

            let name = local_product.name.clone();

            apply_discount_prefix(
                &mut local_product,
                self.local_products.metadata.discount_prefix.clone(),
            );

            let update_request = ProductUpdateRequest::from(&local_product);

            match product_type {
                ProductType::GamePass => {
                    update_gamepass(universe_id, id, &update_request).await?;
                }
                ProductType::DevProduct => {
                    update_dev_product(universe_id, id, &update_request).await?;
                }
            }

            info!("synced {:?} '{}' (id: {})", product_type, name, id);
        }

        info!("finished syncing all gamepasses/products");

        Ok(())
    }

    async fn create() -> Result<Self> {
        info!("fetching local products");
        let local_products_data = VCSProducts::get_products().await?;

        info!("fetching remote products");
        let remote_product_data =
            fetch_all_products(local_products_data.metadata.universe_id).await?;

        info!(
            "fetched {} local products, {} remote products",
            local_products_data.gamepasses.len() + local_products_data.products.len(),
            remote_product_data.len()
        );

        Ok(Self {
            local_products: local_products_data,
            remote_products: remote_product_data,
        })
    }

    pub async fn upload(overwrite: bool) -> Result<()> {
        let mut uploader = Uploader::create().await?;

        let mut run_upload = async || -> Result<()> {
            uploader.upload_empty(overwrite).await?;
            uploader.upload_modified(overwrite).await?;

            Ok(())
        };

        let upload_result = run_upload().await;

        uploader.local_products.save_products().await?;
        uploader.local_products.serialize_luau().await?;

        if let Err(e) = upload_result {
            info!("failed to upload modified products: {}, aborting upload", e);
            return Err(e);
        }

        Ok(())
    }
}
