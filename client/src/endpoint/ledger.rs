//! Contains the endpoint for all ledgers.
use super::{Body, Cursor, Direction, IntoRequest, Limit, Order, Records};
use error::Result;
use http::{Request, Uri};
use resources::{Effect, Ledger, Operation, Transaction};
use std::str::FromStr;
use uri::{self, TryFromUri, UriWrap};

/// Represents the all ledgers end point for the stellar horizon server. The endpoint
/// will return all ledgers filtered by a myriad of different query params.
///
/// <https://www.stellar.org/developers/horizon/reference/endpoints/ledgers-all.html>
///
/// ## Example
/// ```
/// use stellar_client::sync::Client;
/// use stellar_client::endpoint::ledger;
///
/// let client      = Client::horizon_test().unwrap();
/// let endpoint    = ledger::All::default();
/// let records     = client.request(endpoint).unwrap();
/// #
/// # assert!(records.records().len() > 0);
/// ```
#[derive(Debug, Default, Clone)]
pub struct All {
    cursor: Option<String>,
    order: Option<Direction>,
    limit: Option<u32>,
}

impl_cursor!(All);
impl_limit!(All);
impl_order!(All);

impl All {
    fn has_query(&self) -> bool {
        self.order.is_some() || self.cursor.is_some() || self.limit.is_some()
    }
}

impl IntoRequest for All {
    type Response = Records<Ledger>;

    fn into_request(self, host: &str) -> Result<Request<Body>> {
        let mut uri = format!("{}/ledgers", host);

        if self.has_query() {
            uri.push_str("?");

            if let Some(order) = self.order {
                uri.push_str(&format!("order={}&", order.to_string()));
            }

            if let Some(cursor) = self.cursor {
                uri.push_str(&format!("cursor={}&", cursor));
            }

            if let Some(limit) = self.limit {
                uri.push_str(&format!("limit={}", limit));
            }
        }

        let uri = Uri::from_str(&uri)?;
        let request = Request::get(uri).body(Body::None)?;
        Ok(request)
    }
}

impl TryFromUri for All {
    fn try_from_wrap(wrap: &UriWrap) -> ::std::result::Result<All, uri::Error> {
        let params = wrap.params();
        Ok(All {
            cursor: params.get_parse("cursor").ok(),
            order: params.get_parse("order").ok(),
            limit: params.get_parse("limit").ok(),
        })
    }
}

#[cfg(test)]
mod all_ledgers_tests {
    use super::*;

    #[test]
    fn it_leaves_off_the_params_if_not_specified() {
        let ep = All::default();
        let req = ep.into_request("https://www.google.com").unwrap();
        assert_eq!(req.uri().path(), "/ledgers");
        assert_eq!(req.uri().query(), None);
    }

    #[test]
    fn it_puts_the_query_params_on_the_uri() {
        let ep = All::default()
            .with_cursor("CURSOR")
            .with_limit(123)
            .with_order(Direction::Desc);
        let req = ep.into_request("https://www.google.com").unwrap();
        assert_eq!(req.uri().path(), "/ledgers");
        assert_eq!(
            req.uri().query(),
            Some("order=desc&cursor=CURSOR&limit=123")
        );
    }

    #[test]
    fn it_parses_query_params_from_uri() {
        let uri: Uri = "/ledgers?order=desc&cursor=CURSOR&limit=123"
            .parse()
            .unwrap();
        let all = All::try_from(&uri).unwrap();
        assert_eq!(all.order, Some(Direction::Desc));
        assert_eq!(all.cursor, Some("CURSOR".to_string()));
        assert_eq!(all.limit, Some(123));
    }
}

