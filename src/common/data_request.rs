use serde::{de::DeserializeOwned, Serialize};

pub trait DataRequest: Serialize + DeserializeOwned + Clone {
    type DataEnum: Serialize + DeserializeOwned + Clone;
    type DataResponse: Serialize + DeserializeOwned + Clone;
}

#[macro_export]
macro_rules! create_data_enum {
    ($request_enum: ident, {
        $($request_type: ident -> $response_type: ident),*
    }) => {
        #[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
        pub enum $request_enum {
        $(
            $request_type($request_type)
        ),*
        }

        $(
            impl Into<$request_enum> for $request_type {
                fn into(self) -> $request_enum {
                    $request_enum::$request_type(self)
                }
            }

            impl $crate::common::data_request::DataRequest for $request_type {
                type DataEnum = $request_enum;
                type DataResponse = $response_type;
            }
        )*
    };
}
