use diesel::{Connection, SqliteConnection};
use kafi_kaesseli::currency_handling::currency_formatter::CurrencyFormatterImpl;
use kafi_kaesseli::currency_handling::currency_parser::CurrencyParserImpl;
use kafi_kaesseli::data_loader::{DataLoader, DataLoaderImpl};
use kafi_kaesseli::message_handler::{MessageHandler, MessageHandlerImpl};
use kafi_kaesseli::message_router::MessageRouterImpl;
use kafi_kaesseli::models::{Message, User};
use kafi_kaesseli::services::balance_service::BalanceServiceImpl;
use kafi_kaesseli::services::product_service::ProductServiceImpl;
use kafi_kaesseli::services::transaction_service::TransactionServiceImpl;
use kafi_kaesseli::services::user_service::UserServiceImpl;
use tbot::types::parameters::Text;
use tbot::types::{message, update};
use tbot::{prelude::*, types, Bot};

mod product_data_provider;

use product_data_provider::ProductDataProviderImpl;

static DATABASE_NAME: &str = "database.sqlite";
static PRODUCT_DATA: &str = include_str!("../products.toml");

fn main() {
    {
        let database_connection = SqliteConnection::establish(DATABASE_NAME).unwrap();
        kafi_kaesseli::run_migrations(&database_connection).unwrap();

        let data_loader = DataLoaderImpl::new(
            &database_connection,
            Box::new(ProductDataProviderImpl::new(PRODUCT_DATA)),
        );

        data_loader.load_product_data().unwrap();
    }

    let mut bot = Bot::from_env("BOT_TOKEN").event_loop();

    bot.unhandled(|context| {
        if let update::Kind::Message(types::Message {
            chat,
            kind: message::Kind::Text(text),
            from: Some(types::User { id, first_name, .. }),
            ..
        }) = &context.update
        {
            let database_connection = match SqliteConnection::establish(DATABASE_NAME) {
                Ok(database_connection) => database_connection,
                Err(_) => {
                    let reply = context
                        .bot
                        .send_message(chat.id, Text::plain("Database not available"))
                        .into_future()
                        .map_err(|err| {
                            dbg!(err);
                        });

                    tbot::spawn(reply);

                    return;
                }
            };

            let message_handler = MessageHandlerImpl::new(
                Box::new(MessageRouterImpl::new(
                    Box::new(ProductServiceImpl::new(&database_connection)),
                    Box::new(CurrencyParserImpl::default()),
                )),
                Box::new(UserServiceImpl::new(&database_connection)),
                Box::new(ProductServiceImpl::new(&database_connection)),
                Box::new(TransactionServiceImpl::new(&database_connection)),
                Box::new(BalanceServiceImpl::new(&database_connection)),
                Box::new(CurrencyFormatterImpl::default()),
            );

            let responses = message_handler.handle_message(&Message {
                sender: User {
                    id: id.to_string(),
                    name: first_name.to_string(),
                },
                contents: text.value.clone(),
            });

            for response in responses {
                let reply = context
                    .bot
                    .send_message(chat.id, Text::markdown(&response.contents))
                    .into_future()
                    .map_err(|_| ());

                tbot::spawn(reply);
            }
        }
    });

    bot.polling().start();
}

#[cfg(test)]
mod tests {
    use super::*;
    use kafi_kaesseli::data_loader::data_provider::DataProvider;

    #[test]
    fn validate_products_toml() {
        let _ = ProductDataProviderImpl::new(PRODUCT_DATA)
            .get_data()
            .collect::<Vec<_>>();
    }
}
