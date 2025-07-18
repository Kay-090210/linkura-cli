use reqwest::header;

use crate::macros::{define_api_struct, use_common_crate};

use_common_crate!();

define_api_struct!(PetalExchangeApi);

impl<'a> PetalExchangeApi<'a> {
    /// 获取花瓣兑换列表
    pub fn get_list(&self) -> Result<Response> {
        let url = format!("{API_BASE}/petal_exchange/get_list");
        let res = self
            .client
            .post(url)
            .headers(self.runtime_header.clone())
            .header("x-idempotency-key", gen_random_idempotency_key())
            // 与其他接口保持一致，显式声明 content-length=0，避免服务器误判
            .header(header::CONTENT_LENGTH, 0)
            .send()?;
        Ok(res)
    }
} 