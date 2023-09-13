#[derive(Debug, Clone)]
pub struct Header {
    pub name: String,
    pub value: String,
}

impl Header {
    pub fn new(name: String, value: String) -> Self {
        Self { name, value }
    }
}

#[derive(Debug)]
pub struct HttpResponse {
    _version: String,
    status_code: u32,
    _reason: String,
    headers: Vec<Header>,
    body: String,
}

impl HttpResponse {
    pub fn new(
        version: String,
        status_code: u32,
        reason: String,
        headers: Vec<Header>,
        body: String,
    ) -> Self {
        Self {
            _version: version,
            status_code,
            _reason: reason,
            headers,
            body,
        }
    }
    pub fn status_code(&self) -> u32 {
        self.status_code
    }

    pub fn body(&self) -> String {
        self.body.clone()
    }

    pub fn header(&self, name: &str) -> String {
        for h in &self.headers {
            if h.name == name {
                return h.value.clone();
            }
        }

        // TODO: return None
        "".to_string()
    }
}