/// Represents the ledger details endpoint for the stellar horizon server. The endpoint
/// will return a single ledger's details.
///
/// <https://www.stellar.org/developers/horizon/reference/endpoints/ledgers-single.html>
///
/// ## Example
/// ```
/// use stellar_client::sync::Client;
/// use stellar_client::endpoint::ledger;
///
/// let client      = Client::horizon_test().unwrap();
/// let endpoint    = ledger::Details::new(12345);
/// let record      = client.request(endpoint).unwrap();
/// #
/// # assert_eq!(record.sequence(), 12345);
/// ```
#[derive(Debug, Default)]
pub struct Details {
    sequence: u32,
}

impl Details {
    /// Returns a new endpoint for ledger details. Hand this to the client in order to request
    /// details about a ledger.
    ///
    /// In Stellar, the sequence number is the equivalent of Bitcoin's block height. Thus, by
    /// specifying a sequence number of 12345, we are specifying the 12345th ledger in the
    /// Stellar ledger chain (Stellar's blockchain is called a ledger chain).
    pub fn new(sequence: u32) -> Self {
        Self { sequence }
    }
}

impl IntoRequest for Details {
    type Response = Ledger;

    fn into_request(self, host: &str) -> Result<Request<Body>> {
        let uri = Uri::from_str(&format!("{}/ledgers/{}", host, self.sequence))?;
        let request = Request::get(uri).body(Body::None)?;
        Ok(request)
    }
}

#[cfg(test)]
mod ledger_details_tests {
    use super::*;

    #[test]
    fn it_can_make_a_ledger_details_uri() {
        let details = Details::new(12345);
        let request = details
            .into_request("https://horizon-testnet.stellar.org")
            .unwrap();
        assert_eq!(request.uri().host().unwrap(), "horizon-testnet.stellar.org");
        assert_eq!(request.uri().path(), "/ledgers/12345");
    }
}

/// Represents the payments for ledger endpoint on the stellar horizon server.
/// The endpoint will return all the payment for a single ledger in the chain.
///
/// <https://www.stellar.org/developers/horizon/reference/endpoints/payments-for-ledger.html>
///
/// ## Example
/// ```
/// use stellar_client::sync::Client;
/// use stellar_client::endpoint::{ledger, payment, transaction};
///
/// let client   = Client::horizon_test().unwrap();
///
/// // Grab payments so that we know we are getting a ledger involving payments
/// let payments = client.request(payment::All::default()).unwrap();
/// let payment = &payments.records()[0];
///
/// // Payment operations have a reference to the transaction
/// let txn_hash = payment.transaction();
///
/// // The transaction details can then tell us the ledger that
/// // the transaction was on
/// let txn = client.request(transaction::Details::new(txn_hash)).unwrap();
/// let sequence = txn.ledger();
///
/// // Now we issue a request for that ledgers payments
/// let endpoint = ledger::Payments::new(sequence);
/// let payments = client.request(endpoint).unwrap();
///
/// assert!(payments.records().len() > 0);
/// ```
#[derive(Debug, Clone)]
pub struct Payments {
    sequence: u32,
    cursor: Option<String>,
    order: Option<Direction>,
    limit: Option<u32>,
}

impl_cursor!(Payments);
impl_limit!(Payments);
impl_order!(Payments);

impl Payments {
    /// Creates a new payments endpoint struct.
    ///
    /// ```
    /// use stellar_client::endpoint::ledger;
    ///
    /// let payments = ledger::Payments::new(123);
    /// ```
    pub fn new(sequence: u32) -> Payments {
        Payments {
            sequence,
            cursor: None,
            order: None,
            limit: None,
        }
    }

    fn has_query(&self) -> bool {
        self.order.is_some() || self.cursor.is_some() || self.limit.is_some()
    }
}

impl IntoRequest for Payments {
    type Response = Records<Operation>;

