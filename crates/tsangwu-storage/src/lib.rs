use anyhow::Result;
use tsangwu_config::StorageConfig;

/// 统一存储服务（基于 OpenDAL）
pub struct StorageService {
    operator: opendal::Operator,
    base_url: String,
}

impl StorageService {
    /// 根据配置初始化存储后端
    pub fn new(config: &StorageConfig) -> Result<Self> {
        let (operator, base_url) = match config {
            #[cfg(feature = "local-fs")]
            StorageConfig::LocalFs { root } => {
                std::fs::create_dir_all(root)?;
                let builder = opendal::services::Fs::default().root(root);
                let op = opendal::Operator::new(builder)?.finish();
                (op, format!("file://{}", root))
            }
            #[cfg(feature = "s3")]
            StorageConfig::S3 {
                endpoint,
                bucket,
                access_key,
                secret_key,
                base_url,
            } => {
                let builder = opendal::services::S3::default()
                    .endpoint(endpoint)
                    .bucket(bucket)
                    .access_key_id(access_key)
                    .secret_access_key(secret_key);
                let op = opendal::Operator::new(builder)?.finish();
                (op, base_url.clone())
            }
            #[allow(unreachable_patterns)]
            _ => anyhow::bail!("当前编译未启用对应存储后端 feature"),
        };
        Ok(Self { operator, base_url })
    }

    /// 上传文件，返回访问 URL
    pub async fn upload(&self, path: &str, data: Vec<u8>) -> Result<String> {
        self.operator.write(path, data).await?;
        Ok(format!("{}/{}", self.base_url, path))
    }

    /// 下载文件
    pub async fn download(&self, path: &str) -> Result<Vec<u8>> {
        let buf = self.operator.read(path).await?;
        Ok(buf.to_vec())
    }

    /// 删除文件
    pub async fn delete(&self, path: &str) -> Result<()> {
        self.operator.delete(path).await?;
        Ok(())
    }

    /// 检查文件是否存在
    pub async fn exists(&self, path: &str) -> Result<bool> {
        Ok(self.operator.exists(path).await?)
    }

    /// 获取基础 URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}
