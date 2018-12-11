// If a struct X has a single field called `inner` that impls ToString,
// this macro can be used on that struct (`def_str!(X);`) to allow it
// to be str()'d on the Python side.
macro_rules! def_str {
    ($name:ident) => {
        impl<'p> pyo3::basic::PyObjectStrProtocol<'p> for $name {
            type Result = PyResult<Self::Success>;
            type Success = String;
        }

        impl<'p> pyo3::class::basic::PyObjectProtocol<'p> for $name {
            fn __str__(&'p self) -> <$name as pyo3::basic::PyObjectStrProtocol>::Result {
                Ok(self.inner.to_string())
            }
        }
    };
}

macro_rules! def_query {
    (@ty $ty:ty) => {
        $ty
    };

    (@ty $ty:ty => $($tt:tt)*) => {
        $ty
    };

    // Vec<u8> can be returned to Python directly
    (@into Vec<u8>) => {
        Into::into
    };

    // We pretend there is probably an Into conversion available
    // And map the Vec over it
    (@into Vec<$ty:ty>) => {
        |values| values.clone().into_iter().map(Into::into).collect()
    };

    // We need something super special
    (@into $ty:ty => $($tt:tt)*) => {
        $($tt)*
    };

    (@into $ty:ty) => {
        Into::into
    };

    // Declare a 1-parameter query
    (@new $query:tt($param:ty)) => {
        pub fn new(client: &hedera::Client, _1: $param) -> Self {
            Self {
                inner: $query::new(client, _1),
            }
        }
    };

    // Declare a 2-parameter query
    (@new $query:tt($param1:ty, $param2:ty)) => {
        pub fn new(client: &hedera::Client, _1: $param1, _2: $param2) -> Self {
            Self {
                inner: $query::new(client, _1, _2),
            }
        }
    };

    ($query:tt ( $($param:tt)* ) -> $($ty:tt)+) => {
        mashup! {
            m["py"] = Py $query;
        }

        m! {
            #[pyo3::prelude::pyclass(name = $query)]
            pub struct "py" {
                inner: hedera::query::Query<$query>,
            }

            impl "py" {
                def_query!(@new $query($($param)*));
            }

            #[pyo3::prelude::pymethods]
            impl "py" {
                pub fn get(&mut self) -> pyo3::PyResult<def_query!(@ty $($ty)+)> {
                    self.inner
                        .get()
                        .map(def_query!(@into $($ty)+))
                        .map_err(crate::errors::PyException)
                }

                pub fn cost(&mut self) -> pyo3::PyResult<u64> {
                    self.inner.cost().map_err(crate::errors::PyException)
                }
            }
        }
    };
}

macro_rules! def_transaction {
    // Declare a 1-parameter tx
    (@new $tx:tt()) => {
        pub fn new(client: &hedera::Client) -> Self {
            Self {
                inner: $tx::new(client),
            }
        }
    };

    // Declare a 1-parameter tx
    (@new $tx:tt($param:ty)) => {
        pub fn new(client: &hedera::Client, _1: $param) -> Self {
            Self {
                inner: $tx::new(client, _1),
            }
        }
    };

    // Declare a 2-parameter tx
    (@new $tx:tt($param1:ty, $param2:ty)) => {
        pub fn new(client: &hedera::Client, _1: $param1, _2: $param2) -> Self {
            Self {
                inner: $tx::new(client, _1, _2),
            }
        }
    };

    ($tx:tt ( $($param:tt)* ) { $( fn $builder_name:ident($builder_param:ty); )* }) => {
        mashup! {
            m["py"] = Py $tx;
        }

        m! {
            #[pyo3::prelude::pyclass(name = $tx)]
            pub struct "py" {
                inner: hedera::transaction::Transaction<$tx>,
            }

            impl "py" {
                def_transaction!(@new $tx($($param)*));
            }

            #[pyo3::prelude::pymethods]
            impl "py" {
                pub fn execute(&mut self) -> pyo3::PyResult<crate::PyTransactionId> {
                    self.inner
                        .execute()
                        .map(Into::into)
                        .map_err(crate::errors::PyException)
                }

                pub fn memo(&mut self, memo: &str) -> pyo3::PyResult<()> {
                    self.inner.memo(memo);
                    Ok(())
                }

                pub fn transaction_fee(&mut self, fee: u64) -> pyo3::PyResult<()> {
                    self.inner.fee(fee);
                    Ok(())
                }

                pub fn generate_record(&mut self, generate: bool) -> pyo3::PyResult<()> {
                    self.inner.generate_record(generate);
                    Ok(())
                }

                pub fn sign(&mut self, secret: &crate::PySecretKey) -> pyo3::PyResult<()> {
                    self.inner.sign(&secret.inner);
                    Ok(())
                }

                $(
                    fn $builder_name(&mut self, _1: $builder_param) -> pyo3::PyResult<()> {
                        self.inner.$builder_name(_1.clone().into());
                        // fixme: RETURN SELF
                        Ok(())
                    }
                )*
            }
        }
    };
}
