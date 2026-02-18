use log::info;

use crate::Result;
use crate::api::products::fetch_all_products;
use crate::sync::products::{MultiProduct, Product, ProductType, VCSProducts};
use crate::utils::{canonical_name, format_name, is_censored};

pub struct Downloader {
    local_products: VCSProducts,
    remote_products: Vec<MultiProduct>,
}

impl Downloader {
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

        Ok(Downloader {
            local_products: local_products_data,
            remote_products: remote_product_data,
        })
    }

    pub async fn download(overwrite: bool) -> Result<()> {
        let downloader = Downloader::create().await?;

        let mut local_products_data = downloader.local_products;
        let remote_product_data = downloader.remote_products;

        let filters = &local_products_data.metadata.name_filters;

        info!(
            "merging local products, and remote products (overwrite: {})",
            overwrite
        );

        remote_product_data.iter().for_each(|multi_product| {
            let (product, product_type): (Product, ProductType) = match multi_product {
                MultiProduct::GamePass(prod) => (prod.clone(), ProductType::GamePass),
                MultiProduct::DevProduct(prod) => (prod.clone(), ProductType::DevProduct),
            };

            let name = format_name(canonical_name(product.name.clone(), &filters));

            let existing = match product_type {
                ProductType::GamePass => local_products_data.gamepasses.iter().find(|(_, x)| {
                    x.id.map(|id| id as i64).unwrap_or(-1)
                        == product.id.map(|id| id as i64).unwrap_or(-1)
                }),

                ProductType::DevProduct => local_products_data.products.iter().find(|(_, x)| {
                    x.id.map(|id| id as i64).unwrap_or(-1)
                        == product.id.map(|id| id as i64).unwrap_or(-1)
                }),
            };

            let mut product = Product {
                id: product.id,
                name: if !overwrite && existing.is_some() {
                    canonical_name(existing.unwrap().1.name.clone(), &filters)
                } else {
                    canonical_name(product.name.clone(), &filters)
                },
                prefix: if !overwrite && let Some(existing_product) = existing {
                    existing_product.1.prefix.clone()
                } else {
                    None
                },
                description: if !overwrite && let Some(existing_product) = existing {
                    existing_product.1.description.clone()
                } else {
                    product.description
                },
                active: product.active,
                discount: match existing {
                    Some((_, existing_product)) if existing_product.has_discount() => {
                        existing_product.discount
                    }
                    _ => None,
                },
                price: if let Some(existing_product) = existing {
                    if overwrite {
                        product.price
                    } else {
                        existing_product.1.price
                    }
                } else {
                    product.price
                },
                regional_pricing: if let Some(existing_product) = existing {
                    if overwrite {
                        product.regional_pricing
                    } else {
                        existing_product.1.regional_pricing
                    }
                } else {
                    product.regional_pricing
                },
            };

            if let Some(regional_pricing) = product.regional_pricing
                && !regional_pricing
            {
                product.regional_pricing = None;
            }

            if !overwrite && let Some(existing_product) = existing {
                if let Some(desc) = product.description.clone()
                    && is_censored(&desc)
                {
                    product.description = existing_product.1.description.clone();
                }
            }

            let key = match existing.is_none() {
                true => name.clone(),
                false => existing.unwrap().0.clone(),
            };

            match product_type {
                ProductType::GamePass => local_products_data.gamepasses.insert(key, product),
                ProductType::DevProduct => local_products_data.products.insert(key, product),
            };
        });

        info!("finished merging products, saving to disk");
        local_products_data.save_products().await?;

        info!("serializing products to luau format");
        local_products_data.serialize_luau().await?;

        Ok(())
    }
}