    fn into_request(self, host: &str) -> Result<Request<Body>> {
        let mut uri = format!("{}/ledgers/{}/payments", host, self.sequence);

        if self.has_query() {
            uri.push_str("?");

            if let Some(order) = self.order {
                uri.push_str(&format!("order={}&", order.to_string()));
            }

            if let Some(cursor) = self.cursor {
                uri.push_str(&format!("cursor={}&", cursor));
            }

            if let Some(limit) = self.limit {
                uri.push_str(&format!("limit={}", limit));
            }
        }

        let uri = Uri::from_str(&uri)?;
        let request = Request::get(uri).body(Body::None)?;
        Ok(request)
    }
}

impl TryFromUri for Payments {
    fn try_from_wrap(wrap: &UriWrap) -> ::std::result::Result<Self, uri::Error> {
        match wrap.path() {
            ["ledgers", sequence, "payments"] => {
                let params = wrap.params();
                Ok(Self {
                    sequence: sequence.parse()?,
                    cursor: params.get_parse("cursor").ok(),
                    order: params.get_parse("order").ok(),
                    limit: params.get_parse("limit").ok(),
                })
            }
            _ => Err(uri::Error::invalid_path()),
        }
    }
}

#[cfg(test)]
mod ledger_payments_tests {
    use super::*;

    #[test]
    fn it_leaves_off_the_params_if_not_specified() {
        let ep = Payments::new(123);
        let req = ep.into_request("https://www.google.com").unwrap();
        assert_eq!(req.uri().path(), "/ledgers/123/payments");
        assert_eq!(req.uri().query(), None);
    }

    #[test]
    fn it_puts_the_query_params_on_the_uri() {
        let ep = Payments::new(123)
            .with_cursor("CURSOR")
            .with_limit(123)
            .with_order(Direction::Desc);
        let req = ep.into_request("https://www.google.com").unwrap();
        assert_eq!(req.uri().path(), "/ledgers/123/payments");
        assert_eq!(
            req.uri().query(),
            Some("order=desc&cursor=CURSOR&limit=123")
        );
    }

    #[test]
    fn it_parses_from_a_uri() {
        let uri: Uri = "/ledgers/123/payments?cursor=CURSOR&order=desc&limit=123"
            .parse()
            .unwrap();
        let ep = Payments::try_from(&uri).unwrap();
        assert_eq!(ep.sequence, 123);
        assert_eq!(ep.limit, Some(123));
        assert_eq!(ep.cursor, Some("CURSOR".to_string()));
        assert_eq!(ep.order, Some(Direction::Desc));
    }
}

/// Represents the transactions for ledger endpoint on the stellar horizon server.
/// The endpoint will return all the transactions for a single ledger in the chain.
///
/// <https://www.stellar.org/developers/horizon/reference/endpoints/transactions-for-ledger.html>
///
/// ## Example
/// ```
/// use stellar_client::sync::Client;
/// use stellar_client::endpoint::{ledger, transaction, Limit};
///
/// let client   = Client::horizon_test().unwrap();
///
/// // Grab transactions and associated ledger to ensure a ledger sequence with transactions
/// let txns = client.request(transaction::All::default().with_limit(1)).unwrap();
/// let txn = &txns.records()[0];
/// let sequence = txn.ledger();
///
/// // Now we issue a request for that ledgers transactions
/// let endpoint = ledger::Transactions::new(sequence);
/// let ledger_txns = client.request(endpoint).unwrap();
///
/// assert!(ledger_txns.records().len() > 0);
/// ```
#[derive(Debug, Clone)]
pub struct Transactions {
    sequence: u32,
    cursor: Option<String>,
    order: Option<Direction>,
    limit: Option<u32>,
}

impl_cursor!(Transactions);
impl_limit!(Transactions);
impl_order!(Transactions);

impl Transactions {
    /// Creates a new ledger::Transactions endpoint struct.
    ///
    /// ```
    /// use stellar_client::endpoint::ledger;
    ///
    /// let txns = ledger::Transactions::new(123);
    /// ```
    pub fn new(sequence: u32) -> Transactions {
        Transactions {
            sequence,
            cursor: None,
            order: None,
            limit: None,
        }
    }

