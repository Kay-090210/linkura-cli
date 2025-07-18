use crate::config::Global;
use chrono::{Local, Utc};
use linkura_common::jwt::extract_jwt_payload;

pub fn run(ctx: &Global) {
    let api_client = &ctx.api_client;

    match api_client.high_level().get_petal_exchange_list() {
        Ok(petal_json) => {
            print_petal_exchange_info(&petal_json);
        }
        Err(e) => {
            tracing::warn!("Failed to retrieve petal exchange list: {}", e);
        }
    }
}

fn print_latest_trailer_archive(ctx: &Global, wm: &serde_json::Value) {
    let api_client = &ctx.api_client;
    let id = wm.get("live_id").unwrap().as_str().unwrap();
    let live_type = wm.get("live_type").unwrap().as_u64().unwrap();

    let name: &str = wm.get("name").unwrap().as_str().unwrap();
    let description: &str = wm.get("description").unwrap().as_str().unwrap();
    let start_time: &str = wm.get("live_start_time").unwrap().as_str().unwrap();
    let open_time: &str = wm.get("open_time").unwrap().as_str().unwrap();
    tracing::info!(
        "latest {} info: \n{}\n\n{}\nstart_time: {}\nopen_time: {}",
        if live_type == 2 {
            "with meets"
        } else {
            "fes live"
        },
        name,
        description,
        chrono::DateTime::parse_from_rfc3339(start_time)
            .unwrap()
            .with_timezone(&Local)
            .format("%Y-%m-%d %H:%M:%S %:z"),
        chrono::DateTime::parse_from_rfc3339(open_time)
            .unwrap()
            .with_timezone(&Local)
            .format("%Y-%m-%d %H:%M:%S %:z")
    );
    let now = Utc::now();
    if now < chrono::DateTime::parse_from_rfc3339(open_time).unwrap() {
        tracing::warn!(
            "The live has not openned yet! {} {}",
            name,
            chrono::DateTime::parse_from_rfc3339(open_time)
                .unwrap()
                .with_timezone(&Local)
                .format("%Y-%m-%d %H:%M:%S %:z")
        );
        return;
    }
    let maybe_started = Utc::now()
        >= chrono::DateTime::parse_from_rfc3339(start_time).unwrap()
            - chrono::Duration::minutes(10);

    if live_type == 2 {
        let res: Result<serde_json::Value, anyhow::Error> =
            api_client.high_level().get_with_meets_info(id);
        match res {
            Ok(res) => {
                tracing::info!(
                    "with meets info: \n title: {}\n description: {:?}\n room: {:?}\n thumbnail: {:?}\n hls_url: {:?}\n characters: {:?}",
                    name,
                    res.get("description").unwrap().as_str().unwrap(),
                    res.get("room").unwrap().as_object().unwrap(),
                    res.get("thumbnail").unwrap().as_str().unwrap(),
                    res.get("hls_url").unwrap().as_str().unwrap(),
                    res.get("characters").unwrap().as_array().unwrap()
                );
            }
            Err(_) => {
                tracing::warn!(
                    "Can't get latest with meets info for now! {:?} {}",
                    name,
                    id
                );
            }
        }
        if maybe_started {
            let token = api_client.high_level().get_with_meets_connect_token(id);
            match token {
                Ok(token) => {
                    let _ = extract_jwt_payload(token.as_str()).and_then(|payload| {
                        tracing::info!(
                            "Room info: \n address: {}\n port: {}\n room_id: {}",
                            payload["pod"]["address"].as_str().unwrap().to_string(),
                            payload["pod"]["port"].as_u64().unwrap(),
                            payload["room_id"].as_str().unwrap().to_string()
                        );
                        Ok(payload)
                    });
                }
                Err(e) => {
                    tracing::warn!("Failed to get with meets connect token: {}", e);
                }
            }
        }
    }
    if live_type == 1 {
        let res: Result<serde_json::Value, anyhow::Error> =
            api_client.high_level().get_fes_live_info(id);
        match res {
            Ok(res) => {
                tracing::info!(
                    "fes live info: \n title: {}\n description: {:?}\n room: {:?}\n characters: {:?}",
                    name,
                    res.get("description").unwrap().as_str().unwrap(),
                    res.get("room").unwrap().as_object().unwrap(),
                    res.get("characters").unwrap().as_array().unwrap(),
                );
            }
            Err(_) => {
                tracing::warn!("Can't get latest fes live info for now! {:?} {}", name, id);
            }
        }
    }
}

