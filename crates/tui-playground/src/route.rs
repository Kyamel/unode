use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct ParsedRoute {
    pub pathname: String,
    pub query: BTreeMap<String, String>,
}

pub fn parse_route(route: &str) -> ParsedRoute {
    let (pathname, raw_query) = route
        .split_once('?')
        .map_or((route, ""), |(path, query)| (path, query));
    let query = raw_query
        .split('&')
        .filter(|pair| !pair.is_empty())
        .map(|pair| {
            let (key, value) = pair
                .split_once('=')
                .map_or((pair, ""), |(key, value)| (key, value));
            (key.to_string(), value.to_string())
        })
        .collect::<BTreeMap<_, _>>();

    ParsedRoute {
        pathname: pathname.to_string(),
        query,
    }
}