    fn has_query(&self) -> bool {
        self.order.is_some() || self.cursor.is_some() || self.limit.is_some()
    }
}

impl IntoRequest for Transactions {
    type Response = Records<Transaction>;

    fn into_request(self, host: &str) -> Result<Request<Body>> {
        let mut uri = format!("{}/ledgers/{}/transactions", host, self.sequence);

        if self.has_query() {
            uri.push_str("?");

            if let Some(order) = self.order {
                uri.push_str(&format!("order={}&", order.to_string()));
            }

            if let Some(cursor) = self.cursor {
                uri.push_str(&format!("cursor={}&", cursor));
            }

            if let Some(limit) = self.limit {
                uri.push_str(&format!("limit={}", limit));
            }
        }

        let uri = Uri::from_str(&uri)?;
        let request = Request::get(uri).body(Body::None)?;
        Ok(request)
    }
}

impl TryFromUri for Transactions {
    fn try_from_wrap(wrap: &UriWrap) -> ::std::result::Result<Self, uri::Error> {
        match wrap.path() {
            ["ledgers", sequence, "transactions"] => {
                let params = wrap.params();
                Ok(Self {
                    sequence: sequence.parse()?,
                    cursor: params.get_parse("cursor").ok(),
                    order: params.get_parse("order").ok(),
                    limit: params.get_parse("limit").ok(),
                })
            }
            _ => Err(uri::Error::invalid_path()),
        }
    }
}

#[cfg(test)]
mod ledger_transactions_tests {
    use super::*;

    #[test]
    fn it_leaves_off_the_params_if_not_specified() {
        let ep = Transactions::new(123);
        let req = ep.into_request("https://www.google.com").unwrap();
        assert_eq!(req.uri().path(), "/ledgers/123/transactions");
        assert_eq!(req.uri().query(), None);
    }

    #[test]
    fn it_puts_the_query_params_on_the_uri() {
        let ep = Transactions::new(123)
            .with_cursor("CURSOR")
            .with_limit(123)
            .with_order(Direction::Desc);
        let req = ep.into_request("https://www.google.com").unwrap();
        assert_eq!(req.uri().path(), "/ledgers/123/transactions");
        assert_eq!(
            req.uri().query(),
            Some("order=desc&cursor=CURSOR&limit=123")
        );
    }

    #[test]
    fn it_parses_from_a_uri() {
        let uri: Uri = "/ledgers/123/transactions?cursor=CURSOR&order=desc&limit=123"
            .parse()
            .unwrap();
        let ep = Transactions::try_from(&uri).unwrap();
        assert_eq!(ep.sequence, 123);
        assert_eq!(ep.limit, Some(123));
        assert_eq!(ep.cursor, Some("CURSOR".to_string()));
        assert_eq!(ep.order, Some(Direction::Desc));
    }
}

/// Represents the effects for ledger endpoint on the stellar horizon server.
/// The endpoint will return all the effects for a single ledger in the chain.
///
/// <https://www.stellar.org/developers/horizon/reference/endpoints/effects-for-ledger.html>
///
/// ## Example
/// ```
/// use stellar_client::sync::Client;
/// use stellar_client::endpoint::{ledger, transaction, Limit};
///
/// let client   = Client::horizon_test().unwrap();
///
/// // Grab transactions and associated ledger to ensure a ledger sequence with transactions.
/// // We seek transactions because effects have no references to a ledger and a ledger with
/// // transactions by definition has effects.
/// let txns = client.request(transaction::All::default().with_limit(1)).unwrap();
/// let txn = &txns.records()[0];
/// let sequence = txn.ledger();
///
/// // Now we issue a request for that ledger's effects
/// let endpoint = ledger::Effects::new(sequence);
/// let ledger_effects = client.request(endpoint).unwrap();
///
/// assert!(ledger_effects.records().len() > 0);
/// ```
#[derive(Debug, Clone)]
pub struct Effects {
    sequence: u32,
    cursor: Option<String>,
    order: Option<Direction>,
    limit: Option<u32>,
}