fn print_latest_archive_info(ctx: &Global, archive: &serde_json::Value) {
    let title = archive.get("name").unwrap().as_str().unwrap();
    let description = archive.get("description").unwrap().as_str().unwrap();
    let thumbnail = archive
        .get("thumbnail_image_url")
        .unwrap()
        .as_str()
        .unwrap();
    let link = archive.get("external_link").unwrap().as_str().unwrap();
    let video_url = archive.get("video_url").unwrap().as_str().unwrap();
    let mut real_url = String::new();
    if !link.is_empty() {
        real_url = ctx
            .api_client
            .assets()
            .get_hls_url_from_archive(link)
            .unwrap_or_else(|_| String::new());
    }
    tracing::info!(
        "Latest archive: \n title: {:?}\n description: {:?}\n thumbnail: {:?}\n link: {:?}\n url: {:?}\n video_url: {:?}",
        title,
        description,
        thumbnail,
        link,
        real_url,
        video_url
    );
}

/// 打印花瓣兑换信息
fn print_petal_exchange_info(json: &serde_json::Value) {
    // 当前花瓣数量
    let petal_num = json.get("petal_coin_num").and_then(|v| v.as_u64()).unwrap_or(0);
    tracing::info!("Current petal coin num: {}", petal_num);

    let character_list: Vec<serde_json::Value> = json
        .get("character_petal_exchange_list")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut matching_cards: Vec<(String, String, u64, String)> = Vec::new();

    for character in &character_list {
        let exchange_list: Vec<serde_json::Value> = character
            .get("limit_break_exchange_list")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        for card in &exchange_list {
            let card_datas_id = card.get("card_datas_id").and_then(|v| v.as_u64());
            let item_num = card.get("item_num").and_then(|v| v.as_u64()).unwrap_or(0);

            if let Some(card_id) = card_datas_id {
                if should_keep_card(card_id, item_num) {
                    let card_type = match (card_id / 1_000) % 10 {
                        5 => "UR",
                        9 => "BR",
                        4 => "SR",
                        3 => "R",
                        _ => "Other",
                    };
                    let series_id = card.get("card_series_id").and_then(|v| v.as_u64()).unwrap_or(0);
                    matching_cards.push((series_id.to_string(), card_id.to_string(), item_num, card_type.to_string()));
                }
            }
        }
    }

    // 读取本地 card_id.json 进行名称映射（可选）
    use std::collections::HashMap;
    let mut id_name_map: HashMap<String, String> = HashMap::new();
    let mut card_json_content: Option<String> = None;
    if let Ok(content) = std::fs::read_to_string("card_id.json") {
        card_json_content = Some(content);
    }

    if let Some(content) = card_json_content {
        if let Ok(map_json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(obj) = map_json.as_object() {
                for (k, v) in obj {
                    if let Some(name) = v.as_str() {
                        id_name_map.insert(k.clone(), name.to_string());
                    }
                }
            }
        }
    }

    if matching_cards.is_empty() {
        tracing::info!("No cards matched the filter condition");
    } else {
        tracing::info!("Found {} matching cards:", matching_cards.len());
        for (series_id, _card_id, item_num, card_type) in matching_cards {
            let card_name = id_name_map.get(&series_id).cloned().unwrap_or(series_id.clone());
            tracing::info!("{} ({}) - num: {}", card_name, card_type, item_num);
        }
    }
}

/// 过滤逻辑，根据 card_datas_id 的第五位数字进行过滤
fn should_keep_card(card_datas_id: u64, item_num: u64) -> bool {
    let fifth_digit = (card_datas_id / 1_000) % 10;

    match fifth_digit {
        5 => item_num > 100, // UR
        9 => item_num > 75,  // BR
        4 => item_num > 50,  // SR
        3 => item_num > 25,  // R
        _ => item_num > 0,
    }
}
