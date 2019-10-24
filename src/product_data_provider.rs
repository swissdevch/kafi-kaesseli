use kafi_kaesseli::data_loader::data_provider::DataProvider;
use kafi_kaesseli::models::{Product, Rappen};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct TomlRoot {
    #[serde(rename = "product")]
    products: Vec<TomlProduct>,
}

#[derive(Deserialize, Debug)]
struct TomlProduct {
    identifier: String,
    name: String,
    price: Rappen,
}

pub struct ProductDataProviderImpl {
    product_data: &'static str,
}

impl ProductDataProviderImpl {
    pub fn new(product_data: &'static str) -> Self {
        Self { product_data }
    }
}

impl DataProvider<Product> for ProductDataProviderImpl {
    fn get_data(&self) -> Box<dyn Iterator<Item = Result<Product, ()>>> {
        let data: TomlRoot = toml::from_str(self.product_data).unwrap();

        let iterator = data.products.into_iter().map(
            |TomlProduct {
                 identifier,
                 name,
                 price,
             }| {
                Ok(Product {
                    identifier,
                    name,
                    price,
                })
            },
        );

        Box::new(iterator)
    }
}