impl_cursor!(Effects);
impl_limit!(Effects);
impl_order!(Effects);

impl Effects {
    /// Creates a new ledger::Effects endpoint struct.
    ///
    /// ```
    /// use stellar_client::endpoint::ledger;
    ///
    /// let txns = ledger::Effects::new(123);
    /// ```
    pub fn new(sequence: u32) -> Effects {
        Effects {
            sequence,
            cursor: None,
            order: None,
            limit: None,
        }
    }

    fn has_query(&self) -> bool {
        self.order.is_some() || self.cursor.is_some() || self.limit.is_some()
    }
}

impl IntoRequest for Effects {
    type Response = Records<Effect>;

    fn into_request(self, host: &str) -> Result<Request<Body>> {
        let mut uri = format!("{}/ledgers/{}/effects", host, self.sequence);

        if self.has_query() {
            uri.push_str("?");

            if let Some(order) = self.order {
                uri.push_str(&format!("order={}&", order.to_string()));
            }

            if let Some(cursor) = self.cursor {
                uri.push_str(&format!("cursor={}&", cursor));
            }

            if let Some(limit) = self.limit {
                uri.push_str(&format!("limit={}", limit));
            }
        }

        let uri = Uri::from_str(&uri)?;
        let request = Request::get(uri).body(Body::None)?;
        Ok(request)
    }
}

impl TryFromUri for Effects {
    fn try_from_wrap(wrap: &UriWrap) -> ::std::result::Result<Self, uri::Error> {
        match wrap.path() {
            ["ledgers", sequence, "effects"] => {
                let params = wrap.params();
                Ok(Self {
                    sequence: sequence.parse()?,
                    cursor: params.get_parse("cursor").ok(),
                    order: params.get_parse("order").ok(),
                    limit: params.get_parse("limit").ok(),
                })
            }
            _ => Err(uri::Error::invalid_path()),
        }
    }
}

#[cfg(test)]
mod ledger_effects_tests {
    use super::*;

    #[test]
    fn it_leaves_off_the_params_if_not_specified() {
        let ep = Effects::new(123);
        let req = ep.into_request("https://www.google.com").unwrap();
        assert_eq!(req.uri().path(), "/ledgers/123/effects");
        assert_eq!(req.uri().query(), None);
    }

    #[test]
    fn it_puts_the_query_params_on_the_uri() {
        let ep = Effects::new(123)
            .with_cursor("CURSOR")
            .with_limit(123)
            .with_order(Direction::Desc);
        let req = ep.into_request("https://www.google.com").unwrap();
        assert_eq!(req.uri().path(), "/ledgers/123/effects");
        assert_eq!(
            req.uri().query(),
            Some("order=desc&cursor=CURSOR&limit=123")
        );
    }

    #[test]
    fn it_parses_from_a_uri() {
        let uri: Uri = "/ledgers/123/effects?cursor=CURSOR&order=desc&limit=123"
            .parse()
            .unwrap();
        let ep = Effects::try_from(&uri).unwrap();
        assert_eq!(ep.sequence, 123);
        assert_eq!(ep.limit, Some(123));
        assert_eq!(ep.cursor, Some("CURSOR".to_string()));
        assert_eq!(ep.order, Some(Direction::Desc));
    }
}

