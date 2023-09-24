use crate::config::Config as PhoenixConfig;

pub fn get_pomm_config() -> anyhow::Result<PhoenixConfig> {
    let home_path = dirs::home_dir().ok_or(anyhow::anyhow!("can't open home dir"))?;
    let pomm_config_path = home_path.join(".config").join("pomm");
    let config_path = pomm_config_path.join("config.toml");

    // 读取配置文件
    let config_str = std::fs::read_to_string(config_path)?;
    // 解析配置文件
    let phoneix_config: PhoenixConfig = toml::from_str(&config_str)?;

    Ok(phoneix_config)
}
