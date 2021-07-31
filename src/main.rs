extern crate reqwest;
extern crate json;
extern crate serde;
extern crate serde_xml_rs;
extern crate serde_derive;
extern crate serde_json;
extern crate tokio;

use std::collections::{HashMap, BTreeMap};
use json::object;
use std::iter::FromIterator;
use std::array::IntoIter;
use tokio::runtime;

struct DataServiceConfig {
    rootUrl: String,
}

impl DataServiceConfig {
    pub fn new(rootUrl: String) -> Self {
        Self {
            rootUrl
        }
    }
}

struct DataServiceRequestOpts {
    token: String,
    method: reqwest::Method,
    body: String,
    params: HashMap<String, String>,
    urlPath: String,
    headers: HashMap<String, String>,
}

impl DataServiceRequestOpts {
    pub fn new(token: String, method: reqwest::Method, body: String, params: HashMap<String, String>, urlPath: String, headers: HashMap<String, String>) -> Self {
        Self {
            token, method, body, params, urlPath, headers
        }
    }
}

struct DataServiceResponseObject {
    rawData: String,
    dataObject: serde_json::Value,
}

impl Default for DataServiceResponseObject {
    fn default() -> Self {
        return Self { rawData: "".to_owned(), dataObject: serde_json::from_str("{\"data\": \"\"}").unwrap() };
    }
}

impl From<reqwest::Response> for DataServiceResponseObject {
    fn from(response: reqwest::Response) -> Self {
        let mut newObject : &mut DataServiceResponseObject = &mut DataServiceResponseObject::default();
        //assert_eq!(response.status().as_str(), "");
        if !(response.status().is_success()) {
            return Self {
                rawData: "".to_owned(), dataObject: serde_json::from_str("{\"data\": \"\"}").unwrap()
            };
        }
        let tempData = futures::executor::block_on(response.text());
        if tempData.is_err() {
            return Self {
                rawData: "".to_owned(), dataObject: serde_json::from_str("{\"data\": \"\"}").unwrap()
            };
        }
        let tempDataUnwrapped: &String = &(tempData.unwrap());
        newObject.rawData = tempDataUnwrapped.as_str().to_owned();
        let jsonOut: Result<serde_json::Value, _> = serde_json::from_str(newObject.rawData.as_str());
        //assert_eq!(tempDataUnwrapped, "");
        //assert_eq!(jsonOut.is_ok().to_string(), ""); //err().unwrap().to_string(), "");
        if jsonOut.is_ok() {
            newObject.dataObject = jsonOut.unwrap();
        } else {
            let xmlOut = serde_xml_rs::from_str(newObject.rawData.as_str());
            if xmlOut.is_ok() {
                newObject.dataObject = xmlOut.unwrap();
            } else {
                newObject.dataObject = serde_json::from_str("{\"data\": \"\"}").unwrap();
            }
        }
        // Insert custom data here
        return Self {
            rawData: newObject.rawData.to_owned(), dataObject: newObject.dataObject.to_owned()
        };
    }
}

struct DataApiService {
    rootUrl: String,
    client: reqwest::Client,
    //setup: (inputConfig: DataServiceConfig) -> Self,
    //request: (opts: DataServiceRequestOpts) -> reqwest::Response,
}

impl Default for DataApiService {
    fn default() -> Self {
        return Self { rootUrl: "".to_owned(), client: reqwest::Client::new() };
    }
}

impl DataApiService {
    pub fn setup(mut inputConfig: DataServiceConfig) -> Self {
        Self { rootUrl: inputConfig.rootUrl.to_owned(), client: reqwest::Client::new() }
    }
    pub async fn request(&self, opts: DataServiceRequestOpts) -> reqwest::Response {
        let mut headersVar = reqwest::header::HeaderMap::new();
        for (key, value) in &opts.headers {
            headersVar.insert(reqwest::header::HeaderName::from_lowercase(key.to_lowercase().as_bytes()).unwrap(), value.parse().unwrap());
        }
        let mut urlEnd: String = String::from("");
        if opts.urlPath.starts_with("/") {
            urlEnd = opts.urlPath.chars().skip(1).collect();
        } else {
            urlEnd = String::from(opts.urlPath.clone()).clone();
        }
        let mut urlTotal = self.rootUrl.as_str().to_owned();
        urlTotal.push_str(urlEnd.as_str());
        let tokioRuntime = runtime::Builder::new_current_thread().enable_all().build().unwrap();
        return tokioRuntime.block_on(async {
            return self.client.request(opts.method.clone().to_owned(), urlTotal.clone().as_str()).body(opts.body.to_owned()).bearer_auth(opts.token.to_owned()).headers(headersVar).query(&opts.params.to_owned()).send().await.unwrap();
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::array::IntoIter;
    use std::iter::FromIterator;

    #[test]
    fn test_setup() {
        let apiService: DataApiService = DataApiService::setup(DataServiceConfig::new("https://example.com/".to_owned()));
        let apiService2: DataApiService = DataApiService::setup(DataServiceConfig::new("https://example.org/".to_owned()));
        assert_eq!(apiService.rootUrl, "https://example.com/");
        assert_eq!(apiService2.rootUrl, "https://example.org/");
    }

    #[test]
    fn test_request() {
        let apiService: DataApiService = DataApiService::setup(DataServiceConfig::new("http://dummy.restapiexample.com/api/v1/employee/1".to_owned()));
        let response: reqwest::Response = futures::executor::block_on(apiService.request(DataServiceRequestOpts::new("".to_owned(), reqwest::Method::GET, "".to_owned(), HashMap::<_, _>::from_iter(IntoIter::new([/*("123".to_owned(), "456".to_owned()), ("abc".to_owned(), "def".to_owned())*/])), "".to_owned(), HashMap::<_, _>::from_iter(IntoIter::new([])))));
        let output: DataServiceResponseObject = DataServiceResponseObject::from(response);
        //assert_eq!(serde_json::to_string(&output.dataObject).unwrap(), "");
        assert_eq!(output.dataObject.get("status").unwrap().as_str().unwrap(), "success");
    }
}

fn main() {
//    println!("Hello, world!");
}