/// Represents the operations for ledger endpoint on the stellar horizon server.
/// The endpoint will return all the operations for a single ledger in the chain.
///
/// <https://www.stellar.org/developers/horizon/reference/endpoints/operations-for-ledger.html>
///
/// ## Example
/// ```
/// use stellar_client::sync::Client;
/// use stellar_client::endpoint::{ledger, transaction, Limit};
///
/// let client   = Client::horizon_test().unwrap();
///
/// // Grab transactions and associated ledger to ensure a ledger sequence with transactions.
/// // We seek transactions because operations have no references to a ledger and a ledger with
/// // transactions by definition has operations.
/// let txns = client.request(transaction::All::default().with_limit(1)).unwrap();
/// let txn = &txns.records()[0];
/// let sequence = txn.ledger();
///
/// // Now we issue a request for that ledger's operations
/// let endpoint = ledger::Operations::new(sequence);
/// let ledger_operations = client.request(endpoint).unwrap();
///
/// assert!(ledger_operations.records().len() > 0);
/// ```
#[derive(Debug, Clone)]
pub struct Operations {
    sequence: u32,
    cursor: Option<String>,
    order: Option<Direction>,
    limit: Option<u32>,
}

impl_cursor!(Operations);
impl_limit!(Operations);
impl_order!(Operations);

impl Operations {
    /// Creates a new ledger::Operations endpoint struct.
    ///
    /// ```
    /// use stellar_client::endpoint::ledger;
    ///
    /// let txns = ledger::Operations::new(123);
    /// ```
    pub fn new(sequence: u32) -> Operations {
        Operations {
            sequence,
            cursor: None,
            order: None,
            limit: None,
        }
    }

    fn has_query(&self) -> bool {
        self.order.is_some() || self.cursor.is_some() || self.limit.is_some()
    }
}

impl IntoRequest for Operations {
    type Response = Records<Operation>;

    fn into_request(self, host: &str) -> Result<Request<Body>> {
        let mut uri = format!("{}/ledgers/{}/operations", host, self.sequence);

        if self.has_query() {
            uri.push_str("?");

            if let Some(order) = self.order {
                uri.push_str(&format!("order={}&", order.to_string()));
            }

            if let Some(cursor) = self.cursor {
                uri.push_str(&format!("cursor={}&", cursor));
            }

            if let Some(limit) = self.limit {
                uri.push_str(&format!("limit={}", limit));
            }
        }

        let uri = Uri::from_str(&uri)?;
        let request = Request::get(uri).body(Body::None)?;
        Ok(request)
    }
}

impl TryFromUri for Operations {
    fn try_from_wrap(wrap: &UriWrap) -> ::std::result::Result<Self, uri::Error> {
        match wrap.path() {
            ["ledgers", sequence, "operations"] => {
                let params = wrap.params();
                Ok(Self {
                    sequence: sequence.parse()?,
                    cursor: params.get_parse("cursor").ok(),
                    order: params.get_parse("order").ok(),
                    limit: params.get_parse("limit").ok(),
                })
            }
            _ => Err(uri::Error::invalid_path()),
        }
    }
}

#[cfg(test)]
mod ledger_operations_tests {
    use super::*;

    #[test]
    fn it_leaves_off_the_params_if_not_specified() {
        let ep = Operations::new(123);
        let req = ep.into_request("https://www.google.com").unwrap();
        assert_eq!(req.uri().path(), "/ledgers/123/operations");
        assert_eq!(req.uri().query(), None);
    }

    #[test]
    fn it_puts_the_query_params_on_the_uri() {
        let ep = Operations::new(123)
            .with_cursor("CURSOR")
            .with_limit(123)
            .with_order(Direction::Desc);
        let req = ep.into_request("https://www.google.com").unwrap();
        assert_eq!(req.uri().path(), "/ledgers/123/operations");
        assert_eq!(
            req.uri().query(),
            Some("order=desc&cursor=CURSOR&limit=123")
        );
    }

    #[test]
    fn it_parses_from_a_uri() {
        let uri: Uri = "/ledgers/123/operations?cursor=CURSOR&order=desc&limit=123"
            .parse()
            .unwrap();
        let ep = Operations::try_from(&uri).unwrap();
        assert_eq!(ep.sequence, 123);
        assert_eq!(ep.limit, Some(123));
        assert_eq!(ep.cursor, Some("CURSOR".to_string()));
        assert_eq!(ep.order, Some(Direction::Desc));
    }
}
