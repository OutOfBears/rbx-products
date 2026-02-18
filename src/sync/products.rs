use std::collections::HashMap;

use dyn_fmt::AsStrFormatExt;
use nestify::nest;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tokio::{fs, io::AsyncWriteExt};
use toml_edit::Array;

use crate::utils::{deserialize_regex_vec, serialize_regex_vec};
use crate::{
    Result, get_toml_value,
    ui::diffs::{DiffChange, ProductDiff, ProductDiffs},
};

nest! {
    #[derive(Default, Debug, Clone, Serialize, Deserialize)]*
    #[serde(rename_all = "kebab-case")]*
    pub struct VCSProducts {
        pub metadata: pub struct Metadata {
            pub universe_id: u64,
            pub luau_file: Option<String>,
            pub discount_prefix: Option<String>,
            #[serde(default, deserialize_with = "deserialize_regex_vec", serialize_with = "serialize_regex_vec")]
            pub name_filters: Option<Vec<Regex>>,
        },

        #[serde(default)]
        pub gamepasses: HashMap<String, pub struct Product {
            pub id: Option<u64>,
            pub name: String,
            pub prefix: Option<String>,
            pub description: Option<String>,
            pub active: bool,
            pub discount: Option<u8>,
            pub price: i64,
            pub regional_pricing: Option<bool>,
        }>,

        #[serde(default)]
        pub products: HashMap<String, Product>,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProductType {
    GamePass,
    DevProduct,
}
pub enum MultiProduct {
    GamePass(Product),
    DevProduct(Product),
}

macro_rules! check_diff {
    ($diffs:expr, $field:expr, $old:expr, $new:expr, $variant:ident) => {
        if $old != $new {
            $diffs.push(DiffChange::Changed(ProductDiff::$variant(
                $old.clone(),
                $new.clone(),
            )));
        } else {
            $diffs.push(DiffChange::Unchanged(ProductDiff::$variant(
                $old.clone(),
                $new.clone(),
            )));
        }
    };
}

impl VCSProducts {
    pub async fn get_products() -> Result<Self> {
        let file_data = fs::read("products.toml").await?;
        let products: VCSProducts = toml::from_slice(&file_data)?;
        Ok(products)
    }

    pub async fn save_products(&self) -> Result<()> {
        let mut toml_products: toml_edit::DocumentMut;

        if let Ok(data) = fs::read("products.toml").await {
            let document_string = String::from_utf8(data.clone())?;
            toml_products = document_string.parse()?;
        } else {
            toml_products = toml_edit::DocumentMut::new();
        }

        let mut metadata = get_toml_value!(toml_products, "metadata");
        let mut gamepasses = get_toml_value!(toml_products, "gamepasses");
        let mut products = get_toml_value!(toml_products, "products");

        if let Some(discount_prefix) = &self.metadata.discount_prefix {
            metadata["discount-prefix"] = toml_edit::value(discount_prefix.clone());
        } else {
            metadata.remove("discount-prefix");
        }

        metadata["universe-id"] = toml_edit::value(self.metadata.universe_id as i64);

        if let Some(luau_file) = &self.metadata.luau_file {
            metadata["luau-file"] = toml_edit::value(luau_file);
        } else {
            metadata.remove("luau-file");
        }

        let filters = self
            .metadata
            .name_filters
            .clone()
            .unwrap_or_default()
            .iter()
            .map(|x| x.as_str().to_string())
            .collect::<Vec<_>>();

        metadata["name-filters"] = toml_edit::value(Array::from_iter(filters.iter()));

        for gamepass in &self.gamepasses {
            gamepasses[&gamepass.0] = gamepass.1.into();
        }
        for product in &self.products {
            products[&product.0] = product.1.into();
        }

        toml_products["metadata"] = toml_edit::Item::Table(metadata);
        toml_products["gamepasses"] = toml_edit::Item::Table(gamepasses);
        toml_products["products"] = toml_edit::Item::Table(products);

        fs::write("products.toml", toml_products.to_string()).await?;
        Ok(())
    }

    pub async fn serialize_luau(&self) -> Result<()> {
        let products_lua_file = match self.metadata.luau_file.clone() {
            Some(file) => file,
            None => return Ok(()),
        };

        let mut file = fs::File::create(products_lua_file).await?;
        let mut contents = String::new();

        let serialize = |contents: &mut String, products: &HashMap<String, Product>| {
            let mut values: Vec<_> = products.values().collect();
            values.sort_by(|a, b| a.id.cmp(&b.id));

            for (index, product) in values.iter().enumerate() {
                *contents += &format!(
                    "\t\t[{:?}] = {{ id = {:?}, price = {} }}",
                    product.get_title(),
                    product.id.unwrap_or(0),
                    product.get_price()
                );

                if index != products.len() - 1 {
                    *contents += ",\n";
                } else {
                    *contents += "\n";
                }
            }
        };

        contents += "-- This file is automatically generated by rbx-products. Do not edit this file directly.\n";
        contents += "export type Product = { id: number, price: number }\n\n";
        contents += "return {\n\tGamepasses = {\n";
        serialize(&mut contents, &self.gamepasses);
        contents += "\t} :: {[string]: Product},\n\n\tProducts = {\n";
        serialize(&mut contents, &self.products);
        contents += "\t} :: {[string]: Product}\n}";

        file.write_all(contents.as_bytes()).await?;

        Ok(())
    }
}

impl Product {
    pub fn has_discount(&self) -> bool {
        if let Some(discount) = self.discount
            && discount > 0
        {
            true
        } else {
            false
        }
    }

    pub fn get_price(&self) -> u64 {
        if let Some(discount) = self.discount
            && discount > 0
        {
            (self.price as f64 * (1.0 - (discount as f64 / 100.0))).floor() as u64
        } else {
            self.price as u64
        }
    }

    pub fn get_title(&self) -> String {
        if self.has_discount() {
            return self.name.clone();
        }

        if let Some(prefix) = &self.prefix {
            format!("{} {}", prefix, self.name)
        } else {
            self.name.clone()
        }
    }

    pub fn diff(&self, other: &Self, metadata: Option<&Metadata>) -> Option<ProductDiffs> {
        let mut diffs = vec![] as Vec<DiffChange>;

        let title = if let Some(metadata) = metadata {
            if let (Some(discount), Some(prefix)) = (self.discount, &metadata.discount_prefix) {
                format!("{} {}", prefix.format(&[discount]), self.get_title())
            } else {
                self.get_title()
            }
        } else {
            self.get_title()
        };

        let active = self.active;
        let price = self.get_price();
        let description = self.description.clone().unwrap_or(String::default());

        check_diff!(diffs, Title, other.name, title, Title);
        check_diff!(
            diffs,
            Description,
            other.description.clone().unwrap(),
            description,
            Description
        );
        check_diff!(diffs, Price, other.price as u64, price, Price);
        check_diff!(
            diffs,
            RegionalPricing,
            other.regional_pricing.unwrap_or(false),
            self.regional_pricing.unwrap_or(false),
            RegionalPricing
        );
        check_diff!(diffs, Active, other.active, active, Active);

        let has_diffs = diffs.iter().any(|d| match d {
            DiffChange::Changed(_) => true,
            _ => false,
        });

        if has_diffs {
            Some(ProductDiffs {
                name: self.name.clone(),
                id: self.id.unwrap_or(0) as u64,
                diffs,
            })
        } else {
            None
        }
    }
}

impl From<&Product> for toml_edit::Item {
    fn from(prod: &Product) -> Self {
        let mut table = toml_edit::Table::new();

        if let Some(id) = prod.id {
            table["id"] = toml_edit::value(id as i64);
        }

        if let Some(prefix) = &prod.prefix {
            table["prefix"] = toml_edit::value(prefix.clone());
        }

        table["name"] = toml_edit::value(&prod.name);

        if let Some(desc) = &prod.description {
            table["description"] = toml_edit::value(desc);
        }

        table["active"] = toml_edit::value(prod.active);

        if let Some(discount) = prod.discount {
            table["discount"] = toml_edit::value(discount as i64);
        }

        table["price"] = toml_edit::value(prod.price);

        if let Some(regional_pricing) = prod.regional_pricing {
            table["regional-pricing"] = toml_edit::value(regional_pricing);
        }

        toml_edit::Item::Table(table)
    }
}
