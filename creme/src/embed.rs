use mime::Mime;

#[derive(Debug)]
pub struct EmbeddedAssets {
    pub assets: &'static [EmbeddedAsset],
}

impl EmbeddedAssets {
    pub fn new(assets: &'static [EmbeddedAsset]) -> Self {
        Self { assets }
    }

    pub fn get(&self, index: usize) -> Option<&EmbeddedAsset> {
        self.assets.get(index)
    }
}

#[derive(Debug)]
pub struct EmbeddedAsset {
    pub path: &'static str,
    pub mime: Mime,
    pub content: &'static [u8],
}
